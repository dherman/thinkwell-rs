//! Typed ACP conversations that alternate ordinary Rust code with LLM prompts.
//!
//! Thinkwell acts as a headless Agent Client Protocol client. An [Agent] owns a
//! subprocess connection, a [Session] retains conversation history, a [Tool]
//! describes a contract, and a [Plan] supplies a prompt, result type, and per-turn
//! callback bindings. Callbacks can borrow application state without Send or
//! static requirements.
//!
//! # Quick start
//!
//! Install and authenticate the Claude ACP adapter separately and make version
//! 0.23.1 available in the npm cache. [Claude] uses offline npx; Thinkwell does not
//! install software or obtain credentials. Use Tokio with I/O and time enabled.
//!
//! ~~~no_run
//! use thinkwell::{Claude, Plan};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> thinkwell::Result<()> {
//!     let agent = Claude::connect().await?;
//!     let mut session = agent.session().open().await?;
//!     let answer = Plan::<String>::new()
//!         .text("Explain ACP in one sentence.")
//!         .run_in(&mut session)
//!         .await?;
//!     println!("{answer}");
//!     Ok(())
//! }
//! ~~~
//!
//! This uses the agent's default model. [SessionBuilder::model] selects an
//! agent-supported identifier. [AgentConfig] and [Connect] support custom launch
//! commands and application-defined presets.
//!
//! # Tools and results
//!
//! Declare typed contracts before opening a session and bind their implementations
//! on every turn. [Plan] has a complete example of borrowed mutable state, deterministic
//! work between turns, and changing result types in the same conversation.
//! Tool inputs derive Serde Deserialize and schemars JsonSchema; outputs derive
//! Serialize and JsonSchema; final results derive Deserialize and JsonSchema.
//!
//! The catalog stays fixed while the callback captures and final-result type may
//! change. Internal turn identifiers and protocol envelopes are library details,
//! not parameters in application closures.
//!
//! # Ownership, cancellation, and cleanup
//!
//! Only one session can be open on a connection. Each execution exclusively borrows
//! its session and owns its local callbacks. A normally dropped session permits
//! a fresh conversation on the same agent. Failed or abandoned started turns
//! permanently close both session and connection; recovery means explicitly
//! connecting again. Preflight failures and dropping an unpolled execution do not
//! start a turn.
//!
//! Ordinary Drop revokes local access and signals an existing cleanup driver.
//! It does not await remote acknowledgment, roll back mutations, or perform arbitrary
//! asynchronous callback cleanup. Process guards provide fallback when the driver or
//! runtime is destroyed. [Agent::shutdown] optionally confirms bounded cleanup;
//! [Diagnostics] observes a recorded outcome without retaining the owner.
//!
//! # Schema support
//!
//! The initial validator supports object properties/required fields and boolean
//! additionalProperties, homogeneous arrays, primitive types, const/enum, and
//! oneOf/anyOf for tagged enums. The int64 format is enforced. Only the 2020-12 dialect
//! is accepted. Unknown keywords, references/$defs, other formats, and unsupported
//! constraints fail before prompting with [ErrorKind::Contract].
//!
//! Consequently some ordinary Rust types, including nested named types generating
//! references and numeric types generating other formats, are currently unsupported.
//! Integer contracts reject floating representations such as 3.0. Numeric const/enum
//! comparisons avoid converting large integers to f64. Validation is separate from
//! Serde conversion and applies to tool inputs, serialized outputs, and final results.
//!
//! # Initial scope
//!
//! The runtime target is Tokio on macOS/Linux, on current-thread or multithread
//! runtimes; callbacks stay on the calling task and may be non-Send. The crate is
//! unpublished and its API is evolving. HTTP MCP 2025-11-25 and Claude ACP 0.23.1 are
//! the initial interoperability path. Concurrent sessions on one connection, Windows,
//! shared-host embedding, progress streams, and arbitrary JSON Schema are outside
//! this initial slice.
//!
//! A matching turn identifier controls dispatch, but does not prove causal origin
//! if a sender supplies the current identifier for work begun earlier. Effects from
//! already executed tools survive cancellation. Configure [RunOptions] to bound work;
//! [ErrorKind] distinguishes preflight, execution, and cleanup failures.
#![deny(missing_docs)]

mod api;
mod runtime;
mod schema;

pub use api::*;
pub use runtime::{Agent, Diagnostics, Session, SessionBuilder};

/// Stable categories for configuration, execution, and cleanup failures.
///
/// Match categories rather than parsing [Error::message]. This enum is
/// non-exhaustive; downstream matches need a fallback for future categories.
/// Some errors are returned to the agent as tool errors instead of terminating
/// a turn. See [Plan::run_in] and [RunOptions] for those distinctions.
///
/// # Example
///
/// ~~~rust
/// use thinkwell::{Error, ErrorKind};
/// let error = Error::new(ErrorKind::Binding, "missing tool binding");
/// let action = match error.kind {
///     ErrorKind::Binding | ErrorKind::Contract => "fix the plan before retrying",
///     ErrorKind::Closed | ErrorKind::Cancelled => "connect again",
///     _ => "inspect the error and session state",
/// };
/// assert_eq!(action, "fix the plan before retrying");
/// ~~~
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Invalid launch/run options or a missing Tokio runtime.
    Configuration,
    /// A schema uses an unsupported dialect, keyword, format, or structure.
    Contract,
    /// A tool or result value violates its contract or cannot be converted.
    InvalidValue,
    /// Tool names, declarations, or per-turn bindings are invalid or mismatched.
    Binding,
    /// Another session holds the connection's one-session reservation.
    Busy,
    /// The session/connection is unusable or its driver is unavailable.
    Closed,
    /// Agent launch, protocol setup, communication, or response delivery failed.
    Transport,
    /// The requested session model-selection operation failed.
    UnsupportedModel,
    /// The agent ended normally without an accepted typed result.
    MissingResult,
    /// Invalid result submissions exhausted the configured correction budget.
    ResultBudget,
    /// Application-tool requests exceeded the configured call budget.
    CallBudget,
    /// An operation exceeded its time bound; context identifies which operation.
    Deadline,
    /// The agent returned a cancelled stop reason. Local future drop has no return value.
    Cancelled,
    /// The agent returned a terminal reason other than normal completion or cancellation.
    RemoteStop,
    /// A terminal response arrived while a local callback was still active.
    CallbackOverlap,
    /// An application callback reported a domain-specific failure.
    Tool,
    /// Connection/process or HTTP cleanup failed; confirmation was not successful.
    Cleanup,
}

/// An error category and human-readable context.
///
/// Implements [std::error::Error] and Display for application error propagation.
/// Use [kind](Self::kind) for programmatic classification; message text is
/// diagnostic context and is not a stable parsing interface. Schema errors
/// include the unsupported schema location where available.
///
/// # Example: reject an application tool input
///
/// ~~~rust
/// use thinkwell::{Error, Result};
/// fn require_positive(value: i64) -> Result<i64> {
///     if value <= 0 {
///         return Err(Error::tool("amount must be positive"));
///     }
///     Ok(value)
/// }
/// assert!(require_positive(0).is_err());
/// ~~~
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// Machine-readable failure category; match with a fallback for future variants.
    pub kind: ErrorKind,
    /// Diagnostic explanation, potentially including schema paths or transport context.
    pub message: String,
}
impl Error {
    /// Construct an error with an explicit category and contextual explanation.
    ///
    /// For application callback failures, prefer [Error::tool]. Library operations
    /// choose categories to distinguish configuration, turn, and cleanup failures.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
    /// Create an application-level callback error with [ErrorKind::Tool].
    ///
    /// A bound callback returning this error reports a tool failure to the agent,
    /// which may handle it and continue. It does not roll back earlier mutations.
    ///
    /// ~~~rust
    /// use thinkwell::{Error, Plan, Tool};
    /// let divide = Tool::<i64, i64>::new("divide", "Divide 100 by an integer");
    /// let plan = Plan::<String>::new().bind(&divide, async |divisor: i64| {
    ///     if divisor == 0 {
    ///         return Err(Error::tool("divisor must not be zero"));
    ///     }
    ///     Ok(100 / divisor)
    /// });
    /// ~~~
    pub fn tool(message: impl Into<String>) -> Self {
        Self::new(ErrorKind::Tool, message)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)
    }
}
impl std::error::Error for Error {}
/// A Thinkwell operation or tool callback result, using [Error].
///
/// Callbacks return this type so application errors can be reported to the agent.
///
/// ~~~rust
/// fn normalize(text: String) -> thinkwell::Result<String> {
///     Ok(text.trim().to_owned())
/// }
/// assert_eq!(normalize(" hello ".into()).unwrap(), "hello");
/// ~~~
pub type Result<T> = std::result::Result<T, Error>;
