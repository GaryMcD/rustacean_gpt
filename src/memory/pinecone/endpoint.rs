use anyhow::{anyhow, Error};
use reqwest::header::{ACCEPT, CONTENT_TYPE, HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(super) enum Endpoint {
    CreateIndex(String, CreateIndexParameters),
    DescribeIndex(String, String),
    IndexStatistics(String, String, String),
    ListIndexes(String),
    Query(String, String, String, QueryParameters),
    Upsert(String, String, String, UpsertDataParameters),
    WhoAmI(String)
}

impl Endpoint {
    pub(super) async fn get(&self, api_key: &str) -> Result<Value, Error> {
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
            Self::Query(index_name, project_name, region, _) => format!("https://{}-{}.svc.{}.pinecone.io/query", index_name, project_name, region),
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

            Self::Query(_,_,_,_) => 
            (
                Some(HeaderValue::from_static("application/json")),
                Some(HeaderValue::from_static("application/json"))
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

    pub(super) async fn post(&self, api_key: &str, response_type_desired: PostResponse) -> Result<Value, Error> {
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

        let response = match response_type_desired {
            PostResponse::Json => response.json().await?,
            PostResponse::Text => serde_json::Value::String(response.text().await?)
        };

        Ok(response)
    }
}


pub(super) enum PostResponse {
    Json,
    Text
}

#[derive(Deserialize, Serialize)]
pub(super) struct CreateIndexParameters {
    pub(super) name: String,
    pub(super) dimension: u32,
    pub(super) metric: String, // TODO: Make enum
    pub(super) pods: u32,
    pub(super) replicas: u32,
    pub(super) pod_type: String // TODO: Make enum
}

#[derive(Deserialize, Serialize)]
pub(super) struct QueryMatch {
    id: String,
    score: f64,
    pub(super) values: Vec<f64>,
    sparseValues: QuerySparseValue
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct QueryParameters {
    pub(super) topK: u8,
    pub(super) includeValues: bool,
    pub(super) includeMetadata: bool,
    pub(super) vector: Vec<f64>
}

#[derive(Deserialize, Serialize)]
pub(super) struct QueryResponse {
    pub(super) matches: Vec<QueryMatch>,
    namespace: String
}

#[derive(Deserialize, Serialize)]
pub(super) struct QuerySparseValue {
    indices: Vec<u64>,
    values: Vec<f64>
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct UpsertDataParameters {
    pub(super) vectors: Vec<Vector>
}

#[derive(Clone, Deserialize, Serialize)]
pub(super) struct Vector {
    pub(super) id: String,
    pub(super) values: Vec<f64>
}

#[derive(Deserialize, Serialize)]
pub(super) struct WhoAmIResponse {
    pub(super) project_name: String,
    user_label: String,
    user_name: String
}