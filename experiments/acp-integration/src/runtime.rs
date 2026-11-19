//! Combined foundation experiment. Owned drivers never contain application callbacks.
use crate::result_contract;
use agent_client_protocol::v1::schema::*;
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Client, Error, schema::ProtocolVersion};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{mpsc, oneshot};

pub type Result<T> = std::result::Result<T, String>;
type ToolReply = oneshot::Sender<Result<Value>>;
struct Call {
    name: String,
    args: Value,
    reply: ToolReply,
}
struct Route {
    turn: String,
    calls: mpsc::Sender<Call>,
}
struct Http {
    route: Mutex<Option<Route>>,
    token: String,
}
struct Prompt {
    text: String,
    terminal: oneshot::Sender<Result<()>>,
}

/// Drop-only diagnostics remain observable without retaining the driver owner.
#[derive(Clone, Default)]
pub struct Diagnostics(Arc<Mutex<Option<Result<()>>>>);
impl Diagnostics {
    pub fn outcome(&self) -> Option<Result<()>> {
        self.0.lock().unwrap().clone()
    }
}

pub struct Agent {
    stop: Option<oneshot::Sender<()>>,
    done: Option<oneshot::Receiver<Result<()>>>,
    commands: mpsc::Sender<Prompt>,
    http: Arc<Http>,
    diagnostics: Diagnostics,
}
impl Drop for Agent {
    fn drop(&mut self) {
        self.http.route.lock().unwrap().take();
        self.stop.take(); // Closing the existing driver's signal; no spawn or blocking.
    }
}

pub struct Session {
    agent: Agent,
    usable: bool,
}
impl Session {
    /// A cancelled/abandoned turn permanently poisons this experimental session.
    /// Reuse never implicitly resets history or rebinds stale application calls.
    pub fn usable(&self) -> bool {
        self.usable
    }

    /// The in-flight turn owns the exclusive session borrow.
    ///
    /// ```compile_fail,E0499
    /// use thinkwell_acp_experiment::runtime::Session;
    /// async fn overlap(session: &mut Session) {
    ///     let first = session.run::<String>("one", async |_| Ok(0));
    ///     let second = session.run::<String>("two", async |_| Ok(0));
    ///     let _ = first.await;
    ///     let _ = second.await;
    /// }
    /// ```
    ///
    /// ```compile_fail,E0503
    /// use thinkwell_acp_experiment::runtime::Session;
    /// async fn capture(session: &mut Session) {
    ///     let mut count = 0;
    ///     let execution = session.run::<String>("one", async |v| { count += v; Ok(count) });
    ///     count += 1;
    ///     let _ = execution.await;
    /// }
    /// ```
    pub async fn run<T: DeserializeOwned + JsonSchema>(
        &mut self,
        instruction: &str,
        mut callback: impl AsyncFnMut(i64) -> Result<i64>,
    ) -> Result<T> {
        if !self.usable {
            return Err("session is unsettled; cannot reuse".into());
        }
        let schema = result_contract::contract::<T>()?;
        self.usable = false;
        let turn = uuid::Uuid::new_v4().to_string();
        let (calls, mut incoming) = mpsc::channel(1);
        *self.agent.http.route.lock().unwrap() = Some(Route {
            turn: turn.clone(),
            calls,
        });
        // This guard lives in the borrowing execution, not in the owned driver.
        let guard = Revoke(self.agent.http.clone());
        let (terminal, mut completed) = oneshot::channel();
        let text = format!(
            "{instruction}\nUse only the thinkwell_experiment MCP tools. For every accumulate call use {{\"turn\":\"{turn}\",\"input\":{{\"value\":INTEGER}}}}. Submit your final answer exactly once using return_result with turn {turn} and value conforming to this JSON Schema: {schema}. A rejected value may be corrected. After acceptance end the turn."
        );
        self.agent
            .commands
            .send(Prompt { text, terminal })
            .await
            .map_err(|_| "driver unavailable")?;
        let mut candidate = None;
        let mut rejected = 0;
        let mut application_calls = 0;
        let work = async {
            loop {
                tokio::select! {
                    biased;
                    result = &mut completed => {
                        result.map_err(|_| "driver disappeared")??;
                        // No callback future is running here. Revoke routing before reuse.
                        return candidate.take().ok_or_else(|| "normal completion without result".into());
                    }
                    Some(call) = incoming.recv() => {
                        let response = if call.name == "return_result" {
                            if call.args["turn"] != turn {
                                Err("stale turn identifier".into())
                            } else if call.args.get("value").is_none() {
                                rejected += 1;
                                Err("result envelope requires value".into())
                            } else if candidate.is_some() {
                                Err("duplicate result".into())
                            } else {
                                match result_contract::decode::<T>(&schema, call.args["value"].clone()) {
                                    Ok(value) => { candidate = Some(value); Ok(json!({"accepted":true})) }
                                    Err(e) => { rejected += 1; Err(e) }
                                }
                            }
                        } else if call.name == "accumulate" {
                            application_calls += 1;
                            if application_calls > 4 {
                                let _ = call.reply.send(Err("application call budget exhausted".into()));
                                return Err("application call budget exhausted".into());
                            }
                            match call.args["input"]["value"].as_i64() {
                                Some(input) => {
                                    // Completion/cancellation interrupts callback work, never yields a successful T.
                                    tokio::select! {
                                        biased;
                                        _ = &mut completed => return Err("terminal arrived during callback".into()),
                                        result = callback(input) => result.map(|total| json!({"total":total})),
                                    }
                                }
                                None => Err("expected integer value".into()),
                            }
                        } else { Err("unknown tool".into()) };
                        let _ = call.reply.send(response);
                        if rejected >= 2 { return Err("correction budget exhausted".into()); }
                    }
                }
            }
        };
        let result = tokio::time::timeout(Duration::from_secs(90), work)
            .await
            .map_err(|_| "turn deadline")?;
        drop(guard);
        if result.is_ok() {
            self.usable = true;
        }
        // Conservative: every error closes the agent; no callback rebinding after failure.
        if result.is_err() {
            self.agent.stop.take();
        }
        result
    }

    pub async fn shutdown(self) -> Result<()> {
        self.agent.shutdown().await
    }
    pub fn diagnostics(&self) -> Diagnostics {
        self.agent.diagnostics.clone()
    }
}
struct Revoke(Arc<Http>);
impl Drop for Revoke {
    fn drop(&mut self) {
        self.0.route.lock().unwrap().take();
    }
}

impl Agent {
    pub async fn connect(config: AcpAgentConfig, model: Option<&str>) -> Result<Self> {
        let http = Arc::new(Http {
            route: Mutex::new(None),
            token: format!("Bearer {}", uuid::Uuid::new_v4()),
        });
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .map_err(|e| e.to_string())?;
        let address = listener.local_addr().map_err(|e| e.to_string())?;
        let (stop, mut stopping) = oneshot::channel();
        let (done, completion) = oneshot::channel();
        let (ready, initialized) = oneshot::channel();
        let (commands, mut requests) = mpsc::channel::<Prompt>(1);
        let diagnostics = Diagnostics::default();
        let report = diagnostics.clone();
        let state = http.clone();
        let owner_http = http.clone();
        let model = model.map(str::to_owned);
        // Spawn once at construction. Every object moved here is owned.
        tokio::spawn(async move {
            let (http_stop, stopped) = oneshot::channel();
            let server = tokio::spawn(async move {
                axum::serve(
                    listener,
                    Router::new().route("/mcp", post(handle)).with_state(state),
                )
                .with_graceful_shutdown(async {
                    let _ = stopped.await;
                })
                .await
            });
            // Aborting the outer driver also aborts the separately spawned HTTP task.
            let server_guard = Abort(server.abort_handle());
            let directory =
                tempfile::tempdir().map_err(|e| Error::internal_error().data(e.to_string()));
            let token = http.token.clone();
            let client = Client.v1()
                .on_receive_request(async |request: RequestPermissionRequest, responder, _| {
                    let name = request.tool_call.fields.name.as_deref().or(request.tool_call.fields.title.as_deref());
                    let allowed = matches!(name, Some("mcp__thinkwell_experiment__accumulate" | "mcp__thinkwell_experiment__return_result"));
                    let option = request.options.iter().find(|o| allowed && o.kind == PermissionOptionKind::AllowOnce);
                    let outcome = option.map_or(RequestPermissionOutcome::Cancelled, |o| RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(o.option_id.clone())));
                    responder.respond(RequestPermissionResponse::new(outcome))
                }, agent_client_protocol::on_receive_request!())
                .connect_with(AcpAgent::new(config), async move |cx| {
                    let setup = async {
                        let dir = directory?;
                        cx.send_request(InitializeRequest::new(ProtocolVersion::V1)).block_task().await?;
                        let session = cx.send_request(NewSessionRequest::new(dir.path().to_owned()).mcp_servers(vec![
                            McpServer::Http(McpServerHttp::new("thinkwell_experiment", format!("http://{address}/mcp"))
                                .headers(vec![HttpHeader::new("Authorization", token)]))
                        ])).block_task().await?;
                        if let Some(model) = model {
                            let config: SetSessionConfigOptionRequest = serde_json::from_value(json!({"sessionId":session.session_id,"configId":"model","value":model})).map_err(Error::into_internal_error)?;
                            cx.send_request(config).block_task().await?;
                        }
                        Ok::<_,Error>((session.session_id, dir))
                    };
                    let (session, _directory) = tokio::select! {
                        biased;
                        _ = &mut stopping => return Ok(()),
                        result = tokio::time::timeout(Duration::from_secs(30), setup) => result.map_err(|_| Error::internal_error().data("setup deadline"))??,
                    };
                    let _ = ready.send(());
                    loop {
                        let command = tokio::select! {
                            biased;
                            _ = &mut stopping => break,
                            command = requests.recv() => match command { Some(c) => c, None => break },
                        };
                        let mut terminal = command.terminal;
                        let prompt = cx.send_request(PromptRequest::new(session.clone(), vec![ContentBlock::Text(TextContent::new(command.text))])).block_task();
                        tokio::pin!(prompt);
                        let result = tokio::select! {
                            biased;
                            _ = &mut stopping => break,
                            _ = terminal.closed() => break,
                            result = tokio::time::timeout(Duration::from_secs(95), &mut prompt) => match result {
                                Ok(Ok(r)) if r.stop_reason == StopReason::EndTurn => Ok(()),
                                Ok(Ok(r)) => Err(format!("remote stop: {:?}", r.stop_reason)),
                                Ok(Err(e)) => Err(e.to_string()),
                                Err(_) => Err("driver prompt deadline".into()),
                            },
                        };
                        let _ = terminal.send(result);
                    }
                    Ok::<_,Error>(())
                });
            let outcome = tokio::time::timeout(Duration::from_secs(240), client)
                .await
                .map_err(|_| "driver lifetime deadline".to_string())
                .and_then(|r| r.map_err(|e| e.to_string()));
            let _ = http_stop.send(());
            let http_result = tokio::time::timeout(Duration::from_secs(7), server).await;
            let outcome = outcome.and_then(|()| match http_result {
                Ok(Ok(Ok(()))) => Ok(()),
                _ => Err("HTTP shutdown failed".into()),
            });
            drop(server_guard);
            *report.0.lock().unwrap() = Some(outcome.clone());
            let _ = done.send(outcome);
        });
        let agent = Self {
            stop: Some(stop),
            done: Some(completion),
            commands,
            http: owner_http,
            diagnostics,
        };
        initialized
            .await
            .map_err(|_| "agent initialization failed")?;
        Ok(agent)
    }
    pub fn open(self) -> Session {
        Session {
            agent: self,
            usable: true,
        }
    }
    pub async fn shutdown(mut self) -> Result<()> {
        self.http.route.lock().unwrap().take();
        self.stop.take();
        tokio::time::timeout(Duration::from_secs(12), self.done.take().unwrap())
            .await
            .map_err(|_| "shutdown deadline")?
            .map_err(|_| "driver disappeared")?
    }
}
struct Abort(tokio::task::AbortHandle);
impl Drop for Abort {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn handle(
    State(state): State<Arc<Http>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    if headers.contains_key("origin") {
        return StatusCode::FORBIDDEN.into_response();
    }
    if headers.get("authorization").and_then(|h| h.to_str().ok()) != Some(state.token.as_str()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if headers
        .get("mcp-protocol-version")
        .is_some_and(|r| r != "2025-11-25")
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let id = request["id"].clone();
    if request["jsonrpc"] != "2.0" {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if request["method"] == "notifications/initialized" {
        return StatusCode::ACCEPTED.into_response();
    }
    let result = match request["method"].as_str().unwrap_or("") {
        "initialize" => {
            json!({"protocolVersion":"2025-11-25","capabilities":{"tools":{}},"serverInfo":{"name":"thinkwell-experiment","version":"0.0.0"}})
        }
        "ping" => json!({}),
        "tools/list" => json!({"tools":[
            {"name":"accumulate","description":"Add value to application accumulator","inputSchema":{"type":"object","properties":{"turn":{"type":"string"},"input":schemars::schema_for!(crate::Input)},"required":["turn","input"]}},
            {"name":"return_result","description":"Submit final typed result for the current turn","inputSchema":{"type":"object","properties":{"turn":{"type":"string"},"value":{}},"required":["turn","value"]}}
        ]}),
        "tools/call" => {
            let name = request["params"]["name"].as_str().unwrap_or("").to_owned();
            let args = request["params"]["arguments"].clone();
            let route = {
                let route = state.route.lock().unwrap();
                route.as_ref().and_then(|r| {
                    if !matches!(name.as_str(), "return_result" | "accumulate")
                        || args["turn"].as_str() != Some(r.turn.as_str())
                    {
                        None
                    } else {
                        Some(r.calls.clone())
                    }
                })
            };
            let result = match route {
                None => Err("no active matching turn".into()),
                Some(route) => {
                    let (reply, response) = oneshot::channel();
                    match tokio::time::timeout(Duration::from_secs(10), async {
                        route
                            .send(Call { name, args, reply })
                            .await
                            .map_err(|_| "callback revoked".to_string())?;
                        response.await.map_err(|_| "callback revoked".to_string())?
                    })
                    .await
                    {
                        Ok(result) => result,
                        Err(_) => Err("tool deadline".into()),
                    }
                }
            };
            match result {
                Ok(value) => {
                    json!({"content":[{"type":"text","text":value.to_string()}],"isError":false})
                }
                Err(error) => json!({"content":[{"type":"text","text":error}],"isError":true}),
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
mod boundary_tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn revocation_rejects_calls_and_does_not_rebind_old_routes() {
        tokio::time::timeout(Duration::from_secs(2), async {
            let http = Arc::new(Http {
                route: Mutex::new(None),
                token: "Bearer fixture".into(),
            });
            let (old_sender, old_receiver) = mpsc::channel(1);
            *http.route.lock().unwrap() = Some(Route {
                turn: "old".into(),
                calls: old_sender.clone(),
            });
            drop(Revoke(http.clone()));
            drop(old_receiver);
            let (new_sender, mut new_receiver) = mpsc::channel(1);
            *http.route.lock().unwrap() = Some(Route {
                turn: "new".into(),
                calls: new_sender,
            });
            let (reply, _response) = oneshot::channel();
            assert!(
                old_sender
                    .send(Call {
                        name: "accumulate".into(),
                        args: json!({"value":3}),
                        reply
                    })
                    .await
                    .is_err()
            );
            let mut headers = HeaderMap::new();
            headers.insert("authorization", "Bearer fixture".parse().unwrap());
            let response = handle(
                State(http.clone()),
                headers.clone(),
                Json(json!({
                    "jsonrpc":"2.0","id":1,"method":"tools/call",
                    "params":{"name":"return_result","arguments":{"turn":"old","value":3}}
                })),
            )
            .await;
            let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
            let response: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(response["result"]["isError"], true);
            assert!(new_receiver.try_recv().is_err());
            drop(Revoke(http.clone()));
            let response = handle(
                State(http.clone()),
                headers,
                Json(json!({
                    "jsonrpc":"2.0","id":2,"method":"tools/call",
                    "params":{"name":"accumulate","arguments":{"value":3}}
                })),
            )
            .await;
            let bytes = to_bytes(response.into_body(), 8192).await.unwrap();
            let response: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(response["result"]["isError"], true);
            assert!(new_receiver.recv().await.is_none());
            let denied = handle(State(http), HeaderMap::new(), Json(json!({}))).await;
            assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        })
        .await
        .unwrap();
    }
}
