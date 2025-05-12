use rig::{
    agent::Agent as RigAgent,
    providers::gemini::{
        Client,
        completion::{CompletionModel, GEMINI_2_0_FLASH},
    },
    tool::ToolSet,
    vector_store::VectorStoreIndexDyn,
};

pub struct Agent;

impl Agent {
    pub async fn get(
        index: impl VectorStoreIndexDyn + 'static,
        toolset: ToolSet,
    ) -> RigAgent<CompletionModel> {
        Client::from_env()
            .agent(GEMINI_2_0_FLASH)
            .max_tokens(25000)
            .preamble(
                "
                    使用可能なツールがある場合は、ツールを使用してください。
                    使用可能なツールがない場合は、ツールを使用せず回答してください。
                    ツール呼び出しの結果を受け取った場合は、受け取った内容に基づいて回答してください。
                ",
            )
            .dynamic_tools(30, index, toolset)
            .build()
    }
}
