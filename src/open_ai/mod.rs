mod ai_response;
pub use ai_response::AiResponse;

use anyhow::Error;
use async_openai::{types::CreateEmbeddingRequestArgs, Client};
use tiktoken_rs::cl100k_base;

pub async fn get_embedding(text: &str) -> Result<Vec<f32>, Error> {
    
    let client = Client::new();

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-ada-002") // TODO: make this part of configuration
        .input([text])
        .build()?;

    let response = client.embeddings().create(request).await?;
    let embedding = response.data[0].embedding.clone();

    Ok(embedding)
}

pub fn get_token_estimate(text: &str) -> usize {
    let bpe = cl100k_base().unwrap();
    let tokens = bpe.encode_with_special_tokens(text);
    tokens.len()
}