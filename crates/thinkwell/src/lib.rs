//! Typed ACP conversations with borrowed local tools.
//!
//! Requires a Tokio runtime with I/O/time enabled. Callbacks may be non-Send.
//! Cancellation by dropping a started execution closes its session and connection.
//! Drop releases local access and requests cleanup; use Agent::shutdown for confirmation.
mod api;
mod runtime;
mod schema;

pub use api::*;
pub use runtime::{Agent, Diagnostics, Session, SessionBuilder};

/// Structured categories; details retain context without exposing SDK error types.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    Configuration,
    Contract,
    InvalidValue,
    Binding,
    Busy,
    Closed,
    Transport,
    UnsupportedModel,
    MissingResult,
    ResultBudget,
    CallBudget,
    Deadline,
    Cancelled,
    RemoteStop,
    CallbackOverlap,
    Tool,
    Cleanup,
}

/// An error category and human-readable context (including schema paths).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}
impl Error {
    pub fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
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
pub type Result<T> = std::result::Result<T, Error>;
