use anyhow::Result;
use serenity::async_trait;
use serenity::model::channel::Message as SerenityMessage;
use serenity::prelude::*;
use ss_discord_bot::infrastructure::{database, discord, llm};

// pub fn convert_mcp_call_tool_result_to_string(result: CallToolResult) -> String {
//     serde_json::to_string(&result).unwrap()
// }

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: SerenityMessage) {
        // #[derive(Debug, Deserialize)]
        // struct McpConfig {
        //     name: String,
        //     protocol: String,
        //     command: String,
        //     args: Vec<String>,
        // }

        // #[derive(Debug, Deserialize)]
        // struct Config {
        //     mcp: McpConfig,
        // }

        // let content = tokio::fs::read_to_string("config.toml").await.unwrap();
        // let config: Config = toml::from_str(&content).unwrap();
        // let transport = rmcp::transport::TokioChildProcess::new(
        //     tokio::process::Command::new(config.mcp.command).args(config.mcp.args),
        // )
        // .unwrap();
        // let mcp_manager = ().serve(transport).await.unwrap();
        // struct McpToolAdaptor {
        //     tool: McpTool,
        //     server: ServerSink,
        // }

        // impl RigTool for McpToolAdaptor {
        //     fn name(&self) -> String {
        //         self.tool.name.to_string()
        //     }

        //     fn definition(
        //         &self,
        //         _prompt: String,
        //     ) -> std::pin::Pin<
        //         Box<dyn Future<Output = rig::completion::ToolDefinition> + Send + Sync + '_>,
        //     > {
        //         Box::pin(std::future::ready(rig::completion::ToolDefinition {
        //             name: self.name(),
        //             description: self.tool.description.to_string(),
        //             parameters: self.tool.schema_as_json_value(),
        //         }))
        //     }

        //     fn call(
        //         &self,
        //         args: String,
        //     ) -> std::pin::Pin<
        //         Box<dyn Future<Output = Result<String, rig::tool::ToolError>> + Send + Sync + '_>,
        //     > {
        //         let server = self.server.clone();
        //         Box::pin(async move {
        //             let call_mcp_tool_result = server
        //                 .call_tool(CallToolRequestParam {
        //                     name: self.tool.name.clone(),
        //                     arguments: serde_json::from_str(&args)
        //                         .map_err(rig::tool::ToolError::JsonError)?,
        //                 })
        //                 .await
        //                 .map_err(|e| rig::tool::ToolError::ToolCallError(Box::new(e)))?;

        //             Ok(convert_mcp_call_tool_result_to_string(call_mcp_tool_result))
        //         })
        //     }
        // }

        // impl ToolEmbeddingDyn for McpToolAdaptor {
        //     fn context(&self) -> serde_json::Result<serde_json::Value> {
        //         serde_json::to_value(self.tool.clone())
        //     }

        //     fn embedding_docs(&self) -> Vec<String> {
        //         vec![self.tool.description.to_string()]
        //     }
        // }

        // let tools = mcp_manager.list_all_tools().await.unwrap();
        // let mut tool_builder = ToolSet::builder();
        // for tool in tools {
        //     let adaptor = McpToolAdaptor {
        //         tool: tool.clone(),
        //         server: mcp_manager.peer().clone(),
        //     };
        //     tool_builder = tool_builder.dynamic_tool(adaptor);
        // }
        // let tools = tool_builder.build();

        // let embedding_model = client.embedding_model(GEMINI_2_0_FLASH);
        // let cohere_client = cohere::Client::from_env();
        // let embedding_model =
        //     cohere_client.embedding_model(cohere::EMBED_MULTILINGUAL_V3, "classification");

        // let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
        //     .documents(tools.schemas().unwrap())
        //     .unwrap()
        //     .build()
        //     .await
        //     .unwrap();
        // let store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |f| f.name.clone());
        // let index = store.index(embedding_model);
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let llm_agent = llm::client::get();
    let db_pool = database::pool::get().await?;
    let framework = discord::framework::get(llm_agent, db_pool);
    let mut client = discord::client::get(framework).await;
    client.start().await?;

    Ok(())
}
