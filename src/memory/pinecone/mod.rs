// src/memory/pinecone/mod.rs

mod api;
mod index;
mod pinecone;

pub(super) use api::{CreateIndexParameters, Endpoint, PostResponse, QueryParameters, QueryResponse, UpsertDataParameters, Vector, WhoAmIResponse};
pub(super) use index::Index;
pub use pinecone::Pinecone;

use anyhow::{anyhow, Error};
use async_trait::async_trait;
use self::api::VectorMetadata;

use super::{Memory, memory_data::MemoryData};

#[async_trait]
impl Memory for Pinecone {
    async fn add_memory(&mut self, memory: MemoryData) -> Result<(), Error> {

        let id = match self.vector_count {
            Some(count) => count.to_string(),
            None => return Err(anyhow!("Must have vector count so as to have vector id for adding memory."))
        };
        let values = memory.0.0;
        let raw_text = memory.1;
        let metadata = VectorMetadata { raw_text };
        let vector = Vector { id, values, metadata };

        let upsert_data = UpsertDataParameters { vectors: vec![vector] };

        Ok(self.upsert(upsert_data).await?)
    }

    async fn add_memories(&mut self, memories: Vec<MemoryData>) -> Result<(), Error> {
        let mut vectors = vec![];

        for (index, memory) in memories.iter().enumerate() {
            let id = match self.vector_count {
                Some(count) => (count + index as u32).to_string(),
                None => return Err(anyhow!("Must have vector count so as to have vector id for adding memory."))
            };
            let values = memory.0.0.clone();
            let raw_text = memory.1.clone();
            let metadata = VectorMetadata { raw_text };
            let vector = Vector { id, values, metadata };

            vectors.push(vector);
        }

        let upsert_data = UpsertDataParameters { vectors };

        Ok(self.upsert(upsert_data).await?)
    }

    async fn get_similar_memories(&self, related_thought: MemoryData) -> Result<Vec<MemoryData>, Error> {
        let top_k = self.similar_memories_count;
        let include_values = true;
        let include_metadata = true;
        let vector = related_thought.0.0;

        let query = QueryParameters { top_k, include_values, include_metadata, vector };

        Ok(self.query(query).await?)
    }

    async fn initialize(&mut self) -> Result<(), Error> {
        self.initialize().await?;
        Ok(())
    }
}