//! Permission identity comes from the ACP peer, not an authenticated tool origin.
use agent_client_protocol::v1::schema::*;
use std::collections::{HashMap, HashSet};

const MAX_CALLS: usize = 1024;

#[derive(Default)]
pub(crate) struct Permissions {
    session: Option<SessionId>,
    tools: HashSet<String>,
    // None is a tombstone: conflicting/terminal identities cannot be resurrected.
    calls: HashMap<ToolCallId, Option<String>>,
    exhausted: bool,
}
impl Permissions {
    pub(crate) fn begin(&mut self, session: SessionId, tools: impl Iterator<Item = String>) {
        self.end();
        self.session = Some(session);
        self.tools.extend(tools);
        self.tools.insert("return_result".into());
    }
    pub(crate) fn end(&mut self) {
        *self = Self::default();
    }
    fn identity(&self, name: Option<&str>, title: Option<&str>) -> Result<Option<String>, ()> {
        let parse = |s: &str| {
            s.strip_prefix("mcp__thinkwell__")
                .or_else(|| s.strip_prefix("mcp.thinkwell."))
                .filter(|s| self.tools.contains(*s))
                .map(str::to_owned)
        };
        let named = match name {
            Some(name) => Some(parse(name).ok_or(())?),
            None => None,
        };
        let titled = title.and_then(parse);
        if let Some(named) = named {
            if titled.as_ref().is_some_and(|title| title != &named) {
                return Err(());
            }
            Ok(Some(named))
        } else if title.is_some() && titled.is_none() {
            Err(())
        } else {
            Ok(titled)
        }
    }
    fn remember(&mut self, id: ToolCallId, identity: Option<String>) {
        if let Some(previous) = self.calls.get_mut(&id) {
            if *previous != identity {
                *previous = None;
            }
        } else if self.calls.len() < MAX_CALLS {
            self.calls.insert(id, identity);
        } else {
            // Do not evict tombstones and accidentally accept reused IDs.
            self.exhausted = true;
            self.calls.clear();
        }
    }
    pub(crate) fn observe(&mut self, notification: SessionNotification) {
        if self.exhausted || self.session.as_ref() != Some(&notification.session_id) {
            return;
        }
        match notification.update {
            SessionUpdate::ToolCall(call) => {
                let identity = self
                    .identity(call.name.as_deref(), Some(&call.title))
                    .ok()
                    .flatten();
                let identity = if terminal(call.status) {
                    None
                } else {
                    identity
                };
                self.remember(call.tool_call_id, identity);
            }
            SessionUpdate::ToolCallUpdate(call) => {
                let previous = self.calls.get(&call.tool_call_id).cloned().flatten();
                let fields = call.fields;
                let identity = self.identity(fields.name.as_deref(), fields.title.as_deref());
                let next = match identity {
                    Ok(None) => previous.clone(),
                    Ok(Some(name)) if previous.as_ref() == Some(&name) => Some(name),
                    _ => None,
                };
                self.remember(
                    call.tool_call_id,
                    if fields.status.is_some_and(terminal) {
                        None
                    } else {
                        next
                    },
                );
            }
            _ => {}
        }
    }
    pub(crate) fn allows(&mut self, request: &RequestPermissionRequest) -> bool {
        if self.exhausted || self.session.as_ref() != Some(&request.session_id) {
            return false;
        }
        let call = &request.tool_call;
        let identity = self.identity(call.fields.name.as_deref(), call.fields.title.as_deref());
        let identity = if call.fields.status.is_some_and(terminal) {
            None
        } else {
            match identity {
                Ok(Some(name)) => Some(name),
                Ok(None) => self.calls.get(&call.tool_call_id).cloned().flatten(),
                Err(()) => None,
            }
        };
        self.remember(call.tool_call_id.clone(), identity);
        !self.exhausted
            && self
                .calls
                .get(&call.tool_call_id)
                .is_some_and(Option::is_some)
    }
}
fn terminal(status: ToolCallStatus) -> bool {
    !matches!(status, ToolCallStatus::Pending | ToolCallStatus::InProgress)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn state() -> Permissions {
        let mut p = Permissions::default();
        p.begin("session".into(), ["add".into()].into_iter());
        p
    }
    fn request(session: &str, id: &str, fields: Value) -> RequestPermissionRequest {
        let mut call = json!({"toolCallId": id});
        call.as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        serde_json::from_value(json!({"sessionId":session,"toolCall":call,"options":[]})).unwrap()
    }
    fn observe(p: &mut Permissions, session: &str, update: Value) {
        p.observe(serde_json::from_value(json!({"sessionId":session,"update":update})).unwrap());
    }
    fn announce(p: &mut Permissions, id: &str, title: &str) {
        observe(
            p,
            "session",
            json!({"sessionUpdate":"tool_call","toolCallId":id,"title":title}),
        );
    }
    #[test]
    fn names_and_partial_updates() {
        for title in ["mcp__thinkwell__add", "mcp.thinkwell.add"] {
            let mut p = state();
            announce(&mut p, "call", title);
            observe(
                &mut p,
                "session",
                json!({"sessionUpdate":"tool_call_update","toolCallId":"call","status":"in_progress"}),
            );
            assert!(p.allows(&request("session", "call", json!({}))));
            assert!(p.allows(&request(
                "session",
                "result",
                json!({"title":"mcp__thinkwell__return_result"})
            )));
        }
    }
    #[test]
    fn rejects_unknown_conflicting_terminal_and_unrelated_calls() {
        for fields in [
            json!({"title":"shell"}),
            json!({"name":"shell","title":"mcp.thinkwell.add"}),
            json!({"name":"mcp.thinkwell.add","title":"mcp.thinkwell.return_result"}),
            json!({"title":"mcp.thinkwell.undeclared"}),
            json!({"title":"mcp.other.add"}),
            json!({"status":"completed"}),
            json!({"status":"failed"}),
        ] {
            let mut p = state();
            announce(&mut p, "call", "mcp.thinkwell.add");
            assert!(!p.allows(&request("session", "call", fields)));
            announce(&mut p, "call", "mcp.thinkwell.add");
            assert!(!p.allows(&request("session", "call", json!({}))));
        }
        let mut p = state();
        assert!(!p.allows(&request("session", "unknown", json!({}))));
        announce(&mut p, "call", "mcp.thinkwell.add");
        observe(
            &mut p,
            "session",
            json!({"sessionUpdate":"tool_call_update","toolCallId":"call","status":"completed"}),
        );
        assert!(!p.allows(&request(
            "session",
            "call",
            json!({"title":"mcp.thinkwell.add"})
        )));
        announce(&mut p, "conflict", "mcp.thinkwell.add");
        announce(&mut p, "conflict", "mcp.thinkwell.return_result");
        assert!(!p.allows(&request("session", "conflict", json!({}))));
    }
    #[test]
    fn session_turn_and_capacity_boundaries() {
        let mut p = state();
        observe(
            &mut p,
            "other",
            json!({"sessionUpdate":"tool_call","toolCallId":"foreign","title":"mcp.thinkwell.add"}),
        );
        assert!(!p.allows(&request("session", "foreign", json!({}))));
        announce(&mut p, "call", "mcp.thinkwell.add");
        assert!(!p.allows(&request(
            "other",
            "call",
            json!({"title":"mcp.thinkwell.add"})
        )));
        p.end();
        assert!(!p.allows(&request(
            "session",
            "call",
            json!({"title":"mcp.thinkwell.add"})
        )));
        p.begin("session".into(), ["add".into()].into_iter());
        assert!(!p.allows(&request("session", "call", json!({}))));
        for i in 0..=MAX_CALLS {
            announce(&mut p, &format!("call-{i}"), "mcp.thinkwell.add");
        }
        assert!(p.exhausted);
        assert!(p.calls.is_empty());
        assert!(!p.allows(&request(
            "session",
            "new",
            json!({"title":"mcp.thinkwell.add"})
        )));
        p.begin("next".into(), ["add".into()].into_iter());
        assert!(p.allows(&request(
            "next",
            "new",
            json!({"title":"mcp.thinkwell.add"})
        )));
    }
}
