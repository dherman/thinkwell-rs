//! Isolated ownership experiment; not Thinkwell's product API.
use agent_client_protocol::{Agent, Error, mcp_server::*};
use futures::future::BoxFuture;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

#[derive(Deserialize, JsonSchema)]
pub struct Input {
    pub value: i64,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, PartialEq)]
pub struct Output {
    pub total: i64,
}

type Reply = oneshot::Sender<Result<Output, Error>>;

pub struct LocalDispatch(mpsc::Receiver<(Input, Reply)>);

pub struct ToolHandle(mpsc::Sender<(Input, Reply)>);

pub fn local_tool() -> (ToolHandle, LocalDispatch) {
    let (tx, rx) = mpsc::channel(1);
    (ToolHandle(tx), LocalDispatch(rx))
}

impl McpTool<Agent> for ToolHandle {
    type Input = Input;
    type Output = Output;

    fn name(&self) -> String {
        "accumulate".into()
    }

    fn description(&self) -> String {
        "Add a value to application state".into()
    }

    async fn call_tool(&self, input: Input, _: McpConnectionTo<Agent>) -> Result<Output, Error> {
        self.invoke(input).await
    }
}

impl ToolHandle {
    /// Owned request boundary shared by native MCP and the HTTP experiment.
    pub async fn invoke(&self, input: Input) -> Result<Output, Error> {
        let (reply, response) = oneshot::channel();
        self.0
            .send((input, reply))
            .await
            .map_err(Error::into_internal_error)?;
        response.await.map_err(Error::into_internal_error)?
    }
}

impl LocalDispatch {
    /// No Send or 'static bounds on the application callback or its future.
    /// Receiver disappearance cancels an invocation; previous effects remain.
    pub async fn run(mut self, mut callback: impl AsyncFnMut(Input) -> Result<Output, Error>) {
        while let Some((input, mut reply)) = self.0.recv().await {
            tokio::select! {
                biased;
                _ = reply.closed() => {},
                result = callback(input) => { let _ = reply.send(result); }
            }
        }
    }
}

/// Minimal tools/call service for the experiment, not a complete MCP server.
/// The SDK registry supplies type deserialization and output serialization.
pub struct ToolService(Arc<McpToolRegistry<Agent>>);

impl ToolService {
    pub fn new(tool: impl McpTool<Agent> + 'static) -> Self {
        let mut registry = McpToolRegistry::default();
        registry.register_tool(tool);
        Self(Arc::new(registry))
    }
}

impl McpService<Agent> for ToolService {
    fn execute(
        &self,
        request: McpRequest,
        context: McpRequestContext<Agent>,
    ) -> BoxFuture<'static, Result<McpOutcome, Error>> {
        let registry = self.0.clone();
        Box::pin(async move {
            if request.method != "tools/call" {
                return Err(Error::method_not_found());
            }
            let params = request.params.ok_or_else(Error::invalid_params)?;
            let name = params
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or_else(Error::invalid_params)?;
            let tool = registry
                .enabled_tool(name)
                .ok_or_else(Error::invalid_params)?;
            let arguments = params
                .get("arguments")
                .cloned()
                .ok_or_else(Error::invalid_params)?;
            tokio::select! {
                biased;
                _ = context.operation_cancellation().cancelled() => Err(Error::request_cancelled()),
                result = tool.call_tool(arguments, context.connection().clone()) => {
                    let output = result?;
                    Ok(McpOutcome::Result(json!({
                        "content": [{"type": "text", "text": output.to_string()}],
                        "structuredContent": output,
                        "isError": false
                    })))
                }
            }
        })
    }
}

#[cfg(test)]
mod tests;

pub mod result_contract;
pub mod runtime;
