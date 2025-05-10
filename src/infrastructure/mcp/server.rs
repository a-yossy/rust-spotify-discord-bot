use anyhow::Result;
use rmcp::{ServerHandler, tool, transport::SseServer};

pub struct Server;

impl Server {
    const BIND_ADDRESS: &str = "127.0.0.1:8000";

    pub async fn start() -> Result<()> {
        SseServer::serve(Self::BIND_ADDRESS.parse()?)
            .await?
            .with_service(Mcp::new);

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct Mcp;

#[tool(tool_box)]
impl Mcp {
    pub fn new() -> Self {
        Self
    }

    #[tool(description = "MCP サーバーを使っていることを確認する")]
    fn mcp() -> String {
        "MCP サーバーを使っています".to_string()
    }
}

#[tool(tool_box)]
impl ServerHandler for Mcp {}
