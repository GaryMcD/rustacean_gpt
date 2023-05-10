// src/agent/mod.rs

pub mod openai;
mod response;

pub use response::Response;

use async_trait::async_trait;
use anyhow::Error;
use crate::{memory::{Embedding, MemoryData}, system::{Conversation, Chat}};

#[async_trait]
pub trait Agent {
    async fn get_string_embedding(&self, string_to_convert: &str) -> Result<Embedding, Error>;
    async fn initialize(&mut self) -> Result<(), Error>;
    async fn respond(&self, conversation: &Conversation, related_memories: &Vec<MemoryData>) -> Result<Chat, Error>;
}