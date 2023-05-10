// src/configuration.rs

use anyhow::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

use crate::agent::{OpenAiEmbeddingModel, OpenAiModel};

#[derive(Deserialize, Serialize)]
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

#[derive(Deserialize, Serialize)]
pub enum InitialPromptConfiguration {
    File { path: String },
    Raw { value: String },
}

pub async fn get_initial_prompt(config: &InitialPromptConfiguration) -> Result<String, Error> {
    match config {
        InitialPromptConfiguration::File { path } => {
            Ok(fs::read_to_string(path).await?)
        },
        InitialPromptConfiguration::Raw { value } => {
            Ok(value.clone())
        },
    }
}