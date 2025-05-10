use anyhow::Result;
use rig::{
    embeddings::EmbeddingsBuilder,
    providers::cohere,
    tool::ToolSet,
    vector_store::{VectorStoreIndexDyn, in_memory_store::InMemoryVectorStore},
};

pub struct Embedding;

impl Embedding {
    pub async fn build_tool_index(toolset: &ToolSet) -> Result<impl VectorStoreIndexDyn + 'static> {
        let cohere_client = cohere::Client::from_env();
        let embedding_model =
            cohere_client.embedding_model(cohere::EMBED_MULTILINGUAL_V3, "classification");
        let embeddings = EmbeddingsBuilder::new(embedding_model.clone())
            .documents(toolset.schemas()?)?
            .build()
            .await?;
        let store = InMemoryVectorStore::from_documents_with_id_f(embeddings, |f| f.name.clone());
        let index = store.index(embedding_model);

        Ok(index)
    }
}
