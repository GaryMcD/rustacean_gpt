// src/memory/pinecone/index.rs

use anyhow::{anyhow, Error};
use serde_json::Value;
use super::{CreateIndexParameters, Endpoint, PostResponse};
use tokio::time::{sleep, Duration};

pub struct Index {
    pub(super) api_key: String,
    pub(super) index_name: String,
    pub(super) region: String
}

impl Index {
    pub async fn create(&self) -> Result<Value, Error> {
        let parameters = CreateIndexParameters {
            name: self.index_name.clone(),
            dimension: 1536,
            metric: "cosine".to_string(),
            pods: 1,
            replicas: 1,
            pod_type: "p1.x1".to_string()
        };
    
        let create_index_endpoint = Endpoint::CreateIndex(self.region.clone(), parameters);

        Ok(create_index_endpoint.post(&self.api_key, &PostResponse::Text).await?)
    }

    pub async fn exists(&self) -> Result<bool, Error> {
        let list_indexes_endpoint = Endpoint::ListIndexes(self.region.clone());
        let indexes_in_region = list_indexes_endpoint.get(&self.api_key).await?;
    
        if let Value::Array(json_array) = indexes_in_region {
            for item in json_array {
                match item {
                    Value::String(string_item) => {
                        if string_item.as_str() == self.index_name {
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

    async fn get_description(&self) -> Result<Value, Error> {
        let get_index_description_endpoint = Endpoint::DescribeIndex(self.region.clone(), self.index_name.clone());
        Ok(get_index_description_endpoint.get(&self.api_key).await?)
    }

    pub async fn get_statistics(&self, project_name: &str) -> Result<Value, Error> {
        let get_index_statistics_endpoint = Endpoint::IndexStatistics(self.index_name.clone(), project_name.to_string(), self.region.clone());
        Ok(get_index_statistics_endpoint.get(&self.api_key).await?)
    }

    async fn ready(&self) -> Result<bool, Error> {
        let description = self.get_description().await?;
        let ready_status = description["status"]["ready"].as_bool().unwrap();
        let state = description["status"]["state"].as_str().unwrap();
        let initializing = state == "Initializing";
        let ready = ready_status && !initializing;
        Ok(ready)
    }

    pub async fn wait_until_ready(&self) -> Result<(), Error> {
        loop {
            if self.ready().await? {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
    }
}