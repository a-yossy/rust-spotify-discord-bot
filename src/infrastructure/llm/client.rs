use rig::{
    agent::Agent,
    providers::gemini::{
        Client,
        completion::{CompletionModel, GEMINI_2_0_FLASH},
    },
};

pub fn get() -> Agent<CompletionModel> {
    Client::from_env()
        .agent(GEMINI_2_0_FLASH)
        .max_tokens(25000)
        // .dynamic_tools(20, index, tools)
        .build()
}
