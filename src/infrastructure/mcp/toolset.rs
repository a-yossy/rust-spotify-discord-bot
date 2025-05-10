use anyhow::Result;
use rig::tool::{ToolDyn as RigTool, ToolEmbeddingDyn, ToolSet as RigToolSet};
use rmcp::{
    ServiceExt,
    model::{CallToolRequestParam, CallToolResult, Tool as McpTool},
    service::ServerSink,
    transport::SseTransport,
};
use serde::Deserialize;

pub struct ToolSet;

impl ToolSet {
    pub async fn get() -> Result<RigToolSet> {
        let transport = SseTransport::start("http://localhost:8000/sse").await?;

        #[derive(Debug, Deserialize)]
        struct McpConfig {
            name: String,
            protocol: String,
            command: String,
            args: Vec<String>,
        }

        #[derive(Debug, Deserialize)]
        struct Config {
            mcp: McpConfig,
        }

        let mcp_manager = ().serve(transport).await?;
        struct McpToolAdaptor {
            tool: McpTool,
            server: ServerSink,
        }

        impl RigTool for McpToolAdaptor {
            fn name(&self) -> String {
                self.tool.name.to_string()
            }

            fn definition(
                &self,
                _prompt: String,
            ) -> std::pin::Pin<
                Box<dyn Future<Output = rig::completion::ToolDefinition> + Send + Sync + '_>,
            > {
                Box::pin(std::future::ready(rig::completion::ToolDefinition {
                    name: self.name(),
                    description: self.tool.description.to_string(),
                    parameters: self.tool.schema_as_json_value(),
                }))
            }

            fn call(
                &self,
                args: String,
            ) -> std::pin::Pin<
                Box<dyn Future<Output = Result<String, rig::tool::ToolError>> + Send + Sync + '_>,
            > {
                let server = self.server.clone();
                Box::pin(async move {
                    let call_mcp_tool_result = server
                        .call_tool(CallToolRequestParam {
                            name: self.tool.name.clone(),
                            arguments: serde_json::from_str(&args)
                                .map_err(rig::tool::ToolError::JsonError)?,
                        })
                        .await
                        .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;

                    Ok(ToolSet::convert_mcp_call_tool_result_to_string(
                        call_mcp_tool_result,
                    ))
                })
            }
        }

        impl ToolEmbeddingDyn for McpToolAdaptor {
            fn context(&self) -> serde_json::Result<serde_json::Value> {
                serde_json::to_value(self.tool.clone())
            }

            fn embedding_docs(&self) -> Vec<String> {
                vec![self.tool.description.to_string()]
            }
        }

        let tools = mcp_manager.list_all_tools().await?;
        let mut tool_builder = RigToolSet::builder();
        for tool in tools {
            let adaptor = McpToolAdaptor {
                tool: tool.clone(),
                server: mcp_manager.peer().clone(),
            };
            tool_builder = tool_builder.dynamic_tool(adaptor);
        }
        let toolset = tool_builder.build();

        Ok(toolset)
    }

    fn convert_mcp_call_tool_result_to_string(result: CallToolResult) -> String {
        serde_json::to_string(&result).unwrap()
    }
}
