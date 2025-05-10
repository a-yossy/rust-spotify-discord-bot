use anyhow::Result;
use ss_discord_bot::infrastructure::{
    database, discord,
    llm::{self, embedding::Embedding},
    mcp::{server::Server, toolset::ToolSet},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    Server::start().await?;

    let toolset = ToolSet::get().await?;
    let index = Embedding::build_tool_index(&toolset).await?;
    let llm_agent = llm::client::get(index, toolset).await;
    let db_pool = database::pool::get().await?;
    let framework = discord::framework::get(llm_agent, db_pool);
    let mut client = discord::client::get(framework).await;

    client.start().await?;

    Ok(())
}
