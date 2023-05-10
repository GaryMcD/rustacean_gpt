use anyhow::Error;
use serde::{Deserialize, Serialize};
use serde_json::Result as SerdeResult;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::agent::{Agent, Response};
use crate::memory::MemoryData;

pub async fn add_chat_to_conversation(conversation_file_path: &str, chat: Chat) -> Result<Conversation, Error> {
    let mut conversation = match conversation(conversation_file_path).await? {
        Some(existing_conversation) => existing_conversation,
        None => Conversation {
            conversation: Vec::new(),
        },
    };

    conversation.conversation.push(chat);

    let serialized_conversation = serde_json::to_string(&conversation)?;
    let mut file = File::create(conversation_file_path).await?;
    file.write_all(serialized_conversation.as_bytes()).await?;

    Ok(conversation)
}

pub async fn add_chats_to_conversation(conversation_file_path: &str, chats: Vec<Chat>) -> Result<Conversation, Error> {
    let mut conversation = match conversation(conversation_file_path).await? {
        Some(existing_conversation) => existing_conversation,
        None => Conversation {
            conversation: Vec::new(),
        },
    };

    conversation.conversation.extend(chats);

    let serialized_conversation = serde_json::to_string(&conversation)?;
    let mut file = File::create(conversation_file_path).await?;
    file.write_all(serialized_conversation.as_bytes()).await?;

    Ok(conversation)
}

pub async fn conversation(conversation_file_path: &str) -> Result<Option<Conversation>, Error> {
    let mut contents = String::new();
    
    match File::open(conversation_file_path).await {
        Ok(mut file) => {
            file.read_to_string(&mut contents).await?;
            if contents.is_empty() {
                Ok(None)
            } else {
                let conversation: Conversation = serde_json::from_str(&contents)?;
                Ok(Some(conversation))
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(Error::from(e)),
    }
}

#[derive(Deserialize, Serialize)]
pub struct Conversation {
    pub conversation: Vec<Chat>,
}

impl Conversation {
    pub fn latest_chat_whom(&self) -> Option<Whom> {
        match self.conversation.last() {
            Some(chat) => Some(chat.whom),
            None => None
        }
    }

    pub fn second_to_last_chat(&self) -> Option<Chat> {
        if self.conversation.len() > 1 {
            Some(self.conversation[self.conversation.len() - 2].clone())
        } else {
            None
        }
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Chat {
    pub text: String,
    pub whom: Whom,
}

impl Chat {
    pub async fn as_memory_data(&self, agent: &Box<dyn Agent>) -> Result<MemoryData, Error> {
        let embedding = agent.get_string_embedding(&self.text).await?;
        Ok(MemoryData(embedding, self.text.clone()))
    }

    pub fn parse(&self) -> Result<Response, (String, Error)> {
        let parsed_response: SerdeResult<Response> = serde_json::from_str(&self.text);

        match parsed_response {
            Ok(parsed_response) => Ok(parsed_response),
            Err(parsing_error) => Err((self.text.clone(), Error::from(parsing_error))),
        }
    }
}

#[derive(Clone, Copy, Deserialize, Serialize)]
pub enum Whom {
    Agent,
    System,
    User,
}
