use anyhow::{anyhow, Error};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue, };
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use std::thread;
use termion::{color, style};

pub struct Pinecone {
    api_key: String,
    index_name: String,
    project_name: String,
    region: String,
    vector_count: u32,
}

impl Pinecone {
    pub async fn new(api_key: &str, index_name: &str, region: &str) -> Result<Self, Error> {
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

    async fn create_index(&self) -> Result<String, Error> {
        let parameters =  CreateIndexParameters {
            name: self.index_name.clone(),
            dimension: 1536,
            metric: "cosine".to_string(),
            pods: 1,
            replicas: 1,
            pod_type: "p1.x1".to_string()
        };
    
        let create_index_endpoint = Endpoint::CreateIndex(self.region.clone(), parameters);

        Ok(create_index_endpoint.post(&self.api_key).await?)
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

    async fn index_ready(&self) -> Result<bool, Error> {
        let index_description = self.get_index_description().await?;
        let index_ready_status = index_description["status"]["ready"].as_bool().unwrap();
        let index_state = index_description["status"]["state"].as_str().unwrap();
        let initializing = index_state == "Initializing";
        let ready = index_ready_status && !initializing;
        Ok(ready)
    }

    pub fn print(&self) {
        println!("{}{}{}Index{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.index_name, color::Fg(color::Reset));
        println!("{}{}{}Project{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.project_name, color::Fg(color::Reset));
        println!("{}{}{}Region{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.region, color::Fg(color::Reset));
        println!("{}{}{}Vector Count{}: {}{}{}", style::Underline, style::Bold, color::Fg(color::Blue), style::Reset, color::Fg(color::LightBlue), self.vector_count, color::Fg(color::Reset));
    }

    pub async fn upsert(&mut self, data: UpsertDataParameters) -> Result<(), Error> {
        let upsert_endpoint = Endpoint::Upsert(self.index_name.clone(), self.project_name.clone(), self.region.clone(), data.clone());
        _ = upsert_endpoint.post(&self.api_key).await?;

        self.vector_count = self.vector_count + (data.vectors.len() as u32);
        Ok(())
    }

    pub fn vector_count(&self) -> u32 {
        self.vector_count
    }

    async fn who_am_i(&self) -> Result<WhoAmIResponse, Error> {
        let who_am_i_endpoint = Endpoint::WhoAmI(self.region.clone());
        let response_as_value = who_am_i_endpoint.get(&self.api_key).await?;
        let response: WhoAmIResponse = serde_json::from_value(response_as_value)?;
        Ok(response)
    }
}

enum Endpoint {
    CreateIndex(String, CreateIndexParameters),
    DescribeIndex(String, String),
    IndexStatistics(String, String, String),
    ListIndexes(String),
    Upsert(String, String, String, UpsertDataParameters),
    WhoAmI(String)
}

impl Endpoint {
    async fn get(&self, api_key: &str) -> Result<Value, Error> {
        let headers = self.get_headers(api_key);
        let url = self.get_endpoint_url();
        let client = reqwest::Client::new();

        let response = client
            .get(url)
            .headers(headers)
            .send()
            .await?;

        let json: Value = response.json().await?;
        Ok(json)
    }

    fn get_endpoint_url(&self) -> String {
        match self {
            Self::CreateIndex(region, _) => format!("https://controller.{}.pinecone.io/databases", region),
            Self::DescribeIndex(region, index_name) => format!("https://controller.{}.pinecone.io/databases/{}", region, index_name),
            Self::IndexStatistics(index_name, project_name, region) => format!("https://{}-{}.svc.{}.pinecone.io/describe_index_stats", index_name, project_name, region),
            Self::ListIndexes(region) => format!("https://controller.{}.pinecone.io/databases", region),
            Self::Upsert(index_name, project_name, region, _) => format!("https://{}-{}.svc.{}.pinecone.io/vectors/upsert", index_name, project_name, region),
            Self::WhoAmI(region) => format!("https://controller.{}.pinecone.io/actions/whoami", region)
        }
    }

    fn get_headers(&self, api_key: &str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Api-Key", HeaderValue::from_str(api_key).unwrap());

        let (accept_header, content_type_header) = match self {
            Self::CreateIndex(_,_) | Self::IndexStatistics(_,_,_) | Self::Upsert(_,_,_,_) => 
            (
                Some(HeaderValue::from_static("text/plain")),
                Some(HeaderValue::from_static("application/json"))
            ),

            Self::DescribeIndex(_,_) => 
            (
                Some(HeaderValue::from_static("application/json")),
                None
            ),

            Self::ListIndexes(_) =>
            (
                Some(HeaderValue::from_static("application/json; charset=utf-8")),
                None
            ),

            Self::WhoAmI(_) => (None, None)
        };

        match accept_header {
            Some(accept_header) => { headers.insert(ACCEPT, accept_header);}
            None => ()
        }

        match content_type_header {
            Some(content_type) => { headers.insert(CONTENT_TYPE, content_type);}
            None => ()
        }

        headers
    }

    async fn post(&self, api_key: &str) -> Result<String, Error> {
        let headers = self.get_headers(api_key);
        let url = self.get_endpoint_url();
        let client = reqwest::Client::new();
    
        let data = match self {
            Self::CreateIndex(_, parameters) => serde_json::to_string(parameters)?,
            Self::Upsert(_, _, _, parameters) => serde_json::to_string(parameters)?,
            _ => return Err(anyhow!("Cannot post to this endpoint."))
        };

        let response = client.post(url)
            .headers(headers)
            .body(data)
            .send()
            .await?;

        Ok(response.text().await?)
    }
}

#[derive(Deserialize, Serialize)]
struct CreateIndexParameters {
    name: String,
    dimension: u32,
    metric: String, // TODO: Make enum
    pods: u32,
    replicas: u32,
    pod_type: String // TODO: Make enum
}

#[derive(Clone, Deserialize, Serialize)]
pub struct UpsertDataParameters {
    vectors: Vec<Vector>
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Vector {
    id: String,
    values: Vec<f64>
}

#[derive(Deserialize, Serialize)]
struct WhoAmIResponse {
    project_name: String,
    user_label: String,
    user_name: String
}