use anyhow::Result;
use ss_discord_bot::infrastructure::{
    database::pool::Pool,
    discord::{client::Client, framework::Framework},
    llm::{agent::Agent, embedding::Embedding},
    mcp::{server::Server, toolset::ToolSet},
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    let db_pool = Pool::get().await?;
    Server::start(db_pool.clone()).await?;
    
    let toolset = ToolSet::get().await?;
    let index = Embedding::build_tool_index(&toolset).await?;
    let agent = Agent::get(index, toolset).await;
    let framework = Framework::get(agent, db_pool.clone());
    let mut client = Client::get(framework).await;
    client.start().await?;

    Ok(())
}
