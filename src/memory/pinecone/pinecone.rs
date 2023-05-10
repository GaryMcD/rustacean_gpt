// src/memory/pinecone/pinecone.rs

use anyhow::{anyhow, Error};
use crate::{configuration::PineconeMemoryConfiguration, memory::Memory};
use super::{Endpoint, Index, PostResponse, QueryParameters, QueryResponse, UpsertDataParameters, WhoAmIResponse};
use super::super::{Embedding, MemoryData};

pub struct Pinecone {
    // From Configuration
    api_key: String,
    index_name: String,
    region: String,
    pub(super) similar_memories_count: u8,

    // Post initialization
    index: Option<Index>,
    project_name: Option<String>,
    pub(super) vector_count: Option<u32>,
}

impl Pinecone {
    async fn get_project_name(&self) -> Result<String, Error> {
        let who_am_i_endpoint = Endpoint::WhoAmI(self.region.clone());
        let response_as_value = who_am_i_endpoint.get(&self.api_key).await?;
        let response: WhoAmIResponse = serde_json::from_value(response_as_value)?;
        Ok(response.project_name.clone())
    }

    async fn get_vector_count(&self) -> Result<u32, Error> {
        let index = match &self.index {
            Some(index) => index,
            None => return Err(anyhow!("Index should be created and initialized before requesting vector count"))
        };

        let raw_vector_count = match &self.project_name {
            Some(project_name) => index.get_statistics(&project_name).await?,
            None => return Err(anyhow!("Cannot get vector count without first getting project name."))
        };

        let converted_vector_count = raw_vector_count["totalVectorCount"].as_u64();
        match converted_vector_count {
            Some(value) => Ok(value as u32),
            None => Err(anyhow!("Unable to convert vector count to u64. Raw Response: {:?}", raw_vector_count))
        }
    }

    pub(super) async fn initialize(&mut self) -> Result<(), Error> {
        let index = Index { api_key: self.api_key.clone(), index_name: self.index_name.clone(), region: self.region.clone() };
        match index.exists().await? {
            true => {()},
            false => {index.create().await?;}
        }

        self.index = Some(index);

        self.project_name = Some(self.get_project_name().await?);
        self.vector_count = Some(self.get_vector_count().await?);

        self.index.as_ref().unwrap().wait_until_ready().await?;

        Ok(())
    }

    pub fn new(pinecone_configuration: &PineconeMemoryConfiguration) -> Box<dyn Memory> {
        let api_key = pinecone_configuration.api_key.clone();
        let index_name = pinecone_configuration.index_name.clone();
        let region = pinecone_configuration.region.clone();
        let similar_memories_count = pinecone_configuration.similar_memories_count;
        let index = None;
        let project_name = None;
        let vector_count = Some(0);

        Box::new(Self { api_key, index_name, region, similar_memories_count, index, project_name, vector_count})
    }

    pub(super) async fn query(&self, data: QueryParameters) -> Result<Vec<MemoryData>, Error> {
        let project_name = match &self.project_name {
            Some(project_name) => project_name,
            None => return Err(anyhow!("Cannot upsert without first getting project name."))
        };

        let query_endpoint = Endpoint::Query(self.index_name.clone(), project_name.clone(), self.region.clone(), data.clone());
        let query = query_endpoint.post(&self.api_key, &PostResponse::Json).await?;

        let response: QueryResponse = serde_json::from_value(query).unwrap();
        let results = response.matches.iter().map(|single_match| MemoryData(Embedding(single_match.values.clone()), single_match.metadata.raw_text.clone())).collect();

        Ok(results)
    }

    pub(super) async fn upsert(&mut self, data: UpsertDataParameters) -> Result<(), Error> {
        let project_name = match &self.project_name {
            Some(project_name) => project_name,
            None => return Err(anyhow!("Cannot upsert without first getting project name."))
        };

        let upsert_endpoint = Endpoint::Upsert(self.index_name.clone(), project_name.to_string(), self.region.clone(), data.clone());
        _ = upsert_endpoint.post(&self.api_key, &PostResponse::Text).await?;

        self.vector_count = Some(self.vector_count.unwrap() + (data.vectors.len() as u32));
        Ok(())
    }
}