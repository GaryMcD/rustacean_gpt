// src/configuration/memory.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub enum MemoryConfiguration {
    PineconeConfiguration(pinecone_configuration::PineconeMemoryConfiguration)
}

pub mod pinecone_configuration {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct PineconeMemoryConfiguration {
        pub api_key: String, 
        pub index_name: String, 
        pub region: String, 
        pub similar_memories_count: u8 
    }
}