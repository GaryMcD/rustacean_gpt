// src/agent/openai/mod.rs

mod agent;
pub use agent::GPT;

use async_openai::types::{ChatCompletionRequestMessage, Role};
use crate::{memory::MemoryData, system::{Chat, Whom}};
use serde::{Deserialize, Serialize};
use strum_macros;
use super::Agent;
use tiktoken_rs::cl100k_base;

// Used for configuration
#[derive(Clone, Deserialize, Serialize, strum_macros::EnumProperty)]
pub enum OpenAiModel {
    #[strum(props(Name = "gpt-3.5-turbo", TokenLimit = "4096"))]
    GPT3_5Turbo,

    #[strum(props(Name = "gpt-4", TokenLimit = "8192"))]
    GPT4,

    #[strum(props(Name = "gpt-4-0314", TokenLimit = "8192"))]
    GPT4_0314
}

#[derive(Clone, Deserialize, Serialize, strum_macros::EnumProperty)]
pub enum OpenAiEmbeddingModel {
    #[strum(props(Name = "text-embedding-ada-002"))]
    Ada002
}

pub(super) fn chat_to_chat_completion_request_message(chat: &Chat) -> ChatCompletionRequestMessage {
    let role = match chat.whom {
        Whom::Agent => Role::Assistant,
        Whom::System => Role::System,
        Whom::User => Role::User
    };
    let content = chat.text.clone();
    let name = None;

    ChatCompletionRequestMessage { role, content, name }
}

pub(super) fn chat_completion_request_message_token_estimate(chat_completion_request_message: &ChatCompletionRequestMessage) -> u16 {
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(&chat_completion_request_message.content);
    tokens.len() as u16
}

pub(super) fn memory_to_chat_completion_request_message(memory: &MemoryData) -> ChatCompletionRequestMessage {
    let content = format!("Related Memory: {}", memory.1);
    let role = Role::System;
    let name = None;

    ChatCompletionRequestMessage { role, content, name }
}