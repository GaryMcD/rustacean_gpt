use anyhow::Error;
use async_openai::{types::CreateEmbeddingRequestArgs, Client};

pub async fn get_embedding(text: &str) -> Result<Vec<f32>, Error> {
    let client = Client::new();

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-ada-002") // TODO: 
        .input([text])
        .build()?;

    let response = client.embeddings().create(request).await?;
    let embedding = response.data[0].embedding.clone();

    Ok(embedding)
}