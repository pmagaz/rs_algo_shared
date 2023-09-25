use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Post,
    Put,
    Get,
    Patch,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    Json,
    String,
}

pub async fn request<T>(url: &str, data: &T, method: HttpMethod) -> Result<Response, Error>
where
    T: Serialize + Deserialize<'static> + Debug,
{
    let request_body = serde_json::to_string(data).unwrap();
    let request_size = request_body.len();

    log::info!(
        "HTTP {:?} request to {}. Size: {} bytes",
        method , url, request_size
    );

    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent("rs-algo-scanner")
        .build()?;

    let result = match method {
        HttpMethod::Post => client.post(url),
        HttpMethod::Put => client.put(url),
        HttpMethod::Get => client.get(url),
        HttpMethod::Patch => client.patch(url),
        HttpMethod::Delete => client.delete(url),
    };

    result
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_body)
        .send()
        .await
}