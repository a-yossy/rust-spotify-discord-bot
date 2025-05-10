use rig::{
    agent::Agent,
    providers::gemini::{
        Client,
        completion::{CompletionModel, GEMINI_2_0_FLASH},
    },
    tool::ToolSet,
    vector_store::VectorStoreIndexDyn,
};

pub async fn get(
    index: impl VectorStoreIndexDyn + 'static,
    toolset: ToolSet,
) -> Agent<CompletionModel> {
    Client::from_env()
        .agent(GEMINI_2_0_FLASH)
        .max_tokens(25000)
        .dynamic_tools(10, index, toolset)
        .build()
}
