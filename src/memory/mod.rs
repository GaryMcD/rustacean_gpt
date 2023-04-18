mod pinecone;
pub use pinecone::Pinecone;

use anyhow::Error;
use async_trait::async_trait;

#[async_trait]
pub trait Memory {
    async fn add_memory(&mut self, memory: Vec<f64>) -> Result<(), Error>;
    async fn add_memories(&mut self, memories: Vec<Vec<f64>>) -> Result<(), Error>;
    async fn get_similar_memories(&self, related_thought: Vec<f64>) -> Result<Vec<Vec<f64>>, Error>;
    fn print(&self);
    async fn ready(&self) -> Result<bool, Error>;
}