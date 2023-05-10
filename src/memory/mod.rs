// src/memory/mod.rs

mod memory_data;
mod pinecone;

pub use memory_data::{Embedding, MemoryData};
pub use pinecone::Pinecone;

use anyhow::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Memory {
    async fn add_memory(&mut self, memory: MemoryData) -> Result<(), Error>;
    async fn add_memories(&mut self, memories: Vec<MemoryData>) -> Result<(), Error>;
    async fn get_similar_memories(&self, related_thought: MemoryData) -> Result<Vec<MemoryData>, Error>;
    async fn initialize(&mut self) -> Result<(), Error>;
}