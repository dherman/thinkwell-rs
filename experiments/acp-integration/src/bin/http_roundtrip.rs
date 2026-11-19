//! Explicit live experiment. `cargo test` never runs model inference.
use agent_client_protocol::v1::schema::*;
use agent_client_protocol::{AcpAgent, AcpAgentConfig, Client, Error, schema::ProtocolVersion};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
};
use schemars::{JsonSchema, schema_for};
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    cell::Cell,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use thinkwell_acp_experiment::{Input, ToolHandle, local_tool};
use tokio::sync::oneshot;

const REVISION: &str = "2025-11-25";
const TOOL: &str = "mcp__thinkwell_experiment__accumulate";

#[derive(Serialize, JsonSchema)]
struct ToolOutput {
    total: i64,
    receipt: String,
}

struct HttpState {
    tool: ToolHandle,
    authorization: String,
    receipt: String,
    events: Mutex<Vec<Value>>,
    calls: AtomicUsize,
}
impl HttpState {
    fn record(&self, event: Value) {
        self.events.lock().unwrap().push(event);
    }
}

fn rpc_error(id: Value, code: i64, message: &str) -> Response {
    Json(json!({"jsonrpc":"2.0", "id":id, "error":{"code":code,"message":message}})).into_response()
}

async fn handle(
    State(state): State<Arc<HttpState>>,
    headers: HeaderMap,
    Json(request): Json<Value>,
) -> Response {
    // This CLI-only fixture exposes neither a browser origin nor a public listener.
    if headers.contains_key("origin") {
        return StatusCode::FORBIDDEN.into_response();
    }
    if headers.get("authorization").and_then(|v| v.to_str().ok()) != Some(&state.authorization) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    if request["jsonrpc"] != "2.0" {
        return rpc_error(id, -32600, "Invalid JSON-RPC envelope");
    }
    let method = request["method"].as_str().unwrap_or("");
    let revision = headers
        .get("mcp-protocol-version")
        .and_then(|v| v.to_str().ok());
    state.record(json!({"method":method, "protocol_header":revision}));
    if method != "initialize" && revision.is_some_and(|r| r != REVISION) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    if method == "notifications/initialized" && id.is_null() {
        return StatusCode::ACCEPTED.into_response();
    }
    if id.is_null() {
        return rpc_error(id, -32600, "Request ID required");
    }
    let result = match method {
        "initialize" => {
            state.record(
                json!({"offered":request["params"]["protocolVersion"], "selected":REVISION}),
            );
            json!({"protocolVersion":REVISION, "capabilities":{"tools":{}},
                "serverInfo":{"name":"thinkwell-experiment", "version":"0.0.0"}})
        }
        "ping" => json!({}),
        "tools/list" => json!({"tools":[{
            "name":"accumulate", "description":"Add value to the experiment accumulator and return its total and receipt.",
            "inputSchema":schema_for!(Input), "outputSchema":schema_for!(ToolOutput)
        }]}),
        "tools/call" => {
            if request["params"]["name"] != "accumulate" {
                return rpc_error(id, -32602, "Unknown tool");
            }
            let Ok(input) = serde_json::from_value::<Input>(request["params"]["arguments"].clone())
            else {
                return rpc_error(id, -32602, "Invalid tool input");
            };
            if !matches!(input.value, 3 | 4) {
                return rpc_error(id, -32602, "Experiment accepts only 3 or 4");
            }
            if state.calls.fetch_add(1, Ordering::SeqCst) >= 2 {
                return rpc_error(id, -32602, "Two-call experiment budget exhausted");
            }
            state.record(json!({"tool_input":input.value}));
            match tokio::time::timeout(Duration::from_secs(5), state.tool.invoke(input)).await {
                Ok(Ok(output)) => {
                    let output = ToolOutput {
                        total: output.total,
                        receipt: state.receipt.clone(),
                    };
                    json!({"content":[{"type":"text", "text":serde_json::to_string(&output).unwrap()}],
                        "structuredContent":output, "isError":false})
                }
                _ => {
                    json!({"content":[{"type":"text", "text":"Local callback unavailable"}], "isError":true})
                }
            }
        }
        _ => return rpc_error(id, -32601, "Method not implemented by experiment"),
    };
    Json(json!({"jsonrpc":"2.0", "id":id, "result":result})).into_response()
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let working_dir = tempfile::tempdir()?;
    let (tool, dispatch) = local_tool();
    let state = Arc::new(HttpState {
        tool,
        authorization: format!("Bearer {}", uuid::Uuid::new_v4()),
        receipt: uuid::Uuid::new_v4().to_string(),
        events: Mutex::new(Vec::new()),
        calls: AtomicUsize::new(0),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let router = Router::new()
        .route("/mcp", post(handle))
        .with_state(state.clone());
    let (stop_http, stopped) = oneshot::channel();
    let mut server = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                let _ = stopped.await;
            })
            .await
    });
    let messages = Arc::new(Mutex::new(String::new()));
    let notification_messages = messages.clone();
    let permissions = Arc::new(Mutex::new(Vec::<Value>::new()));
    let permission_log = permissions.clone();
    let agent = AcpAgent::new(AcpAgentConfig::new("npx").args([
        "--offline",
        "--yes",
        "@zed-industries/claude-agent-acp@0.23.1",
    ]));
    let client = Client.v1()
        .on_receive_notification(async move |notification: SessionNotification, _| {
            if let SessionUpdate::AgentMessageChunk(chunk) = notification.update
                && let ContentBlock::Text(text) = chunk.content {
                let mut messages = notification_messages.lock().unwrap();
                if messages.len() < 32_768 { messages.push_str(&text.text); }
            }
            Ok(())
        }, agent_client_protocol::on_receive_notification!())
        .on_receive_request(async move |request: RequestPermissionRequest, responder, _| {
            let fields = request.tool_call.fields;
            // Permission identifies the only allowed tool. The HTTP boundary validates
            // its arguments independently before any callback can run.
            let recognized = fields.name.as_deref().or(fields.title.as_deref()) == Some(TOOL);
            permission_log.lock().unwrap().push(json!({"tool":fields.name.or(fields.title), "allowed":recognized}));
            let option = recognized.then(|| request.options.iter().find(|o| o.kind == PermissionOptionKind::AllowOnce)).flatten();
            let outcome = match option {
                Some(option) => RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(option.option_id.clone())),
                None => RequestPermissionOutcome::Cancelled,
            };
            responder.respond(RequestPermissionResponse::new(outcome))
        }, agent_client_protocol::on_receive_request!())
        .connect_with(agent, async |cx| {
            // The timeout is inside connect_with, so normal errors still drive SDK cleanup.
            tokio::time::timeout(Duration::from_secs(120), async {
                let initialized = cx.send_request(InitializeRequest::new(ProtocolVersion::V1)).block_task().await?;
                eprintln!("Initialized real ACP agent; creating HTTP-tool session");
                let session = cx.send_request(NewSessionRequest::new(working_dir.path().to_path_buf()).mcp_servers(vec![
                    McpServer::Http(McpServerHttp::new("thinkwell_experiment", format!("http://{address}/mcp"))
                        .headers(vec![HttpHeader::new("Authorization", state.authorization.clone())]))
                ])).block_task().await?;
                let config: SetSessionConfigOptionRequest = serde_json::from_value(json!({
                    "sessionId":session.session_id, "configId":"model", "value":"haiku"
                })).map_err(Error::into_internal_error)?;
                cx.send_request(config).block_task().await?;
                eprintln!("Sending one bounded prompt (Haiku)");
                let response = cx.send_request(PromptRequest::new(session.session_id, vec![ContentBlock::Text(TextContent::new(
                    "This is a small protocol integration test. Use only the thinkwell_experiment accumulate MCP tool. Call it with value 3, then with value 4. Do not use any other tools. Reply only with the final total and the exact receipt from the second tool result. Do not guess the receipt."
                ))])).block_task().await?;
                Ok::<_, Error>(json!({"agent":initialized.agent_info, "stop_reason":response.stop_reason, "model":"haiku"}))
            }).await.map_err(|_| Error::internal_error().data("120-second experiment deadline"))?
        });
    let mut inputs = Vec::new();
    let total = Rc::new(Cell::new(0));
    let result = {
        let local = dispatch.run(async |input| {
            inputs.push(input.value);
            tokio::task::yield_now().await;
            total.set(total.get() + input.value);
            Ok(thinkwell_acp_experiment::Output { total: total.get() })
        });
        tokio::pin!(local);
        tokio::pin!(client);
        // Outer bound covers faulty transport cleanup too; a hard timeout is a cleanup gap.
        tokio::time::timeout(Duration::from_secs(135), async {
            tokio::select! { result = &mut client => result, _ = &mut local => client.await }
        })
        .await
    };
    let _ = stop_http.send(());
    let http_joined = match tokio::time::timeout(Duration::from_secs(7), &mut server).await {
        Ok(result) => result?.is_ok(),
        Err(_) => {
            server.abort();
            let _ = server.await;
            false
        }
    };
    let listener_closed = tokio::net::TcpStream::connect(address).await.is_err();
    let receipt_seen = messages.lock().unwrap().contains(&state.receipt);
    let state_matches = inputs == [3, 4] && total.get() == 7;
    // Demonstrate caller access after the borrowing execution future has been dropped.
    inputs.push(0);
    let protocol = match result {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => json!({"error_code":error.code, "error_message":error.message}),
        Err(_) => json!({"hard_timeout":true}),
    };
    let success = protocol.get("error_code").is_none()
        && protocol.get("hard_timeout").is_none()
        && receipt_seen
        && state_matches
        && http_joined
        && listener_closed;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "success":success, "protocol":protocol,
            "mcp_events":state.events.lock().unwrap().clone(), "permissions":permissions.lock().unwrap().clone(),
            "inputs_before_caller_mutation":&inputs[..inputs.len()-1], "total":total.get(),
            "tool_receipt_seen_in_agent_reply":receipt_seen, "http_server_joined":http_joined,
            "listener_closed":listener_closed, "caller_access_restored":true,
        }))?
    );
    if !success {
        return Err("live round trip did not meet all experiment assertions".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use thinkwell_acp_experiment::LocalDispatch;

    fn fixture() -> (Arc<HttpState>, LocalDispatch, HeaderMap) {
        let (tool, dispatch) = local_tool();
        let state = Arc::new(HttpState {
            tool,
            authorization: "Bearer fixture".into(),
            receipt: "tool-only-receipt".into(),
            events: Mutex::new(Vec::new()),
            calls: AtomicUsize::new(0),
        });
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer fixture".parse().unwrap());
        headers.insert("mcp-protocol-version", REVISION.parse().unwrap());
        (state, dispatch, headers)
    }

    async fn decode(response: Response) -> Value {
        let bytes = to_bytes(response.into_body(), 16384).await.unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn negotiation_and_derived_contract() {
        let (state, _dispatch, headers) = fixture();
        let initialized = handle(State(state.clone()), headers.clone(), Json(json!({
            "jsonrpc":"2.0", "id":1, "method":"initialize", "params":{"protocolVersion":"2025-11-25"}
        }))).await;
        assert_eq!(
            decode(initialized).await["result"]["protocolVersion"],
            REVISION
        );
        let listed = handle(
            State(state),
            headers,
            Json(json!({"jsonrpc":"2.0", "id":2, "method":"tools/list"})),
        )
        .await;
        let listed = decode(listed).await;
        let tool = &listed["result"]["tools"][0];
        assert_eq!(tool["name"], "accumulate");
        assert_eq!(
            tool["inputSchema"]["properties"]["value"]["type"],
            "integer"
        );
        assert_eq!(
            tool["outputSchema"]["properties"]["receipt"]["type"],
            "string"
        );
    }

    #[tokio::test]
    async fn rejected_requests_never_reach_callback_queue() {
        let (state, _dispatch, headers) = fixture();
        let call = json!({"jsonrpc":"2.0", "id":1, "method":"tools/call", "params":{"name":"accumulate", "arguments":{"value":3}}});
        let denied = handle(State(state.clone()), HeaderMap::new(), Json(call.clone())).await;
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);
        let mut origin = headers.clone();
        origin.insert("origin", "https://example.com".parse().unwrap());
        assert_eq!(
            handle(State(state.clone()), origin, Json(call.clone()))
                .await
                .status(),
            StatusCode::FORBIDDEN
        );
        let mut wrong_revision = headers.clone();
        wrong_revision.insert("mcp-protocol-version", "2026-07-28".parse().unwrap());
        assert_eq!(
            handle(State(state.clone()), wrong_revision, Json(call.clone()))
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
        for value in [json!("3"), json!(99)] {
            let mut invalid = call.clone();
            invalid["params"]["arguments"]["value"] = value;
            let result =
                decode(handle(State(state.clone()), headers.clone(), Json(invalid)).await).await;
            assert_eq!(result["error"]["code"], -32602);
        }
        assert_eq!(state.calls.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn http_calls_reach_borrowed_state_and_respect_call_budget() {
        tokio::time::timeout(Duration::from_secs(5), async {
            let (state, dispatch, headers) = fixture();
            let mut inputs = vec![];
            let total = Rc::new(Cell::new(0));
            {
                let local = dispatch.run(async |input| {
                    inputs.push(input.value);
                    tokio::task::yield_now().await;
                    total.set(total.get() + input.value);
                    Ok(thinkwell_acp_experiment::Output { total:total.get() })
                });
                let requests = async {
                    for (id, value, expected) in [(1,3,3), (2,4,7), (3,3,7)] {
                        let response = handle(State(state.clone()), headers.clone(), Json(json!({
                            "jsonrpc":"2.0", "id":id, "method":"tools/call", "params":{"name":"accumulate", "arguments":{"value":value}}
                        }))).await;
                        let decoded = decode(response).await;
                        if id == 3 { assert_eq!(decoded["error"]["code"], -32602); }
                        else {
                            assert_eq!(decoded["result"]["structuredContent"], json!({"total":expected, "receipt":"tool-only-receipt"}));
                        }
                    }
                };
                tokio::select! { _ = requests => {}, _ = local => panic!("dispatcher exited early") }
            }
            assert_eq!(inputs, [3,4]);
            inputs.push(9);
            assert_eq!(total.get(), 7);
        }).await.unwrap();
    }
}
