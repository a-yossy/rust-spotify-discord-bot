use rig::{agent::Agent, providers::gemini::completion::CompletionModel};

pub struct Data {
    pub llm_agent: Agent<CompletionModel>,
}

impl Data {
    pub fn new(llm_agent: Agent<CompletionModel>) -> Self {
        Self { llm_agent }
    }
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
