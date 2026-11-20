//! Use an agent from Rust, then work with its answer as an ordinary Rust value.
//!
//! Thinkwell lets you alternate between prompts and code: ask an agent for an
//! answer, process it in Rust, and ask a follow-up question. You choose the answer's
//! type. You can also give the agent tools that call your Rust functions.
//!
//! # Your first conversation
//!
//! There are three steps: connect to an agent, open a conversation, and send a
//! request. Here we ask Claude for a title and receive a `String`:
//!
//! ~~~no_run
//! use thinkwell::{Claude, Plan};
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> thinkwell::Result<()> {
//!     let agent = Claude::connect().await?;
//!     let mut session = agent.session().open().await?;
//!
//!     let title = Plan::<String>::new()
//!         .text("Suggest a title for a beginner's gardening guide.")
//!         .run_in(&mut session)
//!         .await?;
//!
//!     println!("{title}");
//!     Ok(())
//! }
//! ~~~
//!
//! An [Agent] is your connection to an agent program. A [Session] is one conversation:
//! it remembers earlier questions and answers. A [Plan] describes a single request,
//! including its prompt and the Rust type you want back. Creating a plan sends
//! nothing; `run_in(...).await` sends it and waits for the answer.
//!
//! To run this example, use Tokio with its macros, runtime, I/O, and time support
//! (the `full` feature includes these). You also need Node/npx, the Claude ACP
//! adapter version 0.23.1 in your npm cache, and working Claude authentication.
//! See [Claude] for the exact command Thinkwell runs. Thinkwell does not install
//! or authenticate the agent for you.
//!
//! # Build on the first example
//!
//! - **Continue the conversation.** Run another plan in the same [Session]. Its
//!   example shows a follow-up question that uses the earlier answer.
//! - **Ask for a struct.** Derive `Deserialize` and `JsonSchema` for your answer type.
//!   [Plan] shows how to ask for a suggestion with a title and a reason.
//! - **Let the agent use your code.** Describe an operation with [Tool], then provide
//!   its Rust function with [Plan::bind]. The examples count words and update a
//!   shopping list borrowed from your application.
//! - **Adjust the defaults.** Choose a model with [SessionBuilder::model], set time
//!   and tool-call limits with [RunOptions], or launch another agent with [AgentConfig].
//!
//! Each request can return a different type. Tool functions can borrow local state,
//! so you can use ordinary Rust values before and after a request without moving
//! all your application logic into the agent.
//!
//! # Finishing and handling errors
//!
//! When a request succeeds, its answer is ready to use and its tool work is finished.
//! You do not need a separate call to finish the request. Let the session and agent
//! go out of scope when you are done; Thinkwell starts cleanup automatically.
//! [Agent::shutdown] is optional, for applications that need to wait for confirmation
//! that cleanup completed.
//!
//! Thinkwell checks a plan before sending it. If those checks fail, you can correct
//! the plan and try again in the same session. If a request fails after it starts,
//! the session and connection close; connect again to continue. Cancelling a running
//! request by dropping its future also closes the connection. Changes already made
//! by tools are not undone. [Plan::run_in] explains these cases in more detail.
//!
//! # Current limitations
//!
//! This is an early version for Tokio on macOS and Linux. Each connection supports
//! one open conversation at a time; use separate agents for concurrent conversations.
//! Choose a conversation's tools when opening it and supply their functions on every
//! request. You cannot add new tools to an already open conversation.
//!
//! Thinkwell talks to agents using ACP (Agent Client Protocol), rather than calling
//! model providers directly. The initial tested combination is the Claude ACP
//! adapter 0.23.1 with HTTP MCP 2025-11-25 for tools. Other agents must support the
//! required protocols. Windows, progress streams, and shared-host embedding are
//! not supported in this version. The crate is unpublished and its API is evolving.
//!
//! ## Schema support
//!
//! Thinkwell uses JSON Schema to describe the types of answers and tool values it
//! accepts. Start with strings, booleans, `i64`, lists, or simple structs like the
//! one in [Plan]. Not every Rust type that derives `JsonSchema` is supported yet.
//! Unsupported schemas produce [ErrorKind::Contract] before a prompt is sent.
//!
//! For reference, the validator accepts the 2020-12 dialect and these schema features:
//! object properties and required fields, boolean `additionalProperties`, homogeneous
//! arrays, primitive types, `const`/`enum`, `oneOf`/`anyOf`, and the `int64` format.
//! References (`$ref`/`$defs`), other formats, and other constraints are rejected.
//! This currently excludes some nested named types and numeric types.
//!
//! Validation checks tool inputs, tool outputs, and final answers as well as their
//! Serde conversions. Integer fields reject floating representations such as `3.0`.
//! Numeric `const`/`enum` comparisons preserve precision for large integers.
#![deny(missing_docs)]

mod api;
mod runtime;
mod schema;

pub use api::*;
pub use runtime::{Agent, Diagnostics, Session, SessionBuilder};

/// The kind of problem that caused an operation to fail.
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
    /// Another conversation is already open on this agent.
    Busy,
    /// The conversation or agent connection is no longer available.
    Closed,
    /// Agent launch, protocol setup, communication, or response delivery failed.
    Transport,
    /// The requested session model-selection operation failed.
    UnsupportedModel,
    /// The agent ended normally without an accepted typed result.
    MissingResult,
    /// The agent tried too many times to supply an answer of the requested type.
    ResultBudget,
    /// The agent requested more tool calls than allowed by [RunOptions::max_tool_calls].
    CallBudget,
    /// An operation took too long; the error message explains which one.
    Deadline,
    /// The agent reported that the request was cancelled. Dropping a future returns no error.
    Cancelled,
    /// The agent stopped for a reason other than success or cancellation.
    RemoteStop,
    /// The agent finished its request while one of your tool functions was still running.
    CallbackOverlap,
    /// One of your tool functions returned an error.
    Tool,
    /// Thinkwell could not confirm successful cleanup of the agent or tool server.
    Cleanup,
}

/// An explanation of a failed operation.
///
/// Use `?` to pass errors to your caller, or display them with `eprintln!("{error}")`.
/// If you want to handle particular problems differently, match [kind](Self::kind).
/// The [message](Self::message) is for people to read; avoid parsing it in code.
///
/// Tool functions can use [Error::tool] to explain why an operation could not be
/// performed. The agent receives that explanation and may try another approach:
///
/// ~~~rust
/// use thinkwell::{Error, Result};
///
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
    /// What kind of problem occurred. Include a fallback when matching this enum.
    pub kind: ErrorKind,
    /// An explanation of what went wrong, for display or logging.
    pub message: String,
}
impl Error {
    /// Create an error with a category and an explanation.
    ///
    /// Use [Error::tool] for errors from your tool functions. This constructor is
    /// useful when you need to choose another [ErrorKind] explicitly.
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
    /// Tell the agent why a tool could not complete its work.
    ///
    /// For example, reject division by zero before attempting the calculation:
    ///
    /// ~~~rust
    /// use thinkwell::{Error, Plan, Tool};
    ///
    /// let divide = Tool::<i64, i64>::new("divide", "Divide 100 by an integer");
    /// let plan = Plan::<String>::new().bind(&divide, async |divisor: i64| {
    ///     if divisor == 0 {
    ///         return Err(Error::tool("divisor must not be zero"));
    ///     }
    ///     Ok(100 / divisor)
    /// });
    /// ~~~
    ///
    /// This creates an error with [ErrorKind::Tool]. Returning it from a bound function
    /// reports the failure to the agent, which may continue the conversation. It does
    /// not undo any changes your function already made.
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
