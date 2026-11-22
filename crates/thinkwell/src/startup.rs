//! Bounded correlation for adapter startup reports, including reports received
//! before session/new supplies the ID and after it has completed.
use agent_client_protocol::v1::schema::SessionId;
use std::collections::HashSet;

const MAX_PENDING: usize = 64;

#[derive(Default)]
pub(crate) struct Startup {
    opening: bool,
    pending: HashSet<SessionId>,
    overflow: bool,
    current: Option<SessionId>,
    failed: bool,
}
impl Startup {
    pub(crate) fn begin(&mut self) {
        *self = Self {
            opening: true,
            ..Self::default()
        };
    }
    pub(crate) fn select(&mut self, id: SessionId) {
        self.failed = self.overflow || self.pending.contains(&id);
        self.current = Some(id);
        self.opening = false;
        self.pending.clear();
    }
    pub(crate) fn report(&mut self, id: SessionId) {
        if self.opening {
            if self.pending.len() < MAX_PENDING || self.pending.contains(&id) {
                self.pending.insert(id);
            } else {
                // Fail setup closed on an excessive pre-response report stream.
                self.overflow = true;
            }
        } else if self.current.as_ref() == Some(&id) {
            self.failed = true;
        }
    }
    pub(crate) fn failed(&self) -> bool {
        self.failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_reports_are_cleared_and_correlated() {
        let mut state = Startup::default();
        state.begin();
        state.report("old".into());
        state.select("new".into());
        assert!(!state.failed());
        assert!(state.pending.is_empty());
        state.report("old".into());
        assert!(!state.failed());
        state.report("new".into());
        assert!(state.failed());
        state.begin();
        for i in 0..=MAX_PENDING {
            state.report(format!("session-{i}").into());
        }
        assert_eq!(state.pending.len(), MAX_PENDING);
        state.select("new".into());
        assert!(state.failed());
        state.begin();
        state.select("replacement".into());
        assert!(!state.failed());
    }
}
