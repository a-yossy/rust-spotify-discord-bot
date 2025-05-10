use anyhow::Result;
use ss_discord_bot::infrastructure::{database, discord, llm, mcp::server::Server};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv()?;

    Server::start().await?;

    let llm_agent = llm::client::get().await;
    let db_pool = database::pool::get().await?;
    let framework = discord::framework::get(llm_agent, db_pool);
    let mut client = discord::client::get(framework).await;

    client.start().await?;

    Ok(())
}
