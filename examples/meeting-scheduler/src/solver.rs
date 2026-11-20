use crate::model::{Day, Slot, ValidatedAvailability};
use z3::{
    Params, SatResult, Solver,
    ast::{Bool, Int},
};

/// Canonical outcome from deterministic computation, never chosen by the agent.
#[derive(Debug, PartialEq, Eq)]
pub enum Schedule {
    /// At least one slot works; this is one of them, not necessarily the earliest.
    Scheduled(Slot),
    /// No slot satisfies the validated constraints.
    NoSlot,
    /// The solver could not establish an answer, for example because of its timeout.
    Inconclusive(String),
}
impl std::fmt::Display for Schedule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scheduled(slot) => write!(
                f,
                "Scheduled: {} {:02}:00–{:02}:00 (shared timezone)",
                slot.day,
                slot.start_hour,
                slot.start_hour + 1
            ),
            Self::NoSlot => f.write_str("No feasible one-hour meeting in weekday business hours."),
            Self::Inconclusive(reason) => write!(
                f,
                "Inconclusive: {reason}. No feasibility claim was established."
            ),
        }
    }
}

// Keep status interpretation separate so Unknown can be tested without a flaky timeout.
fn interpret(
    status: SatResult,
    model_slot: impl FnOnce() -> crate::Result<Slot>,
    reason: impl FnOnce() -> String,
) -> crate::Result<Schedule> {
    match status {
        SatResult::Sat => Ok(Schedule::Scheduled(model_slot()?)),
        SatResult::Unsat => Ok(Schedule::NoSlot),
        SatResult::Unknown => Ok(Schedule::Inconclusive(reason())),
    }
}

/// Solve for a whole one-hour meeting, with a ten-second Z3 query timeout.
///
/// The solver checks the extracted constraints, not the correctness of the agent's
/// interpretation. Model extraction and independent validation failures are errors.
pub fn solve(availability: &ValidatedAvailability) -> crate::Result<Schedule> {
    let day = Int::fresh_const("day");
    let hour = Int::fresh_const("hour");
    let solver = Solver::new();
    let mut params = Params::new();
    params.set_u32("timeout", 10_000);
    solver.set_params(&params);
    solver.assert(day.ge(0));
    solver.assert(day.lt(5));
    solver.assert(hour.ge(9));
    solver.assert(hour.lt(17));
    for attendee in &availability.0.attendees {
        let windows: Vec<Bool> = attendee
            .windows
            .iter()
            .map(|window| {
                Bool::and(&[
                    day.eq(window.day.index()),
                    hour.ge(window.start_hour),
                    (&hour + 1).le(window.end_hour),
                ])
            })
            .collect();
        // The disjunction of no windows is false, not unconstrained availability.
        solver.assert(Bool::or(&windows));
    }
    let outcome = interpret(
        solver.check(),
        || {
            let model = solver
                .get_model()
                .ok_or("Z3 returned Sat without a model")?;
            let day = model
                .eval(&day, true)
                .and_then(|n| n.as_i64())
                .ok_or("Z3 did not return an integer weekday")?;
            let hour = model
                .eval(&hour, true)
                .and_then(|n| n.as_i64())
                .ok_or("Z3 did not return an integer hour")?;
            let day = usize::try_from(day)
                .ok()
                .and_then(|i| Day::ALL.get(i))
                .copied()
                .ok_or("Z3 returned an invalid weekday")?;
            let slot = Slot {
                day,
                start_hour: hour,
            };
            if !availability.permits(slot) {
                return Err("Z3's slot failed the independent availability check".into());
            }
            Ok(slot)
        },
        || {
            solver
                .get_reason_unknown()
                .unwrap_or_else(|| "solver supplied no reason".into())
        },
    )?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_is_not_unsatisfiable_and_does_not_read_a_model() {
        let result = interpret(
            SatResult::Unknown,
            || panic!("no model"),
            || "timeout".into(),
        )
        .unwrap();
        assert_eq!(result, Schedule::Inconclusive("timeout".into()));
        assert!(!result.to_string().contains("No feasible"));
    }

    #[test]
    fn invalid_model_is_an_error_not_no_slot() {
        assert!(
            interpret(
                SatResult::Sat,
                || Err("missing model".into()),
                || panic!("no reason")
            )
            .is_err()
        );
        assert_eq!(
            interpret(
                SatResult::Unsat,
                || panic!("no model"),
                || panic!("no reason")
            )
            .unwrap(),
            Schedule::NoSlot
        );
    }
}
