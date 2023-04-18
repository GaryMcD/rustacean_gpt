mod endpoint;
mod pinecone;
pub use pinecone::Pinecone;

use anyhow::Error;
use async_trait::async_trait;
use endpoint::{QueryParameters, UpsertDataParameters, Vector};
use termion::{color, style};

#[async_trait]
impl super::Memory for Pinecone {
    async fn add_memory(&mut self, memory: Vec<f64>) -> Result<(), Error> {
        let vector_id = self.vector_count().to_string();
        let vector = Vector {
            id: vector_id,
            values: memory
        };

        let upsert_data = UpsertDataParameters {
            vectors: vec![vector]
        };

        Ok(self.upsert(upsert_data).await?)
    }

    async fn add_memories(&mut self, memories: Vec<Vec<f64>>) -> Result<(), Error> {
        let mut vectors = vec![];

        for (index, memory) in memories.iter().enumerate() {
            let vector_id = (self.vector_count() + index as u32).to_string();
            let vector = Vector {
                id: vector_id,
                values: memory.clone()
            };

            vectors.push(vector);
        }

        let upsert_data = UpsertDataParameters { vectors };

        Ok(self.upsert(upsert_data).await?)
    }
    
    async fn get_similar_memories(&self, related_thought: Vec<f64>) -> Result<Vec<Vec<f64>>, Error> {
        let topK = 10; // TODO: make part of configuration
        let includeValues = true;
        let includeMetadata = false;
        let vector = related_thought;

        let query = QueryParameters { topK, includeValues, includeMetadata, vector };

        Ok(self.query(query).await?)
    }

    fn print(&self) {
        println!("{}{}{}Index{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.index_name, color::Fg(color::Reset));
        println!("{}{}{}Project{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.project_name, color::Fg(color::Reset));
        println!("{}{}{}Region{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.region, color::Fg(color::Reset));
        println!("{}{}{}Vector Count{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.vector_count, color::Fg(color::Reset));
    }

    async fn ready(&self) -> Result<bool, Error> {
        Ok(self.index_ready().await?)
    }
}