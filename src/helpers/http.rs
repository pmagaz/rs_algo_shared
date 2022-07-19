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
    for<'de> T: Serialize + Deserialize<'de> + Debug,
{
    println!("[HTTP] {:?} request to {}", method, url);

    let result = match method {
        HttpMethod::Post => Client::builder().build()?.post(url),
        HttpMethod::Put => Client::builder().build()?.put(url),
        HttpMethod::Get => Client::builder().build()?.get(url),
        HttpMethod::Patch => Client::builder().build()?.patch(url),
        HttpMethod::Delete => Client::builder().build()?.delete(url),
    };
    let response = result.json(&data).send().await;
    println!("[HTTP] response {:?}", &response);

    response
}
