use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

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
    // Serialize the data to JSON and calculate the size of the request body
    let request_body = serde_json::to_string(data).unwrap();
    let request_size = request_body.len();

    println!(
        "[HTTP] {:?} request to {} with request size: {} bytes",
        method, url, request_size
    );

    let client = Client::new();
    let result = match method {
        HttpMethod::Post => Client::builder().build()?.post(url),
        HttpMethod::Put => Client::builder().build()?.put(url),
        HttpMethod::Get => Client::builder().build()?.get(url),
        HttpMethod::Patch => Client::builder().build()?.patch(url),
        HttpMethod::Delete => Client::builder().build()?.delete(url),
    };

    result
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(request_body)
        .send()
        .await
}





