use super::*;
use agent_client_protocol::{Channel, Client, NullRun, RunWithConnectionTo, TransportFrame};
use futures::{
    StreamExt,
    future::{AbortHandle, Abortable, pending},
};
use serde_json::Value;
use std::{cell::Cell, rc::Rc, time::Duration};

struct Peer {
    channel: Channel,
    server: Value,
}

fn send(channel: &Channel, message: Value) {
    channel
        .tx
        .unbounded_send(TransportFrame::Single(
            serde_json::from_value(message).unwrap(),
        ))
        .unwrap();
}

async fn receive(channel: &mut Channel) -> Value {
    match channel
        .rx
        .next()
        .await
        .expect("connection closed before response")
    {
        TransportFrame::Single(message) => serde_json::to_value(message).unwrap(),
        _ => panic!("unexpected batch"),
    }
}

impl Peer {
    async fn accept(mut channel: Channel) -> Self {
        let setup = receive(&mut channel).await;
        assert_eq!(setup["method"], "session/new");
        let server = setup["params"]["mcpServers"][0]["serverId"].clone();
        assert!(server.is_string());
        send(
            &channel,
            json!({"jsonrpc":"2.0", "id":setup["id"], "result":{"sessionId":"experiment"}}),
        );
        Self { channel, server }
    }

    fn call(&self, id: i64, arguments: Value) {
        send(
            &self.channel,
            json!({
                "jsonrpc":"2.0", "id":id, "method":"mcp/message",
                "params":{
                    "serverId":self.server, "requestId":format!("tool-{id}"), "method":"tools/call",
                    "params": {"name":"accumulate", "arguments":arguments, "_meta":{
                        "io.modelcontextprotocol/protocolVersion":"2026-07-28",
                        "io.modelcontextprotocol/clientCapabilities":{}
                    }}
                }
            }),
        );
    }

    async fn response(&mut self, id: i64) -> Value {
        let value = receive(&mut self.channel).await;
        assert_eq!(value["id"], id, "{value}");
        value
    }

    async fn total(&mut self, id: i64, value: i64, expected: i64) {
        self.call(id, json!({"value": value}));
        let response = self.response(id).await;
        assert_eq!(
            response["result"]["result"]["structuredContent"],
            json!({"total":expected}),
            "{response}"
        );
    }
}

/// No tasks are spawned: all borrowing futures are owned by this call.
async fn exchange(
    service: ToolService,
    runner: impl RunWithConnectionTo<Agent>,
    local: impl Future<Output = ()>,
    script: impl AsyncFnOnce(Peer, oneshot::Sender<()>, AbortHandle),
    abort_expected: bool,
) {
    tokio::time::timeout(Duration::from_secs(5), async {
        let (transport, peer) = Channel::duplex();
        let (stop, stopped) = oneshot::channel();
        let client = Client.v1().connect_with(transport, async move |cx| {
            cx.build_session_cwd()?
                .with_mcp_server(McpServer::new_service("experiment", service, runner))?
                .block_task()
                .run_until(async |_session| {
                    stopped.await.map_err(Error::into_internal_error)?;
                    Ok(())
                })
                .await
        });
        let execution = async {
            tokio::pin!(local);
            // A finished dispatcher is allowed: the service then reports closed replies.
            // Continue driving the connection so it can report that error.
            tokio::pin!(client);
            tokio::select! {
                result = &mut client => result,
                _ = &mut local => client.await,
            }
        };
        let (abort, registration) = AbortHandle::new_pair();
        let (result, ()) = tokio::join!(Abortable::new(execution, registration), async {
            script(Peer::accept(peer).await, stop, abort).await
        });
        if abort_expected {
            assert!(result.is_err());
        } else {
            result.expect("unexpected abort").unwrap();
        }
    })
    .await
    .expect("experiment timed out");
}

#[tokio::test(flavor = "current_thread")]
async fn local_non_send_borrow_survives_await_and_returns_typed_output() {
    let mut total = 0;
    let local_only = Rc::new(Cell::new(0));
    let (tool, dispatch) = local_tool();
    exchange(
        ToolService::new(tool),
        NullRun,
        dispatch.run(async |input| {
            total += input.value;
            tokio::task::yield_now().await;
            local_only.set(local_only.get() + 1);
            Ok(Output { total })
        }),
        async |mut peer, stop, _| {
            peer.total(1, 3, 3).await;
            peer.total(2, 4, 7).await;
            stop.send(()).unwrap();
        },
        false,
    )
    .await;
    total += 10;
    assert_eq!(total, 17);
    assert_eq!(local_only.get(), 2);
}

#[tokio::test(flavor = "current_thread")]
async fn direct_send_callback_borrows_without_static() {
    let mut total = 0;
    let callback = async |input: Input, _: McpConnectionTo<Agent>| {
        total += input.value;
        tokio::task::yield_now().await;
        Ok(Output { total })
    };
    let (tool, runner) = tool_fn_mut("accumulate", "Add", callback, |callback, input, context| {
        Box::pin(callback(input, context))
    });
    exchange(
        ToolService::new(tool),
        runner,
        pending(),
        async |mut peer, stop, _| {
            peer.total(1, 2, 2).await;
            peer.total(2, 5, 7).await;
            stop.send(()).unwrap();
        },
        false,
    )
    .await;
    total += 10;
    assert_eq!(total, 17);
}

struct DropSignal(Option<oneshot::Sender<()>>);
impl Drop for DropSignal {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn remote_cancel_drops_local_invocation_and_allows_next_call() {
    let mut total = 0;
    let (started, start) = oneshot::channel();
    let (dropped, drop_seen) = oneshot::channel();
    let mut started = Some(started);
    let mut dropped = Some(dropped);
    let (tool, dispatch) = local_tool();
    exchange(
        ToolService::new(tool),
        NullRun,
        dispatch.run(async |input| {
            total += input.value;
            if input.value == 1 {
                let _guard = DropSignal(dropped.take());
                started.take().unwrap().send(()).unwrap();
                pending::<()>().await;
            }
            Ok(Output { total })
        }),
        async |mut peer, stop, _| {
            peer.call(1, json!({"value":1}));
            start.await.unwrap();
            send(
                &peer.channel,
                json!({"jsonrpc":"2.0", "method":"$/cancel_request", "params":{"requestId":1}}),
            );
            drop_seen.await.unwrap();
            let response = peer.response(1).await;
            assert!(response.get("error").is_some(), "{response}");
            peer.total(2, 4, 5).await;
            stop.send(()).unwrap();
        },
        false,
    )
    .await;
    assert_eq!(total, 5, "cancellation does not roll back prior mutations");
}

#[tokio::test(flavor = "current_thread")]
async fn dropping_execution_releases_inflight_borrow() {
    let mut total = 0;
    let (started, start) = oneshot::channel();
    let (dropped, drop_seen) = oneshot::channel();
    let mut started = Some(started);
    let mut dropped = Some(dropped);
    let (tool, dispatch) = local_tool();
    exchange(
        ToolService::new(tool),
        NullRun,
        dispatch.run(async |input| {
            total += input.value;
            let _guard = DropSignal(dropped.take());
            started.take().unwrap().send(()).unwrap();
            pending::<()>().await;
            Ok(Output { total })
        }),
        async |peer, _stop, abort| {
            peer.call(1, json!({"value":9}));
            start.await.unwrap();
            abort.abort();
            drop_seen.await.unwrap();
        },
        true,
    )
    .await;
    total += 1;
    assert_eq!(total, 10);
}

#[tokio::test(flavor = "current_thread")]
async fn callback_and_invalid_input_errors_do_not_poison_session() {
    let mut calls = 0;
    let (tool, dispatch) = local_tool();
    exchange(
        ToolService::new(tool),
        NullRun,
        dispatch.run(async |input| {
            calls += 1;
            if input.value < 0 {
                return Err(Error::internal_error().data("fixture tool failure"));
            }
            Ok(Output { total: input.value })
        }),
        async |mut peer, stop, _| {
            peer.call(1, json!({"value":"wrong type"}));
            assert!(peer.response(1).await.get("error").is_some());
            peer.call(2, json!({"value":-1}));
            assert!(peer.response(2).await.get("error").is_some());
            peer.total(3, 8, 8).await;
            stop.send(()).unwrap();
        },
        false,
    )
    .await;
    assert_eq!(calls, 2, "invalid input must not invoke callback");
}

#[tokio::test(flavor = "current_thread")]
async fn missing_dispatcher_reports_error_instead_of_hanging() {
    let (tool, dispatch) = local_tool();
    drop(dispatch);
    exchange(
        ToolService::new(tool),
        NullRun,
        pending(),
        async |mut peer, stop, _| {
            peer.call(1, json!({"value":1}));
            assert!(peer.response(1).await.get("error").is_some());
            stop.send(()).unwrap();
        },
        false,
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn finishing_session_cancels_pending_local_call() {
    let mut total = 0;
    let (started, start) = oneshot::channel();
    let (dropped, drop_seen) = oneshot::channel();
    let mut started = Some(started);
    let mut dropped = Some(dropped);
    let (tool, dispatch) = local_tool();
    exchange(
        ToolService::new(tool),
        NullRun,
        dispatch.run(async |input| {
            total += input.value;
            let _guard = DropSignal(dropped.take());
            started.take().unwrap().send(()).unwrap();
            pending::<()>().await;
            Ok(Output { total })
        }),
        async |peer, stop, _| {
            peer.call(1, json!({"value": 11}));
            start.await.unwrap();
            stop.send(()).unwrap();
            drop_seen.await.unwrap();
        },
        false,
    )
    .await;
    total += 1;
    assert_eq!(total, 12);
}
