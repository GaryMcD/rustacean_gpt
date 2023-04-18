use anyhow::{anyhow, Error};
use super::endpoint::{CreateIndexParameters, Endpoint, PostResponse, QueryParameters, QueryResponse, UpsertDataParameters, WhoAmIResponse};
use serde_json::Value;
use std::time::Duration;
use std::thread;

pub struct Pinecone {
    api_key: String,
    pub(super) index_name: String,
    pub(super) project_name: String,
    pub(super) region: String,
    pub(super) vector_count: u32,
}

impl Pinecone {
    pub(crate) async fn new(api_key: &str, index_name: &str, region: &str) -> Result<Self, Error> {
        let mut new_pinecone_memory = Self {
            api_key: api_key.to_string(), 
            index_name: index_name.to_string(),
            project_name: "".to_string(),
            region: region.to_string(),
            vector_count: 0,
        };

        match new_pinecone_memory.index_exists().await? {
            true => { println!("Index found by the name of {}.", new_pinecone_memory.index_name);},
            false => {
                println!("No index found by the name of {}.", new_pinecone_memory.index_name);
                _ = new_pinecone_memory.create_index().await?;
                while !(new_pinecone_memory.index_ready().await?) {
                    thread::sleep(Duration::from_millis(500));
                }
                println!("Completed creating index by the name of {}.", new_pinecone_memory.index_name)
            }
        }

        let who_am_i = new_pinecone_memory.who_am_i().await?;
        new_pinecone_memory.project_name = who_am_i.project_name.clone();

        let vector_count = new_pinecone_memory.get_current_vector_count().await?;
        new_pinecone_memory.vector_count = vector_count;

        Ok(new_pinecone_memory)
    }

    async fn create_index(&self) -> Result<Value, Error> {
        let parameters = CreateIndexParameters {
            name: self.index_name.clone(),
            dimension: 1536,
            metric: "cosine".to_string(),
            pods: 1,
            replicas: 1,
            pod_type: "p1.x1".to_string()
        };
    
        let create_index_endpoint = Endpoint::CreateIndex(self.region.clone(), parameters);

        Ok(create_index_endpoint.post(&self.api_key, PostResponse::Text).await?)
    }
    
    async fn get_current_vector_count(&self) -> Result<u32, Error> {
        let raw_vector_count = self.get_index_statistics().await?;
        let converted_vector_count = raw_vector_count["totalVectorCount"].as_u64();
        match converted_vector_count {
            Some(value) => Ok(value as u32),
            None => Err(anyhow!("Unable to convert vector count to u64. Raw Response: {:?}", raw_vector_count))
        }
    }

    async fn get_index_description(&self) -> Result<Value, Error> {
        let get_index_description_endpoint = Endpoint::DescribeIndex(self.region.clone(), self.index_name.clone());
        Ok(get_index_description_endpoint.get(&self.api_key).await?)
    }

    async fn get_index_statistics(&self) -> Result<Value, Error> {
        let get_index_statistics_endpoint = Endpoint::IndexStatistics(self.index_name.clone(), self.project_name.clone(), self.region.clone());
        Ok(get_index_statistics_endpoint.get(&self.api_key).await?)
    }

    async fn index_exists(&self) -> Result<bool, Error> {
        let list_indexes_endpoint = Endpoint::ListIndexes(self.region.clone());
        let indexes_in_region = list_indexes_endpoint.get(&self.api_key).await?;
    
        if let Value::Array(json_array) = indexes_in_region {
            for item in json_array {
                match item {
                    Value::String(string_item) => {
                        if string_item == self.index_name.clone() {
                            return Ok(true)
                        }
                    }
                    _ => ()
                }
            }
        } else {
            return Err(anyhow!("Response from pinecone was not a string array as expected."))
        }
    
        Ok(false)
    }

    pub(super) async fn index_ready(&self) -> Result<bool, Error> {
        let index_description = self.get_index_description().await?;
        let index_ready_status = index_description["status"]["ready"].as_bool().unwrap();
        let index_state = index_description["status"]["state"].as_str().unwrap();
        let initializing = index_state == "Initializing";
        let ready = index_ready_status && !initializing;
        Ok(ready)
    }

    pub(super) async fn query(&self, data: QueryParameters) -> Result<Vec<Vec<f64>>, Error> {
        let query_endpoint = Endpoint::Query(self.index_name.clone(), self.project_name.clone(), self.region.clone(), data.clone());
        let query = query_endpoint.post(&self.api_key, PostResponse::Json).await?;

        let response: QueryResponse = serde_json::from_value(query).unwrap();
        let results = response.matches.iter().map(|single_match| single_match.values.clone()).collect();

        Ok(results)
    }

    pub(super) async fn upsert(&mut self, data: UpsertDataParameters) -> Result<(), Error> {
        let upsert_endpoint = Endpoint::Upsert(self.index_name.clone(), self.project_name.clone(), self.region.clone(), data.clone());
        _ = upsert_endpoint.post(&self.api_key, PostResponse::Text).await?;

        self.vector_count = self.vector_count + (data.vectors.len() as u32);
        Ok(())
    }

    pub(super) fn vector_count(&self) -> u32 {
        self.vector_count
    }

    async fn who_am_i(&self) -> Result<WhoAmIResponse, Error> {
        let who_am_i_endpoint = Endpoint::WhoAmI(self.region.clone());
        let response_as_value = who_am_i_endpoint.get(&self.api_key).await?;
        let response: WhoAmIResponse = serde_json::from_value(response_as_value)?;
        Ok(response)
    }
}