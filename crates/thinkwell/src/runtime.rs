use crate::{
    AgentConfig, Error, ErrorKind, Result, Tool,
    api::{Descriptor, Plan},
    schema,
};
use agent_client_protocol::{
    AcpAgent, AcpAgentConfig, Client, schema::ProtocolVersion, v1::schema::*,
};
use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};

fn err(kind: ErrorKind, message: impl Into<String>) -> Error {
    Error::new(kind, message)
}
fn transport(e: impl std::fmt::Display) -> Error {
    err(ErrorKind::Transport, e.to_string())
}
/// A non-owning observer of connection cleanup completion.
///
/// Cloning this handle does not keep [Agent] alive or prevent shutdown. It exposes
/// a snapshot, not a subscription or an async join. An absent outcome does not prove
/// cleanup succeeded: runtime teardown can prevent the driver from publishing one.
/// A default-constructed handle is unattached and always has no outcome.
///
/// # Example
///
/// ~~~no_run
/// use thinkwell::Claude;
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let diagnostics = agent.diagnostics();
/// agent.shutdown().await?;
/// assert_eq!(diagnostics.outcome(), Some(Ok(())));
/// # Ok(())
/// # }
/// ~~~
///
/// For destructor-initiated cleanup, keep the runtime running and inspect
/// [outcome](Self::outcome) later. Use [Agent::shutdown] when confirmation is required.
#[derive(Clone, Default)]
pub struct Diagnostics(Arc<Mutex<Option<Result<()>>>>);
impl Diagnostics {
    /// Return the cleanup result recorded so far, cloning it.
    ///
    /// `None` means no result has been published, not success. `Some(Err(_))` retains
    /// a cleanup failure even when the Agent was dropped without awaited shutdown.
    /// See [Diagnostics] for a confirmation example.
    pub fn outcome(&self) -> Option<Result<()>> {
        self.0.lock().unwrap().clone()
    }
}
struct Control {
    stop: Mutex<Option<oneshot::Sender<()>>>,
    closed: AtomicBool,
    busy: AtomicBool,
}
impl Control {
    fn close(&self) {
        self.closed.store(true, Ordering::SeqCst);
        self.stop.lock().unwrap().take();
    }
}
struct Call {
    name: String,
    args: Value,
    reply: oneshot::Sender<Result<Value>>,
}
struct Route {
    turn: String,
    queue: mpsc::Sender<Call>,
    timeout: Duration,
}
struct Http {
    token: String,
    tools: Vec<Descriptor>,
    route: Mutex<Option<Route>>,
    active: AtomicBool,
}
struct HttpTask {
    http: Arc<Http>,
    stop: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    join: tokio::task::JoinHandle<std::io::Result<()>>,
}
impl Drop for HttpTask {
    fn drop(&mut self) {
        self.http.active.store(false, Ordering::SeqCst);
        self.http.route.lock().unwrap().take();
        self.stop.lock().unwrap().take();
        self.join.abort();
    }
}
struct Open {
    http: Arc<Http>,
    url: String,
    model: Option<String>,
    reply: oneshot::Sender<Result<SessionId>>,
    server: HttpTask,
}
enum Command {
    Open(Open),
    Prompt {
        id: SessionId,
        text: String,
        reply: oneshot::Sender<Result<()>>,
    },
}
/// An owned ACP subprocess connection, reusable for sequential conversations.
///
/// Create it with [Agent::connect], [Claude::connect](crate::Claude::connect), or
/// the [Connect](crate::Connect) extension trait. Only one session can be open per
/// connection; use independent agents for concurrent sessions.
///
/// Sessions borrow this owner. A normally dropped session releases its reservation
/// so the agent can open a fresh conversation with new history and credentials.
/// A failed or abandoned started turn closes the connection; reconnect explicitly.
///
/// Drop signals the existing owned cleanup driver. It does not block or wait for
/// remote acknowledgment. The driver attempts graceful shutdown; process guards
/// provide forceful fallback if the driver/runtime is destroyed. For confirmation
/// and cleanup errors, use [shutdown](Self::shutdown) after dropping sessions.
///
/// # Example: reuse a connection for independent conversations
///
/// ~~~no_run
/// use thinkwell::{Claude, Plan};
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// {
///     let mut session = agent.session().open().await?;
///     let title = Plan::<String>::new().text("Suggest a title for a Rust workshop.")
///         .run_in(&mut session).await?;
///     println!("{title}");
/// } // Normal session drop releases the one-session reservation.
/// {
///     let mut fresh = agent.session().open().await?;
///     let title = Plan::<String>::new().text("Suggest a title for a cooking class.")
///         .run_in(&mut fresh).await?;
///     println!("{title}"); // Fresh conversation, not the previous session's history.
/// }
/// # Ok(())
/// # }
/// ~~~
pub struct Agent {
    control: Arc<Control>,
    commands: mpsc::Sender<Command>,
    done: Option<oneshot::Receiver<Result<()>>>,
    diagnostics: Diagnostics,
    config: AgentConfig,
}
impl Drop for Agent {
    fn drop(&mut self) {
        self.control.close();
    }
}
/// Configuration for opening a fixed-catalog conversation on an [Agent].
///
/// Create with [Agent::session], declare contracts with [tool](Self::tool),
/// optionally select a [model](Self::model), then [open](Self::open).
/// Building configuration sends no prompt and acquires no session reservation.
///
/// # Example
///
/// ~~~no_run
/// use thinkwell::{Claude, Tool};
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let echo = Tool::<String, String>::new("echo", "Echo a message");
/// let session = agent.session().tool(&echo).model("haiku").open().await?;
/// drop(session); // No prompts were sent.
/// # Ok(())
/// # }
/// ~~~
///
/// Tool implementations are supplied later through [Plan::bind](crate::Plan::bind).
/// The catalog stays fixed for the lifetime of the returned session.
pub struct SessionBuilder<'a> {
    agent: &'a Agent,
    tools: Vec<Descriptor>,
    error: Option<Error>,
    model: Option<String>,
}
/// A persistent conversation borrowing its owning [Agent].
///
/// Create through [Agent::session] and [SessionBuilder::open]. Each
/// [Plan::run_in](crate::Plan::run_in) exclusively borrows the session for one turn.
/// Conversation history persists, but callback captures are scoped to each execution.
/// See [Plan](crate::Plan) for a complete borrowed-tool example.
///
/// # Example: inspect usability after failure
///
/// ~~~no_run
/// use thinkwell::{Claude, Plan};
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let mut session = agent.session().open().await?;
/// match Plan::<String>::new().text("Summarize ACP.").run_in(&mut session).await {
///     Ok(answer) => println!("{answer}"),
///     Err(error) => {
///         eprintln!("{error}");
///         if !session.usable() {
///             // Start a new connection explicitly; this session cannot recover.
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ~~~
///
/// Normal Drop revokes tool routes and releases the agent's session reservation.
/// Dropping a started execution or receiving a fatal turn error permanently
/// invalidates both session and connection. Preflight errors leave them unchanged.
/// Cancellation never rolls back already performed application effects.
pub struct Session<'a> {
    agent: &'a Agent,
    id: SessionId,
    http: Arc<Http>,
    stop: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    usable: bool,
}
impl Drop for Session<'_> {
    fn drop(&mut self) {
        self.http.active.store(false, Ordering::SeqCst);
        self.http.route.lock().unwrap().take();
        self.stop.lock().unwrap().take();
        self.agent.control.busy.store(false, Ordering::SeqCst);
    }
}
struct Reservation {
    control: Arc<Control>,
    keep: bool,
    started: bool,
}
impl Drop for Reservation {
    fn drop(&mut self) {
        if !self.keep {
            self.control.busy.store(false, Ordering::SeqCst);
            if self.started {
                self.control.close();
            }
        }
    }
}
struct TurnGuard {
    http: Arc<Http>,
    control: Arc<Control>,
    success: bool,
}
impl Drop for TurnGuard {
    fn drop(&mut self) {
        self.http.route.lock().unwrap().take();
        if !self.success {
            self.control.close();
        }
    }
}
impl SessionBuilder<'_> {
    /// Add a tool contract to this session's fixed catalog.
    ///
    /// The descriptor is cloned; the session does not borrow the Tool or own its
    /// implementation. Bind it on every turn with [Plan::bind](crate::Plan::bind).
    /// Duplicate names and errors retained by Tool construction are reported by
    /// [open](Self::open). See [SessionBuilder] for an example.
    pub fn tool<I, O>(mut self, tool: &Tool<I, O>) -> Self {
        match &tool.descriptor {
            Ok(d) if self.tools.iter().any(|t| t.name == d.name) => {
                self.error = Some(err(ErrorKind::Binding, "duplicate tool name"))
            }
            Ok(d) => self.tools.push(d.clone()),
            Err(e) => self.error = Some(e.clone()),
        }
        self
    }
    /// Select an agent-provided model identifier when opening the session.
    ///
    /// Repeated calls replace the previous selection. Omitting this method preserves
    /// the agent's default. Identifiers and availability depend on the agent; Thinkwell
    /// does not translate provider names or enumerate models. For the tested Claude
    /// adapter, `haiku` is one selector. See [SessionBuilder] for usage.
    ///
    /// An empty selector is a local configuration error. Failure of the model-selection
    /// request is reported as [ErrorKind::UnsupportedModel].
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}
impl<'a> SessionBuilder<'a> {
    /// Open a new remote conversation and its authenticated loopback tool endpoint.
    ///
    /// Uses the Agent's setup timeout. The endpoint/catalog is created before the
    /// remote session request; no model prompt is sent. Session working directories
    /// are managed internally, not a caller-specified project directory.
    ///
    /// # Errors and cancellation
    ///
    /// Invalid descriptors, duplicate names, or an empty model selector fail locally.
    /// [ErrorKind::Busy] means another session is open; [ErrorKind::Closed] means the
    /// connection is unavailable. Remote setup/model errors and deadlines are returned
    /// with their category. Failure or cancellation after remote setup starts closes
    /// the connection to avoid ambiguous ownership of partially acquired resources.
    ///
    /// Requires the same enabled Tokio runtime as [Agent::connect]. See [SessionBuilder]
    /// for a tool/model example, or [Agent] for sequential conversations.
    pub async fn open(self) -> Result<Session<'a>> {
        if let Some(error) = self.error {
            return Err(error);
        }
        if self.model.as_ref().is_some_and(|s| s.is_empty()) {
            return Err(err(ErrorKind::Configuration, "empty model selector"));
        }
        let control = &self.agent.control;
        if control.closed.load(Ordering::SeqCst) {
            return Err(err(ErrorKind::Closed, "agent is closed"));
        }
        if control
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(err(
                ErrorKind::Busy,
                "one session per connection; drop the existing session first",
            ));
        }
        let mut reservation = Reservation {
            control: control.clone(),
            keep: false,
            started: false,
        };
        let http = Arc::new(Http {
            token: format!("Bearer {}", uuid::Uuid::new_v4()),
            tools: self.tools,
            route: Mutex::new(None),
            active: AtomicBool::new(true),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(transport)?;
        let address = listener.local_addr().map_err(transport)?;
        let router = Router::new()
            .route("/mcp", post(handle))
            .layer(DefaultBodyLimit::max(256 * 1024))
            .with_state(http.clone());
        let (stop, stopped) = oneshot::channel();
        let join = tokio::spawn(async move {
            axum::serve(listener, router)
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
        });
        let stop = Arc::new(Mutex::new(Some(stop)));
        let server = HttpTask {
            http: http.clone(),
            stop: stop.clone(),
            join,
        };
        let (reply, response) = oneshot::channel();
        reservation.started = true;
        self.agent
            .commands
            .send(Command::Open(Open {
                http: http.clone(),
                url: format!("http://{address}/mcp"),
                model: self.model,
                reply,
                server,
            }))
            .await
            .map_err(|_| err(ErrorKind::Closed, "driver unavailable"))?;
        let id = tokio::time::timeout(self.agent.config.setup_timeout, response)
            .await
            .map_err(|_| err(ErrorKind::Deadline, "session setup deadline"))?
            .map_err(|_| err(ErrorKind::Closed, "driver disappeared during session setup"))??;
        reservation.keep = true;
        Ok(Session {
            agent: self.agent,
            id,
            http,
            stop,
            usable: true,
        })
    }
}
impl Agent {
    /// Start configuring a fresh conversation with no tools or model override.
    ///
    /// Does not contact the agent. Call [SessionBuilder::open] to acquire the
    /// one-session reservation and create remote history. See [Agent] for reuse.
    pub fn session(&self) -> SessionBuilder<'_> {
        SessionBuilder {
            agent: self,
            tools: vec![],
            error: None,
            model: None,
        }
    }
    /// Obtain an observer that survives dropping this agent without keeping it alive.
    ///
    /// See [Diagnostics] for an example and the meaning of an absent cleanup result.
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics.clone()
    }
    /// Consume the agent, request shutdown, and await a bounded cleanup outcome.
    ///
    /// All sessions must first be dropped; the borrow checker enforces this. Uses
    /// [AgentConfig::shutdown_timeout]. A deadline or cleanup error means successful
    /// cleanup was not confirmed; dropping this future still leaves shutdown requested
    /// once it has been polled. This is optional: ordinary Drop also requests cleanup.
    ///
    /// ~~~no_run
    /// use thinkwell::{Claude, Plan};
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Claude::connect().await?;
    /// let mut session = agent.session().open().await?;
    /// let answer = Plan::<String>::new().text("Say hello.").run_in(&mut session).await?;
    /// println!("{answer}");
    /// drop(session);
    /// agent.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ~~~
    pub async fn shutdown(mut self) -> Result<()> {
        self.control.close();
        tokio::time::timeout(self.config.shutdown_timeout, self.done.take().unwrap())
            .await
            .map_err(|_| err(ErrorKind::Deadline, "shutdown deadline"))?
            .map_err(|_| err(ErrorKind::Cleanup, "driver disappeared"))?
    }
    /// Launch an ACP subprocess and initialize the protocol connection.
    ///
    /// Accepts [AgentConfig] or any preset implementing `Into<AgentConfig>`.
    /// No session or prompt is created yet. Install/authenticate the agent externally.
    /// The initial supported environment is macOS/Linux with Tokio; the later session
    /// path requires HTTP MCP support from the chosen agent.
    ///
    /// # Example
    ///
    /// ~~~no_run
    /// use thinkwell::{Agent, AgentConfig};
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Agent::connect(
    ///     AgentConfig::new("my-acp-agent").args(["--transport", "stdio"])
    /// ).await?;
    /// let session = agent.session().open().await?;
    /// drop(session);
    /// agent.shutdown().await?;
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// # Runtime, errors, and cancellation
    ///
    /// Call inside a Tokio runtime with I/O and time enabled. Missing runtime,
    /// empty commands, or invalid time bounds produce configuration errors. Disabled
    /// Tokio I/O/time facilities can panic when those facilities are used. Process or
    /// initialization failures produce transport/setup errors; initialization is bounded
    /// by [AgentConfig::setup_timeout].
    ///
    /// Dropping a partially completed connection future signals owned cleanup.
    /// The driver owns transport/process resources, never borrowed application callbacks.
    /// Successful connection alone does not establish that a custom agent supports
    /// the HTTP MCP or model-selection capabilities needed by later operations.
    pub async fn connect(config: impl Into<AgentConfig>) -> Result<Self> {
        let config = config.into();
        if config.command.is_empty()
            || config.setup_timeout.is_zero()
            || config.shutdown_timeout.is_zero()
            || std::time::Instant::now()
                .checked_add(config.setup_timeout)
                .is_none()
            || std::time::Instant::now()
                .checked_add(config.shutdown_timeout)
                .is_none()
        {
            return Err(err(
                ErrorKind::Configuration,
                "command and nonzero setup/shutdown bounds required",
            ));
        }
        tokio::runtime::Handle::try_current()
            .map_err(|_| err(ErrorKind::Configuration, "Tokio runtime required"))?;
        let (stop, mut stopping) = oneshot::channel();
        let control = Arc::new(Control {
            stop: Mutex::new(Some(stop)),
            closed: AtomicBool::new(false),
            busy: AtomicBool::new(false),
        });
        let (commands, mut requests) = mpsc::channel(1);
        let (done, completion) = oneshot::channel();
        let (ready, initialized) = oneshot::channel();
        let diagnostics = Diagnostics::default();
        let report = diagnostics.clone();
        let driver_control = control.clone();
        let setup_timeout = config.setup_timeout;
        let shutdown_timeout = config.shutdown_timeout;
        let launch = AcpAgentConfig::new(config.command.clone())
            .args(config.args.clone())
            .envs(config.env.clone());
        tokio::spawn(async move {
            let allowed = Arc::new(Mutex::new(None::<Arc<Http>>));
            let permissions = allowed.clone();
            let servers = Arc::new(Mutex::new(Vec::<HttpTask>::new()));
            let driver_servers = servers.clone();
            let client=Client.v1()
                .on_receive_request(async move |request:RequestPermissionRequest,responder,_| {
                    let name=request.tool_call.fields.name.as_deref().or(request.tool_call.fields.title.as_deref());
                    let allowed=permissions.lock().unwrap().as_ref().is_some_and(|http| {
                        http.active.load(Ordering::SeqCst) && name.is_some_and(|name| {
                            name=="mcp__thinkwell__return_result" || http.tools.iter().any(|t|name==format!("mcp__thinkwell__{}",t.name))
                        })
                    });
                    let option=request.options.iter().find(|o|allowed && o.kind==PermissionOptionKind::AllowOnce);
                    let outcome=option.map_or(RequestPermissionOutcome::Cancelled,|o|RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(o.option_id.clone())));
                    responder.respond(RequestPermissionResponse::new(outcome))
                },agent_client_protocol::on_receive_request!())
                .connect_with(AcpAgent::new(launch),async move |cx| {
                    let initialize=cx.send_request(InitializeRequest::new(ProtocolVersion::V1)).block_task();
                    let init=tokio::select! {
                        biased;
                        _=&mut stopping=>return Ok(()),
                        r=tokio::time::timeout(setup_timeout,initialize)=>r.map_err(|_|err(ErrorKind::Deadline,"initialize deadline")).and_then(|r|r.map_err(transport)),
                    };
                    let _=ready.send(init.map(|_|()));
                    let directory=tempfile::tempdir().map_err(agent_client_protocol::Error::into_internal_error)?;
                    loop {
                        let command=tokio::select! {
                            biased;
                            _=&mut stopping=>break,
                            command=requests.recv()=>match command {Some(c)=>c,None=>break},
                        };
                        match command {
                            Command::Open(mut open)=>{
                                *allowed.lock().unwrap()=Some(open.http.clone());
                                driver_servers.lock().unwrap().push(open.server);
                                let operation=async {
                                    let s=cx.send_request(NewSessionRequest::new(directory.path().to_owned()).mcp_servers(vec![
                                        McpServer::Http(McpServerHttp::new("thinkwell",open.url).headers(vec![HttpHeader::new("Authorization",open.http.token.clone())]))
                                    ])).block_task().await.map_err(transport)?;
                                    if let Some(model)=open.model {
                                        let config:SetSessionConfigOptionRequest=serde_json::from_value(json!({"sessionId":s.session_id,"configId":"model","value":model})).map_err(transport)?;
                                        cx.send_request(config).block_task().await.map_err(|e|err(ErrorKind::UnsupportedModel,e.to_string()))?;
                                    }
                                    Ok(s.session_id)
                                };
                                let result=tokio::select! {
                                    biased;
                                    _=&mut stopping=>break,
                                    _=open.reply.closed()=>break,
                                    result=tokio::time::timeout(setup_timeout,operation)=>result.map_err(|_|err(ErrorKind::Deadline,"session setup deadline")).and_then(|r|r),
                                };
                                let _=open.reply.send(result);
                            }
                            Command::Prompt{id,text,mut reply}=>{
                                let prompt=cx.send_request(PromptRequest::new(id,vec![ContentBlock::Text(TextContent::new(text))])).block_task();
                                let result=tokio::select! {
                                    biased;
                                    _=&mut stopping=>break,
                                    _=reply.closed()=>break,
                                    r=prompt=>r.map_err(transport).and_then(|r| match r.stop_reason {
                                        StopReason::EndTurn=>Ok(()),
                                        StopReason::Cancelled=>Err(err(ErrorKind::Cancelled,"agent cancelled turn")),
                                        reason=>Err(err(ErrorKind::RemoteStop,format!("{reason:?}"))),
                                    }),
                                };
                                let _=reply.send(result);
                            }
                        }
                    }
                    Ok::<_,agent_client_protocol::Error>(())
                });
            let outcome = client
                .await
                .map_err(|e| err(ErrorKind::Cleanup, e.to_string()));
            driver_control.closed.store(true, Ordering::SeqCst);
            let tasks = std::mem::take(&mut *servers.lock().unwrap());
            let cleanup = async {
                for mut task in tasks {
                    task.http.active.store(false, Ordering::SeqCst);
                    task.http.route.lock().unwrap().take();
                    task.stop.lock().unwrap().take();
                    match (&mut task.join).await {
                        Ok(Ok(())) => {}
                        other => {
                            return Err(err(
                                ErrorKind::Cleanup,
                                format!("HTTP shutdown: {other:?}"),
                            ));
                        }
                    }
                }
                Ok(())
            };
            let cleanup = tokio::time::timeout(shutdown_timeout, cleanup)
                .await
                .map_err(|_| err(ErrorKind::Cleanup, "HTTP shutdown deadline"))
                .and_then(|r| r);
            let outcome = outcome.and(cleanup);
            *report.0.lock().unwrap() = Some(outcome.clone());
            let _ = done.send(outcome);
        });
        let agent = Self {
            control,
            commands,
            done: Some(completion),
            diagnostics,
            config,
        };
        initialized
            .await
            .map_err(|_| err(ErrorKind::Transport, "agent failed during initialization"))??;
        Ok(agent)
    }
}
impl Session<'_> {
    /// Whether this session and its connection are currently available for a turn.
    ///
    /// This is a local snapshot, not a remote health check. It is false during a started
    /// turn or after connection closure. Preflight failures leave it true; failed or
    /// abandoned started turns make it permanently false. A true snapshot cannot guarantee
    /// the next request will succeed. See [Session] for error handling.
    pub fn usable(&self) -> bool {
        self.usable && !self.agent.control.closed.load(Ordering::SeqCst)
    }
    pub(crate) async fn run<T: DeserializeOwned + JsonSchema, S>(
        &mut self,
        mut plan: Plan<'_, T, S>,
    ) -> Result<T> {
        if !self.usable() {
            return Err(err(ErrorKind::Closed, "session or agent is closed"));
        }
        if let Some(e) = plan.error {
            return Err(e);
        }
        plan.options.validate()?;
        let schema = schema::contract::<T>()?;
        if plan.bindings.len() != self.http.tools.len() {
            return Err(err(
                ErrorKind::Binding,
                "bind every declared tool exactly once",
            ));
        }
        let mut names = std::collections::HashSet::new();
        for binding in &plan.bindings {
            if !names.insert(&binding.descriptor.name)
                || !self.http.tools.contains(&binding.descriptor)
            {
                return Err(err(
                    ErrorKind::Binding,
                    "duplicate or mismatched tool descriptor",
                ));
            }
        }
        self.usable = false;
        let turn = uuid::Uuid::new_v4().to_string();
        let (queue, mut calls) = mpsc::channel(1);
        *self.http.route.lock().unwrap() = Some(Route {
            turn: turn.clone(),
            queue,
            timeout: plan.options.turn_timeout,
        });
        let mut guard = TurnGuard {
            http: self.http.clone(),
            control: self.agent.control.clone(),
            success: false,
        };
        let text = format!(
            "{}\nUse only thinkwell MCP tools. Every application call requires {{\"turn\":\"{turn}\",\"input\":APPLICATION_ARGUMENTS}}. Submit the final answer using return_result with turn {turn} and value matching JSON Schema {schema}. After acceptance end the turn. Rejected values may be corrected within the configured budget.",
            plan.text
        );
        let (reply, mut terminal) = oneshot::channel();
        self.agent
            .commands
            .send(Command::Prompt {
                id: self.id.clone(),
                text,
                reply,
            })
            .await
            .map_err(|_| err(ErrorKind::Closed, "driver unavailable"))?;
        let mut candidate = None;
        let mut invalid = 0;
        let mut count = 0;
        let work = async {
            loop {
                tokio::select! {
                    biased;
                    r=&mut terminal=>{
                        r.map_err(|_|err(ErrorKind::Transport,"driver disappeared"))??;
                        return candidate.take().ok_or_else(||err(ErrorKind::MissingResult,"normal completion without valid result"));
                    }
                    Some(call)=calls.recv()=>{
                        let response=if call.name=="return_result" {
                            if candidate.is_some() {Err(err(ErrorKind::InvalidValue,"duplicate result"))}
                            else {
                                let result=call.args.get("value").ok_or_else(||err(ErrorKind::InvalidValue,"missing value"))
                                    .and_then(|value|schema::decode::<T>(&schema,value.clone()));
                                match result {
                                    Ok(value)=>{candidate=Some(value);Ok(json!({"accepted":true}))}
                                    Err(e)=>{invalid+=1;Err(e)}
                                }
                            }
                        } else {
                            count+=1;
                            if count>plan.options.max_tool_calls {
                                let e=err(ErrorKind::CallBudget,"application call budget exhausted");
                                let _=call.reply.send(Err(e.clone()));return Err(e);
                            }
                            let binding=plan.bindings.iter_mut().find(|b|b.descriptor.name==call.name).expect("route validates catalog");
                            let invocation=async {
                                let input=call.args.get("input").ok_or_else(||err(ErrorKind::InvalidValue,"missing input"))?;
                                binding.callback.call(input.clone()).await
                            };
                            tokio::select! {
                                biased;
                                _=&mut terminal=>return Err(err(ErrorKind::CallbackOverlap,"terminal arrived during callback")),
                                r=tokio::time::timeout(plan.options.callback_timeout,invocation)=>r.map_err(|_|err(ErrorKind::Deadline,"callback deadline")).and_then(|r|r),
                            }
                        };
                        let _=call.reply.send(response);
                        if invalid>=plan.options.max_invalid_results{return Err(err(ErrorKind::ResultBudget,"invalid result budget exhausted"));}
                    }
                }
            }
        };
        let result = tokio::time::timeout(plan.options.turn_timeout, work)
            .await
            .map_err(|_| err(ErrorKind::Deadline, "turn deadline"))
            .and_then(|r| r);
        guard.success = result.is_ok();
        self.usable = guard.success;
        result
    }
}

async fn handle(
    State(http): State<Arc<Http>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    if headers.contains_key("origin") {
        return StatusCode::FORBIDDEN.into_response();
    }
    if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some(http.token.as_str()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if !http.active.load(Ordering::SeqCst) {
        return StatusCode::GONE.into_response();
    }
    if headers
        .get("mcp-protocol-version")
        .is_some_and(|v| v != "2025-11-25")
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if request["jsonrpc"] != "2.0" {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if request["method"] == "notifications/initialized" {
        return StatusCode::ACCEPTED.into_response();
    }
    let id = request["id"].clone();
    if id.is_null() {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let result = match request["method"].as_str().unwrap_or("") {
        "initialize" => {
            json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"thinkwell","version":env!("CARGO_PKG_VERSION")}})
        }
        "ping" => json!({}),
        "tools/list" => {
            let mut tools=http.tools.iter().map(|t|json!({"name":t.name,"description":t.description,
                "inputSchema":{"type":"object","properties":{"turn":{"type":"string"},"input":t.input},"required":["turn","input"]},
                "outputSchema":{"type":"object","properties":{"value":t.output},"required":["value"]}})).collect::<Vec<_>>();
            tools.push(json!({"name":"return_result","description":"Submit final typed result","inputSchema":{"type":"object","properties":{"turn":{"type":"string"},"value":{}},"required":["turn","value"]}}));
            json!({"tools":tools})
        }
        "tools/call" => {
            let name = request["params"]["name"].as_str().unwrap_or("").to_owned();
            let args = request["params"]["arguments"].clone();
            let route = {
                let route = http.route.lock().unwrap();
                route
                    .as_ref()
                    .filter(|r| {
                        args["turn"].as_str() == Some(r.turn.as_str())
                            && (name == "return_result"
                                || http.tools.iter().any(|t| t.name == name))
                    })
                    .map(|r| (r.queue.clone(), r.timeout))
            };
            let result = match route {
                None => Err(err(
                    ErrorKind::InvalidValue,
                    "no active matching tool and turn",
                )),
                Some((queue, timeout)) => {
                    let (reply, response) = oneshot::channel();
                    // Turn lifetime closes the queue/reply. Bound unpolled/waiting HTTP clients as well.
                    tokio::time::timeout(timeout, async {
                        queue
                            .send(Call { name, args, reply })
                            .await
                            .map_err(|_| err(ErrorKind::Closed, "callback revoked"))?;
                        response
                            .await
                            .map_err(|_| err(ErrorKind::Closed, "callback revoked"))?
                    })
                    .await
                    .map_err(|_| err(ErrorKind::Deadline, "HTTP call deadline"))
                    .and_then(|r| r)
                }
            };
            match result {
                Ok(value) => {
                    json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":{"value":value},"isError":false})
                }
                Err(e) => json!({"content":[{"type":"text","text":e.to_string()}],"isError":true}),
            }
        }
        _ => {
            return Json(
                json!({"jsonrpc":"2.0","id":id,"error":{"code":-32601,"message":"unknown method"}}),
            )
            .into_response();
        }
    };
    Json(json!({"jsonrpc":"2.0","id":id,"result":result})).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn revoked_queue_never_reaches_replacement_callback() {
        tokio::time::timeout(Duration::from_secs(2),async {
            let http=Arc::new(Http {
                token:"Bearer fixture".into(),
                tools:vec![Descriptor{name:"add".into(),description:String::new(),input:json!({}),output:json!({})}],
                route:Mutex::new(None),active:AtomicBool::new(true),
            });
            let (old,old_receiver)=mpsc::channel(1);
            *http.route.lock().unwrap()=Some(Route{turn:"A".into(),queue:old.clone(),timeout:Duration::from_secs(1)});
            http.route.lock().unwrap().take();
            drop(old_receiver);
            let (new,mut receiver)=mpsc::channel(1);
            *http.route.lock().unwrap()=Some(Route{turn:"B".into(),queue:new,timeout:Duration::from_secs(1)});
            let (reply,_)=oneshot::channel();
            assert!(old.send(Call{name:"add".into(),args:json!({}),reply}).await.is_err());
            let mut headers=HeaderMap::new();
            headers.insert("authorization","Bearer fixture".parse().unwrap());
            let response=handle(State(http),headers,Json(json!({
                "jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"add","arguments":{"turn":"A","input":3}}
            }))).await;
            let response:Value=serde_json::from_slice(&to_bytes(response.into_body(),8192).await.unwrap()).unwrap();
            assert_eq!(response["result"]["isError"],true);
            assert!(receiver.try_recv().is_err());
        }).await.unwrap();
    }
}
