use rig::{agent::Agent, providers::gemini::completion::CompletionModel};
use sqlx::MySqlPool;

pub struct Data {
    pub llm_agent: Agent<CompletionModel>,
    pub db_pool: MySqlPool,
}

impl Data {
    pub fn new(llm_agent: Agent<CompletionModel>, db_pool: MySqlPool) -> Self {
        Self { llm_agent, db_pool }
    }
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
