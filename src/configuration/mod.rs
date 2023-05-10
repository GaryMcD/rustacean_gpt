// src/configuration/mod.rs

mod agent;
mod memory;
mod system;

pub use agent::{
    AgentConfiguration,
    open_ai_configuration::OpenAIAgentConfiguration,
};
pub use memory::{
    MemoryConfiguration,
    pinecone_configuration::PineconeMemoryConfiguration
};
pub use system::{
    get_initial_prompt,
    InitialPromptConfiguration,
    SystemConfiguration
};

use anyhow::Error;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;

#[derive(Deserialize)]
pub struct ApplicationConfiguration {
    pub agent: AgentConfiguration,
    pub memory: MemoryConfiguration,
    pub system: SystemConfiguration,
}

pub async fn load_configuration<P: AsRef<Path>>(config_path: P) -> Result<ApplicationConfiguration, Error> {
    let config_data = fs::read_to_string(config_path).await?;
    let config: ApplicationConfiguration = serde_json::from_str(&config_data)?;
    Ok(config)
}