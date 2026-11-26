//! Optional, best-effort progress for a typed execution.
use crate::{Agent, Plan, Result, Session};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use std::{
    collections::VecDeque,
    future::Future,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

/// An update you can display while waiting for a typed answer.
///
/// Match the updates you need and ignore the rest. New kinds may be added:
///
/// ```no_run
/// use thinkwell::{Plan, ProgressEvent, agent::Codex};
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Codex::connect().await?;
/// let answer = Plan::<String>::new().text("Suggest a garden name.")
///     .observe(|event| match event {
///         ProgressEvent::Message(message) => eprint!("{}", message.text()),
///         ProgressEvent::ToolStarted(tool) => eprintln!("Running {}", tool.name()),
///         _ => {}
///     }).run(&agent).await?;
/// println!("Answer: {answer}");
/// # Ok(())
/// # }
/// ```
///
/// Progress is not a complete transcript or an audit log. It can be dropped when
/// the bounded queue fills; [Self::EventsDropped] reports that loss. The awaited
/// result, not a message or [Self::ResultAccepted], determines success. Agent
/// text and titles can contain application data; Thinkwell does not save them.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum ProgressEvent {
    /// User-visible text reported by the agent; not the validated answer.
    Message(MessageProgress),
    /// Agent-reported tool activity, which may describe native work.
    AgentTool(AgentToolProgress),
    /// Thinkwell is beginning an application tool invocation, including validation.
    ToolStarted(ToolProgress),
    /// Thinkwell finished an application tool invocation.
    ToolFinished(ToolCompletion),
    /// A final-result submission was rejected; correction may still succeed.
    ResultRejected(ResultRejection),
    /// A validated candidate exists, but the turn may still fail.
    ResultAccepted(ResultAcceptance),
    /// Updates were omitted to keep observation within its memory budget.
    EventsDropped(DroppedEvents),
}

/// How a local tool invocation ended. Earlier effects are not rolled back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ToolOutcome {
    /// Input, callback and output validation succeeded.
    Succeeded,
    /// Input validation, the callback, or output validation failed.
    Failed,
    /// The callback's deadline expired.
    Deadline,
}

/// A status reported by an agent, not proof of host callback execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum AgentToolStatus {
    /// The agent says the call is pending.
    Pending,
    /// The agent says the call is in progress.
    InProgress,
    /// The agent says the call completed.
    Completed,
    /// The agent says the call failed.
    Failed,
    /// An unrecognized status was reported.
    Unknown,
}

/// A chunk of user-visible text from the agent.
#[derive(Clone, Debug)]
pub struct MessageProgress {
    pub(crate) text: String,
}
impl MessageProgress {
    /// The text in this chunk. Chunks need not be complete words or lines.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// An agent tool announcement or partial update.
#[derive(Clone, Debug)]
pub struct AgentToolProgress {
    pub(crate) id: String,
    pub(crate) title: Option<String>,
    pub(crate) status: Option<AgentToolStatus>,
}
impl AgentToolProgress {
    /// The agent's call ID, scoped to this execution.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// A display title, when this update includes one.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }
    /// The status, when this update includes one.
    pub fn status(&self) -> Option<AgentToolStatus> {
        self.status
    }
}

/// A local application tool invocation, including input and output validation.
#[derive(Clone, Debug)]
pub struct ToolProgress {
    pub(crate) id: String,
    pub(crate) name: String,
}
impl ToolProgress {
    /// An opaque local invocation ID; it need not match an agent tool ID.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// The name registered by the application.
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// The observed outcome of a local application tool invocation.
#[derive(Clone, Debug)]
pub struct ToolCompletion {
    pub(crate) tool: ToolProgress,
    pub(crate) outcome: ToolOutcome,
}
impl ToolCompletion {
    /// The local invocation ID, matching the corresponding start event.
    pub fn id(&self) -> &str {
        self.tool.id()
    }
    /// The name registered by the application.
    pub fn name(&self) -> &str {
        self.tool.name()
    }
    /// How the invocation ended; this does not undo its effects.
    pub fn outcome(&self) -> ToolOutcome {
        self.outcome
    }
}

/// A rejected result submission, without the submitted value.
#[derive(Clone, Debug)]
pub struct ResultRejection {
    pub(crate) attempt: usize,
    pub(crate) reason: &'static str,
}
impl ResultRejection {
    /// The one-based submission number, including duplicates.
    pub fn attempt(&self) -> usize {
        self.attempt
    }
    /// A short description that excludes the submitted value.
    pub fn reason(&self) -> &str {
        self.reason
    }
}

/// A result submission passed validation; turn completion is still pending.
#[derive(Clone, Debug)]
pub struct ResultAcceptance {
    pub(crate) attempt: usize,
}
impl ResultAcceptance {
    /// The one-based submission number that supplied the accepted candidate.
    pub fn attempt(&self) -> usize {
        self.attempt
    }
}

/// The number of progress updates omitted since the previous loss report.
#[derive(Clone, Debug)]
pub struct DroppedEvents {
    pub(crate) count: u64,
}
impl DroppedEvents {
    /// The omitted event count, saturating at u64::MAX.
    pub fn count(&self) -> u64 {
        self.count
    }
}

/// A plan with a borrowed progress callback attached by [Plan::observe].
///
/// Finish building the plan before attaching the observer, then call [run](Self::run)
/// or [run_in](Self::run_in). Observed plans cannot be cloned. Clone an unobserved
/// base plan first when you want to reuse its instructions.
///
/// The observer runs synchronously on the task polling execution and may borrow
/// non-Send state. Keep it short: blocking it also delays tools and timeouts. It
/// returns (), so logging failures can be stored in a separate borrowed variable.
/// Panics propagate normally and initiate connection cleanup during unwinding.
/// No final observer callback is guaranteed when execution is dropped.
///
/// ```no_run
/// use thinkwell::{Agent, Plan};
/// async fn example(agent: &Agent) -> thinkwell::Result<()> {
///     let mut updates = 0;
///     let answer = Plan::<String>::new().text("Suggest a herb.")
///         .observe(|_| updates += 1).run(agent).await?;
///     println!("{answer} ({updates} updates)");
///     Ok(())
/// }
/// ```
///
/// The callback's captures are released when the owned execution is consumed or
/// dropped. Drop retained pinned futures before using their captures again.
///
/// ```compile_fail,E0503
/// use thinkwell::{Agent, Plan};
/// async fn example(agent: &Agent) {
///     let mut count = 0;
///     let work = Plan::<String>::new().observe(|_| count += 1).run(agent);
///     count += 1;
///     let _ = work.await;
/// }
/// ```
pub struct ObservedPlan<'a, T, S, F> {
    pub(crate) plan: Plan<'a, T, S>,
    pub(crate) observer: F,
}
impl<T: DeserializeOwned + JsonSchema, S, F: FnMut(&ProgressEvent)> ObservedPlan<'_, T, S, F> {
    /// Run one turn with fresh history and return its typed answer.
    ///
    /// Uses [Plan::run]'s setup, validation and cleanup rules. Observation begins
    /// when polled; an unpolled future sends nothing. Setup may produce no updates.
    pub async fn run(mut self, agent: &Agent) -> Result<T> {
        let sink = Sink::new();
        let _guard = agent.observer_guard(sink.clone());
        self.plan.progress = Some(sink.clone());
        drive(self.plan.run(agent), self.observer, sink).await
    }
    /// Observe a turn in an existing conversation, keeping its history.
    ///
    /// ```no_run
    /// use thinkwell::{Plan, Session};
    /// async fn example(session: &mut Session<'_>) -> thinkwell::Result<()> {
    ///     let mut updates = 0;
    ///     let answer = Plan::<String>::new().text("Explain the previous answer.")
    ///         .observe(|_| updates += 1).run_in(session).await?;
    ///     println!("{answer} ({updates} updates)");
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Uses [Plan::run_in]'s validation and invalidation rules. Earlier session
    /// setup is outside this observer's lifetime. The callback receives progress
    /// only while this execution is polled; the transport never invokes it.
    pub async fn run_in(mut self, session: &mut Session<'_>) -> Result<T> {
        let sink = Sink::new();
        let _guard = session.observer_guard(sink.clone());
        self.plan.progress = Some(sink.clone());
        drive(self.plan.run_in(session), self.observer, sink).await
    }
}

const MAX_EVENTS: usize = 128;
const MAX_BYTES: usize = 64 * 1024;
#[derive(Default)]
struct Buffer {
    queue: VecDeque<(ProgressEvent, usize)>,
    bytes: usize,
    dropped: u64,
    closed: bool,
    hidden: std::collections::HashSet<String>,
    agent_tools_disabled: bool,
}
#[derive(Clone)]
pub(crate) struct Sink(Arc<(Mutex<Buffer>, Notify)>);
impl Sink {
    fn new() -> Self {
        Self(Arc::new((Mutex::new(Buffer::default()), Notify::new())))
    }
    pub(crate) fn push(&self, event: ProgressEvent) {
        let bytes = event.bytes();
        let mut buffer = self.0.0.lock().unwrap();
        if buffer.closed {
            return;
        }
        if buffer.queue.len() >= MAX_EVENTS || bytes > MAX_BYTES - buffer.bytes {
            buffer.dropped = buffer.dropped.saturating_add(1);
        } else {
            buffer.bytes += bytes;
            buffer.queue.push_back((event, bytes));
        }
        drop(buffer);
        self.0.1.notify_one();
    }
    fn pop(&self) -> Option<ProgressEvent> {
        let mut buffer = self.0.0.lock().unwrap();
        if let Some((event, bytes)) = buffer.queue.pop_front() {
            buffer.bytes -= bytes;
            return Some(event);
        }
        if buffer.dropped > 0 {
            let count = std::mem::take(&mut buffer.dropped);
            return Some(ProgressEvent::EventsDropped(DroppedEvents { count }));
        }
        None
    }
    pub(crate) fn close(&self) {
        self.0.0.lock().unwrap().closed = true;
    }
    async fn next(&self) -> ProgressEvent {
        loop {
            let notified = self.0.1.notified();
            if let Some(event) = self.pop() {
                return event;
            }
            notified.await;
        }
    }
}
impl ProgressEvent {
    fn bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + match self {
                Self::Message(v) => v.text.len(),
                Self::AgentTool(v) => v.id.len() + v.title.as_ref().map_or(0, String::len),
                Self::ToolStarted(v) => v.id.len() + v.name.len(),
                Self::ToolFinished(v) => v.tool.id.len() + v.tool.name.len(),
                _ => 0,
            }
    }
}
async fn drive<T>(
    work: impl Future<Output = Result<T>>,
    mut observe: impl FnMut(&ProgressEvent),
    sink: Sink,
) -> Result<T> {
    tokio::pin!(work);
    let mut batch = 0;
    loop {
        tokio::select! {
            biased;
            result = &mut work => {
                sink.close();
                while let Some(event) = sink.pop() { observe(&event); }
                return result;
            }
            event = sink.next() => observe(&event),
        }
        batch += 1;
        if batch == 16 {
            batch = 0;
            tokio::task::yield_now().await;
        }
    }
}

impl Sink {
    pub(crate) fn agent_update(&self, update: &agent_client_protocol::v1::schema::SessionUpdate) {
        use agent_client_protocol::v1::schema::{ContentBlock, SessionUpdate, ToolCallStatus};
        if self.0.0.lock().unwrap().closed {
            return;
        }
        let status = |status| match status {
            ToolCallStatus::Pending => AgentToolStatus::Pending,
            ToolCallStatus::InProgress => AgentToolStatus::InProgress,
            ToolCallStatus::Completed => AgentToolStatus::Completed,
            ToolCallStatus::Failed => AgentToolStatus::Failed,
            _ => AgentToolStatus::Unknown,
        };
        let event = match update {
            SessionUpdate::AgentMessageChunk(chunk) => match &chunk.content {
                ContentBlock::Text(text) => ProgressEvent::Message(MessageProgress {
                    text: text.text.clone(),
                }),
                _ => return,
            },
            SessionUpdate::ToolCall(call) => {
                if internal_tool(call.name.as_deref(), Some(&call.title)) {
                    self.hide(call.tool_call_id.0.as_ref());
                    return;
                }
                ProgressEvent::AgentTool(AgentToolProgress {
                    id: call.tool_call_id.to_string(),
                    title: Some(call.title.clone()),
                    status: Some(status(call.status)),
                })
            }
            SessionUpdate::ToolCallUpdate(call) => {
                if internal_tool(call.fields.name.as_deref(), call.fields.title.as_deref()) {
                    self.hide(call.tool_call_id.0.as_ref());
                    return;
                }
                ProgressEvent::AgentTool(AgentToolProgress {
                    id: call.tool_call_id.to_string(),
                    title: call.fields.title.clone(),
                    status: call.fields.status.map(status),
                })
            }
            _ => return,
        };
        if let ProgressEvent::AgentTool(tool) = &event {
            let buffer = self.0.0.lock().unwrap();
            if buffer.hidden.contains(&tool.id) {
                return;
            }
            if buffer.agent_tools_disabled {
                drop(buffer);
                self.omit();
                return;
            }
        }
        self.push(event);
    }
    fn hide(&self, id: &str) {
        let mut buffer = self.0.0.lock().unwrap();
        if buffer.closed {
            return;
        }
        if (buffer.hidden.len() < MAX_EVENTS || buffer.hidden.contains(id)) && id.len() <= 512 {
            buffer.hidden.insert(id.to_owned());
        } else {
            buffer.agent_tools_disabled = true;
            drop(buffer);
            self.omit();
        }
    }
    fn omit(&self) {
        let mut buffer = self.0.0.lock().unwrap();
        if !buffer.closed {
            buffer.dropped = buffer.dropped.saturating_add(1);
        }
        drop(buffer);
        self.0.1.notify_one();
    }
}

fn internal_tool(name: Option<&str>, title: Option<&str>) -> bool {
    [name, title].into_iter().flatten().any(|s| {
        matches!(
            s,
            "mcp__thinkwell__return_result" | "mcp.thinkwell.return_result"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    /// anchorbook: A-progress-observation#behavior
    #[tokio::test]
    async fn bounded_queue_drains_tail_and_reports_loss() {
        let sink = Sink::new();
        for _ in 0..MAX_EVENTS + 7 {
            sink.push(ProgressEvent::ResultAccepted(ResultAcceptance {
                attempt: 1,
            }));
        }
        let mut events = Vec::new();
        assert_eq!(
            drive(async { Ok(42) }, |e| events.push(e.clone()), sink.clone())
                .await
                .unwrap(),
            42
        );
        assert_eq!(events.len(), MAX_EVENTS + 1);
        assert!(matches!(events.last(),Some(ProgressEvent::EventsDropped(d)) if d.count()==7));
        sink.push(ProgressEvent::ResultAccepted(ResultAcceptance {
            attempt: 1,
        }));
        assert!(sink.pop().is_none());
    }
    /// anchorbook: A-progress-observation#behavior
    #[test]
    fn byte_budget_counts_oversized_events_and_revokes_producers() {
        let sink = Sink::new();
        sink.push(ProgressEvent::Message(MessageProgress {
            text: "x".repeat(MAX_BYTES),
        }));
        assert!(matches!(sink.pop(),Some(ProgressEvent::EventsDropped(d)) if d.count()==1));
        for _ in 0..100 {
            sink.push(ProgressEvent::Message(MessageProgress {
                text: "x".repeat(1024),
            }));
        }
        let buffer = sink.0.0.lock().unwrap();
        assert!(buffer.bytes <= MAX_BYTES);
        assert!(buffer.queue.len() < MAX_EVENTS);
        assert!(buffer.dropped > 0);
        drop(buffer);
        sink.close();
        while sink.pop().is_some() {}
        sink.push(ProgressEvent::ResultAccepted(ResultAcceptance {
            attempt: 1,
        }));
        assert!(sink.pop().is_none());
    }
    /// anchorbook: A-progress-observation#behavior
    #[tokio::test]
    async fn observer_runs_while_work_waits_and_preserves_retained_order() {
        let sink = Sink::new();
        let (reply, response) = tokio::sync::oneshot::channel();
        let mut reply = Some(reply);
        let mut messages = Vec::new();
        let work = async {
            for text in ["first", "second"] {
                sink.push(ProgressEvent::Message(MessageProgress {
                    text: text.into(),
                }));
            }
            response.await.unwrap();
            Ok(())
        };
        tokio::time::timeout(
            std::time::Duration::from_secs(1),
            drive(
                work,
                |event| {
                    if let ProgressEvent::Message(message) = event {
                        messages.push(message.text().to_owned());
                        if messages.len() == 2 {
                            reply.take().unwrap().send(()).unwrap();
                        }
                    }
                },
                sink.clone(),
            ),
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(messages, ["first", "second"]);
    }
    /// anchorbook: A-progress-observation#behavior
    #[test]
    fn panic_while_draining_completed_work_closes_agent() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let dir = tempfile::tempdir().unwrap();
        let agent = rt
            .block_on(Agent::connect(crate::AgentConfig::new("python3").args([
                format!("{}/tests/peer.py", env!("CARGO_MANIFEST_DIR")),
                "correct".into(),
                dir.path().join("pid").display().to_string(),
            ])))
            .unwrap();
        let sink = Sink::new();
        sink.push(ProgressEvent::ResultAccepted(ResultAcceptance {
            attempt: 1,
        }));
        let caught = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = agent.observer_guard(sink.clone());
            // The ready result branch wins; the observer panics in tail delivery.
            rt.block_on(drive(async { Ok(()) }, |_| panic!("tail observer"), sink))
        }));
        assert!(caught.is_err());
        let error = rt.block_on(Plan::<String>::new().run(&agent)).unwrap_err();
        assert_eq!(error.kind, crate::ErrorKind::Closed);
        rt.block_on(agent.shutdown()).unwrap();
    }
    /// anchorbook: A-progress-observation#behavior
    #[test]
    fn internal_tool_updates_and_filter_metadata_are_bounded() {
        let sink = Sink::new();
        for update in [
            serde_json::json!({"sessionUpdate":"tool_call_update","toolCallId":"internal","title":"mcp.thinkwell.return_result"}),
            serde_json::json!({"sessionUpdate":"tool_call_update","toolCallId":"internal","status":"completed"}),
        ] {
            sink.agent_update(&serde_json::from_value(update).unwrap());
        }
        assert!(sink.pop().is_none());
        for i in 0..MAX_EVENTS {
            sink.hide(&format!("id-{i}"));
        }
        assert_eq!(sink.0.0.lock().unwrap().hidden.len(), MAX_EVENTS);
        assert!(sink.0.0.lock().unwrap().agent_tools_disabled);
        assert!(matches!(sink.pop(), Some(ProgressEvent::EventsDropped(_))));
        sink.push(ProgressEvent::ResultAccepted(ResultAcceptance {
            attempt: 1,
        }));
        assert!(matches!(sink.pop(), Some(ProgressEvent::ResultAccepted(_))));
    }
}
