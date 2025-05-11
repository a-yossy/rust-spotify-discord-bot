use anyhow::Result;
use rig::{
    embeddings::EmbeddingsBuilder,
    providers::gemini,
    tool::ToolSet,
    vector_store::{VectorStoreIndexDyn, in_memory_store::InMemoryVectorStore},
};

pub struct Embedding;

impl Embedding {
    pub async fn build_tool_index(toolset: &ToolSet) -> Result<impl VectorStoreIndexDyn + 'static> {
        let client = gemini::Client::from_env();
        let embedding_model = client.embedding_model(gemini::embedding::EMBEDDING_004);
        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(toolset.schemas()?)?
            .build()
            .await?;
        let store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |f| f.name.clone());
        let index = store.index(embedding_model);

        Ok(index)
    }
}
