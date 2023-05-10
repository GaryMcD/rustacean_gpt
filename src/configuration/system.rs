// src/configuration/system.rs

use anyhow::Error;
use serde::{Deserialize, Serialize};
use tokio::fs;

#[derive(Deserialize, Serialize)]
pub struct SystemConfiguration {
    pub conversation_file_path: String,
    pub initial_prompt: InitialPromptConfiguration,
    pub working_directory: String
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
            Ok(value.to_owned())
        },
    }
}