// src/configuration/agent.rs

use serde::Deserialize;

#[derive(Deserialize)]
pub enum AgentConfiguration {
    OpenAIAgentConfiguration(open_ai_configuration::OpenAIAgentConfiguration)
}

pub mod open_ai_configuration {
    use serde::Deserialize;
    use crate::agent::openai::{OpenAiEmbeddingModel, OpenAiModel};

    #[derive(Deserialize)]
    pub struct OpenAIAgentConfiguration {
        pub api_key: String, 
        pub embedding_model: OpenAiEmbeddingModel, 
        pub model: OpenAiModel,
        pub tokens_reserved_for_history: u16,
        pub tokens_reserved_for_memories: u16
    }
}
