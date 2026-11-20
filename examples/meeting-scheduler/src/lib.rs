//! Application helpers for the meeting scheduler example, not additions to Thinkwell's API.

mod model;
mod solver;
mod workflow;

pub use model::{
    Attendee, AttendeeAvailability, Day, Explanation, ParsedAvailability, Slot, TimeWindow,
    ValidatedAvailability, validate_availability, validate_roster,
};
pub use solver::{Schedule, solve};
pub use workflow::{explain, extract, format_extraction};

/// Errors from input validation, solving, or an agent request.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
