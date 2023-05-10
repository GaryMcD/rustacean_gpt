// src/agent/openai/agent.rs

use anyhow::Error;
use async_trait::async_trait;
use async_openai::{Client, types::{ChatCompletionRequestMessage, CreateChatCompletionRequestArgs, CreateEmbeddingRequestArgs}};
use crate::{memory::{Embedding, MemoryData}, system::{Conversation, Chat, Whom}, configuration::OpenAIAgentConfiguration};
use strum::EnumProperty;
use super::{Agent, OpenAiEmbeddingModel, OpenAiModel, chat_to_chat_completion_request_message, chat_completion_request_message_token_estimate, memory_to_chat_completion_request_message};

pub struct GPT {
    api_key: String, 
    embedding_model: OpenAiEmbeddingModel, 
    model: OpenAiModel,
    tokens_reserved_for_history: u16,
    tokens_reserved_for_memories: u16,
}

impl GPT {
    fn gather_chat_history(&self, conversation: &Conversation) -> Vec<ChatCompletionRequestMessage> {
        let mut chat_history = vec![];

        // First two messages must be included in history as they are prompt and user objective.
        let initial_prompt = chat_to_chat_completion_request_message(&conversation.conversation[0]);
        let user_objective = chat_to_chat_completion_request_message(&conversation.conversation[1]);
        let initial_prompt_tokens = chat_completion_request_message_token_estimate(&initial_prompt);
        let user_objective_tokens = chat_completion_request_message_token_estimate(&user_objective);
        chat_history.push(initial_prompt);
        chat_history.push(user_objective);

        let mut history_tokens_current = initial_prompt_tokens + user_objective_tokens;

        for (index, chat) in conversation.conversation.iter().enumerate().rev() {
            if index <= 1 {
                break
            }

            let chat_completion_request_message = chat_to_chat_completion_request_message(chat);
            let token_estimate = chat_completion_request_message_token_estimate(&chat_completion_request_message);

            let potential_tokens = token_estimate + history_tokens_current;
            if potential_tokens > self.tokens_reserved_for_history {
                break;
            } else {
                chat_history.insert(2, chat_completion_request_message);
                history_tokens_current = potential_tokens;
            }
        }
        chat_history
    }

    async fn get_embedding(&self, string_to_convert: &str) -> Result<Embedding, Error> {        
        let client = Client::new().with_api_key(&self.api_key);
    
        let request = CreateEmbeddingRequestArgs::default()
            .model(self.embedding_model.get_str("Name").unwrap())
            .input([string_to_convert])
            .build()?;
    
        let response = client.embeddings().create(request).await?;
        let embedding = response.data[0].embedding.clone();
    
        Ok(Embedding(embedding))
    }

    async fn get_ai_response(&self, combined_history: Vec<ChatCompletionRequestMessage>) -> Result<Chat, Error> {
        let client = Client::new().with_api_key(&self.api_key);

        let request = CreateChatCompletionRequestArgs::default()
            .model(self.model.get_str("Name").unwrap())
            .messages(combined_history)
            .build()?;

        let ai_response = client.chat().create(request).await?;

        Ok(Chat { text: ai_response.choices[0].message.content.clone(), whom: Whom::Agent})
    }

    pub fn new(configuration: &OpenAIAgentConfiguration) -> Box<dyn Agent> {
        let api_key = configuration.api_key.clone();
        let embedding_model = configuration.embedding_model.clone();
        let model = configuration.model.clone();
        let tokens_reserved_for_history = configuration.tokens_reserved_for_history;
        let tokens_reserved_for_memories = configuration.tokens_reserved_for_memories;

        Box::new(Self { api_key, embedding_model, model, tokens_reserved_for_history, tokens_reserved_for_memories})
    }

    fn prune_memories_to_limit(&self, memories: &Vec<MemoryData>) -> Vec<ChatCompletionRequestMessage> {
        let mut memories_as_chat = vec![];
        let mut tokens_current = 0;

        for memory in memories {
            let chat_completion_request_message = memory_to_chat_completion_request_message(memory);
            let tokens = chat_completion_request_message_token_estimate(&chat_completion_request_message);

            let potential_tokens = tokens_current + tokens;
            if potential_tokens > self.tokens_reserved_for_memories {
                continue;
            } else {
                memories_as_chat.push(chat_completion_request_message);
                tokens_current = potential_tokens;
            }
        }

        memories_as_chat
    }
}

#[async_trait]
impl Agent for GPT {
    async fn get_string_embedding(&self, string_to_convert: &str) -> Result<Embedding, Error> {
        self.get_embedding(string_to_convert).await
    }

    async fn initialize(&mut self) -> Result<(), Error> {
        Ok(())
    }

    async fn respond(&self, conversation: &Conversation, related_memories: &Vec<MemoryData>) -> Result<Chat, Error> {
        let mut chat_history = self.gather_chat_history(conversation);
        let pruned_memories = self.prune_memories_to_limit(related_memories);
        chat_history.extend(pruned_memories);
        self.get_ai_response(chat_history).await
    }
}