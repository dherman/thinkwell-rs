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
/// A way to check whether automatic cleanup has finished.
///
/// Most applications can simply let their [Agent] go out of scope. Use this handle
/// if you also want to inspect the cleanup result afterward. Keeping or cloning
/// it does not keep the agent running.
///
/// ~~~no_run
/// use thinkwell::agent::Claude;
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let diagnostics = agent.diagnostics();
/// drop(agent); // Starts cleanup without waiting for it.
///
/// // Elsewhere, while the Tokio runtime is still running:
/// match diagnostics.outcome() {
///     Some(Ok(())) => println!("Cleanup finished."),
///     Some(Err(error)) => eprintln!("Cleanup failed: {error}"),
///     None => println!("No cleanup result yet."),
/// }
/// # Ok(())
/// # }
/// ~~~
///
/// This handle only reports the result available when you check it. It does not
/// wait or notify you of changes. If you need to wait, use [Agent::shutdown].
/// If the runtime stops before recording a result, [outcome](Self::outcome) may
/// remain `None`. A handle created with `Default::default()` is not attached to
/// any agent and always returns `None`.
#[derive(Clone, Default)]
pub struct Diagnostics(Arc<Mutex<Option<Result<()>>>>);
impl Diagnostics {
    /// Read the cleanup result, if one has been recorded.
    ///
    /// `Some(Ok(()))` means cleanup succeeded; `Some(Err(error))` explains a failure.
    /// `None` means no result is available, not that cleanup succeeded. Each call
    /// returns a copy of the current result. See [Diagnostics] for an example.
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
/// A connection to an agent program, used to start conversations.
///
/// Begin with [Claude::connect](crate::agent::Claude::connect), or use [Agent::connect]
/// with your own configuration. Then call [session](Self::session) to open a
/// conversation. You can reuse a connection for separate conversations; each new
/// [Session] starts without the previous conversation's history.
///
/// # Start two independent conversations
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Plan};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// {
///     let mut session = agent.session().open().await?;
///     let title = Plan::<String>::new()
///         .text("Suggest a title for a Rust workshop.")
///         .run_in(&mut session).await?;
///     println!("{title}");
/// } // The first conversation ends here.
///
/// let mut fresh = agent.session().open().await?;
/// let title = Plan::<String>::new()
///     .text("Suggest a title for a cooking class.")
///     .run_in(&mut fresh).await?;
/// println!("{title}");
/// # Ok(())
/// # }
/// ~~~
///
/// Only one conversation can be open on a connection at a time. Use separate
/// agents if you need concurrent conversations. A session borrows its agent,
/// so Rust ensures the agent stays alive while you use the session.
///
/// # Cleanup and failures
///
/// Letting the agent go out of scope starts cleanup automatically. This does not
/// block your application while the agent stops. If you need to wait for cleanup
/// and check its result, use [shutdown](Self::shutdown).
///
/// A request that fails or is cancelled after starting closes the connection as
/// well as the session. Connect again to continue. Errors caught before sending
/// the request leave the connection available; see [Plan::run_in](crate::Plan::run_in).
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
/// Choose a conversation's tools and model before opening it.
///
/// Get a builder from [Agent::session], add any settings you need, then call
/// [open](Self::open). If you do not add settings, the conversation has no
/// application tools and uses the agent's default model.
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Plan};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let mut session = agent.session().model("haiku").open().await?;
/// let answer = Plan::<String>::new()
///     .text("Suggest three herbs for a windowsill garden.")
///     .run_in(&mut session).await?;
/// println!("{answer}");
/// # Ok(())
/// # }
/// ~~~
///
/// To offer Rust functions to the agent, register their descriptions with
/// [tool](Self::tool). See [Tool] for a complete example. The tools you choose here
/// stay available throughout the conversation; provide their functions with
/// [Plan::bind](crate::Plan::bind) on each request.
pub struct SessionBuilder<'a> {
    agent: &'a Agent,
    tools: Vec<Descriptor>,
    error: Option<Error>,
    model: Option<String>,
}
/// One conversation with an agent, including its earlier questions and answers.
///
/// Open a session with `agent.session().open().await?`. Run a [Plan](crate::Plan)
/// in it to ask a question, then run another plan in the same session to follow up.
/// Each request and answer is called a *turn*.
///
/// # Ask a follow-up question
///
/// ~~~no_run
/// use thinkwell::{agent::Claude, Plan};
///
/// # async fn example() -> thinkwell::Result<()> {
/// let agent = Claude::connect().await?;
/// let mut session = agent.session().open().await?;
///
/// let idea = Plan::<String>::new()
///     .text("Suggest a vegetable to grow on a sunny balcony.")
///     .run_in(&mut session).await?;
/// println!("Try: {idea}");
///
/// let steps = Plan::<Vec<String>>::new()
///     .text("Give me three steps to get started growing it.")
///     .run_in(&mut session).await?;
/// for step in steps {
///     println!("- {step}");
/// }
/// # Ok(())
/// # }
/// ~~~
///
/// The second request remembers the first answer, even though it asks for a
/// different Rust type. Only one request can run at a time in a session.
///
/// # Ending a conversation
///
/// Let the session go out of scope when you are done. Its [Agent] can then open a
/// fresh conversation. You do not need to explicitly finish a successful turn.
///
/// If a request fails or is cancelled after starting, both the session and its
/// connection close. Changes already made by tools remain. Errors caught before
/// sending a request leave the session available for a corrected plan. See
/// [usable](Self::usable) for an error-handling example and
/// [Plan::run_in](crate::Plan::run_in) for details.
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
    /// Make a tool available in this conversation.
    ///
    /// Pass a [Tool] describing the operation. When sending each request, also use
    /// [Plan::bind](crate::Plan::bind) to supply the Rust function that performs it.
    /// [Tool] has a complete word-counting example showing both steps.
    ///
    /// The builder copies the description; it does not borrow the Tool or store a
    /// function. Duplicate names and unsupported tool types are reported when you
    /// call [open](Self::open).
    pub fn tool<I, O>(mut self, tool: &Tool<I, O>) -> Self {
        self = self.descriptor(tool.descriptor.clone());
        self
    }
    pub(crate) fn descriptor(mut self, descriptor: Result<Descriptor>) -> Self {
        match descriptor {
            Ok(d) if self.tools.iter().any(|t| t.name == d.name) => {
                self.error = Some(err(ErrorKind::Binding, "duplicate tool name"))
            }
            Ok(d) => self.tools.push(d),
            Err(e) => self.error = Some(e),
        }
        self
    }
    /// Choose which model the agent should use for this conversation.
    ///
    /// For example, `agent.session().model("haiku").open().await?` selects Haiku
    /// with the tested Claude adapter. Leave this setting out to use the agent's
    /// default. See [SessionBuilder] for a complete example.
    ///
    /// Model names depend on the agent. Thinkwell passes the name through; it does
    /// not translate names or list available models. Calling this method again
    /// replaces the previous selection. An empty name is a configuration error;
    /// failure of the selection request produces [ErrorKind::UnsupportedModel].
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }
}
impl<'a> SessionBuilder<'a> {
    /// Open the conversation with the settings you have chosen.
    ///
    /// This prepares the agent to receive requests; it does not send a prompt. Next,
    /// use [Plan::run_in](crate::Plan::run_in) to ask a question. See [SessionBuilder]
    /// for a complete example.
    ///
    /// # Errors
    ///
    /// Only one session can be open per agent. Close the previous session before
    /// opening another, or this returns [ErrorKind::Busy]. A closed connection
    /// returns [ErrorKind::Closed]. Invalid tool descriptions, duplicate tool names,
    /// and an empty model name are rejected before contacting the agent.
    ///
    /// Setup uses [AgentConfig::setup_timeout]. If setup fails or is cancelled after
    /// contacting the agent, the connection closes; connect again before retrying.
    /// Model-selection failures return [ErrorKind::UnsupportedModel].
    ///
    /// # Runtime requirements
    ///
    /// Use the same Tokio runtime as [Agent::connect], with I/O and time enabled.
    /// Thinkwell manages the session's working directory internally; this method
    /// does not open your application's project directory for the agent.
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
    /// Start setting up a new conversation.
    ///
    /// Call `agent.session().open().await?` to use the defaults, or add tools and
    /// choose a model first. See [SessionBuilder] for those options. Creating the
    /// builder does not contact the agent; [open](SessionBuilder::open) does.
    pub fn session(&self) -> SessionBuilder<'_> {
        SessionBuilder {
            agent: self,
            tools: vec![],
            error: None,
            model: None,
        }
    }
    /// Get a handle for checking the result of automatic cleanup.
    ///
    /// The handle can outlive the agent without keeping it running. See [Diagnostics]
    /// for an example. Most applications do not need to inspect cleanup separately.
    pub fn diagnostics(&self) -> Diagnostics {
        self.diagnostics.clone()
    }
    /// Stop the agent and wait for cleanup to complete.
    ///
    /// This is optional: letting an agent go out of scope already starts cleanup.
    /// Use this method when your application needs to confirm cleanup finished,
    /// for example before stopping its Tokio runtime. Successful requests are already
    /// complete before this call; shutdown does not finish or submit their answers.
    ///
    /// ~~~no_run
    /// use thinkwell::{agent::Claude, Plan};
    ///
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Claude::connect().await?;
    /// let mut session = agent.session().open().await?;
    /// let answer = Plan::<String>::new()
    ///     .text("Say hello.")
    ///     .run_in(&mut session).await?;
    /// println!("{answer}"); // The answer is ready to use.
    ///
    /// drop(session);
    /// agent.shutdown().await?; // Wait for cleanup before leaving the runtime.
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// The agent is consumed, and all its sessions must be dropped first. Rust checks
    /// this for you. Waiting is limited by [AgentConfig::shutdown_timeout]; a timeout
    /// or cleanup error means success was not confirmed. Once this future has been
    /// polled, dropping it leaves cleanup requested but stops waiting for the outcome.
    pub async fn shutdown(mut self) -> Result<()> {
        self.control.close();
        tokio::time::timeout(self.config.shutdown_timeout, self.done.take().unwrap())
            .await
            .map_err(|_| err(ErrorKind::Deadline, "shutdown deadline"))?
            .map_err(|_| err(ErrorKind::Cleanup, "driver disappeared"))?
    }
    /// Start an agent program and establish a connection to it.
    ///
    /// Pass an [AgentConfig] or a preset such as [Claude](crate::agent::Claude). For the
    /// standard Claude configuration, [Claude::connect](crate::agent::Claude::connect) is
    /// a shorter spelling. Use a custom command when connecting to another agent:
    ///
    /// ~~~no_run
    /// use thinkwell::{Agent, AgentConfig};
    ///
    /// # async fn example() -> thinkwell::Result<()> {
    /// let config = AgentConfig::new("my-acp-agent").args(["--transport", "stdio"]);
    /// let agent = Agent::connect(config).await?;
    /// let mut session = agent.session().open().await?;
    /// // The session is now ready for a Plan.
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// Install and authenticate the agent beforehand. Connecting does not open a
    /// conversation or send a prompt; use [session](Self::session) next.
    ///
    /// # Agent and runtime requirements
    ///
    /// The agent must speak ACP over standard input/output. Opening conversations
    /// also requires HTTP MCP support. Connecting alone does not check every
    /// capability needed later, such as tool support or model selection.
    ///
    /// Call this within Tokio on macOS or Linux, with I/O and time enabled. A missing
    /// runtime, empty command, or invalid timeout produces [ErrorKind::Configuration].
    /// Disabled Tokio I/O or time support can cause a panic when used.
    ///
    /// Starting the process or establishing the connection can fail with a transport
    /// error. Setup is limited by [AgentConfig::setup_timeout]. Cancelling a connection
    /// attempt by dropping its future starts cleanup of any resources already acquired.
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
            let identities = Arc::new(Mutex::new(crate::permissions::Permissions::default()));
            let observed = identities.clone();
            let startup_failed = Arc::new(AtomicBool::new(false));
            let startup_report = startup_failed.clone();
            let requested = identities.clone();
            let servers = Arc::new(Mutex::new(Vec::<HttpTask>::new()));
            let driver_servers = servers.clone();
            let client=Client.v1()
                .on_receive_request(async move |request:RequestPermissionRequest,responder,_| {
                    let allowed=permissions.lock().unwrap().as_ref().is_some_and(|http| {
                        http.active.load(Ordering::SeqCst) && requested.lock().unwrap().allows(&request)
                    });
                    let option=request.options.iter().find(|o|allowed && o.kind==PermissionOptionKind::AllowOnce);
                    let outcome=option.map_or(RequestPermissionOutcome::Cancelled,|o|RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(o.option_id.clone())));
                    responder.respond(RequestPermissionResponse::new(outcome))
                },agent_client_protocol::on_receive_request!())
                .on_receive_notification(async move |notification: SessionNotification, _| {
                    if let SessionUpdate::ToolCall(call) = &notification.update
                        && call.status == ToolCallStatus::Failed
                        && call.title == "mcp__thinkwell__startup"
                    {
                        startup_report.store(true, Ordering::SeqCst);
                    }
                    observed.lock().unwrap().observe(notification);
                    Ok(())
                }, agent_client_protocol::on_receive_notification!())
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
                                startup_failed.store(false, Ordering::SeqCst);
                                *allowed.lock().unwrap()=Some(open.http.clone());
                                driver_servers.lock().unwrap().push(open.server);
                                let operation=async {
                                    let s=cx.send_request(NewSessionRequest::new(directory.path().to_owned()).mcp_servers(vec![
                                        McpServer::Http(McpServerHttp::new("thinkwell",open.url).headers(vec![HttpHeader::new("Authorization",open.http.token.clone())]))
                                    ])).block_task().await.map_err(transport)?;
                                    if startup_failed.load(Ordering::SeqCst) {
                                        return Err(err(ErrorKind::Configuration, "Thinkwell MCP server failed to start; check the adapter configuration for an existing server named thinkwell"));
                                    }
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
                                let tools=allowed.lock().unwrap().as_ref().map(|http| http.tools.iter().map(|tool|tool.name.clone()).collect::<Vec<_>>()).unwrap_or_default();
                                identities.lock().unwrap().begin(id.clone(), tools.into_iter());
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
                                identities.lock().unwrap().end();
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
    /// Check whether this session is still available for another request.
    ///
    /// This is useful after an error: a plan rejected before sending can be corrected,
    /// but a request that failed after starting closes the session and connection.
    ///
    /// ~~~no_run
    /// use thinkwell::{agent::Claude, Plan};
    ///
    /// # async fn example() -> thinkwell::Result<()> {
    /// let agent = Claude::connect().await?;
    /// let mut session = agent.session().open().await?;
    /// if let Err(error) = Plan::<String>::new()
    ///     .text("Suggest a herb to grow indoors.")
    ///     .run_in(&mut session).await
    /// {
    ///     eprintln!("{error}");
    ///     if !session.usable() {
    ///         eprintln!("Connect again to start a new conversation.");
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ~~~
    ///
    /// This checks local state, not whether the agent will answer successfully.
    /// It is false during a running request or after closure. A failed or cancelled
    /// request that already started leaves it permanently false.
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
