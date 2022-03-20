use reqwest::{Client, Error, Response};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpMethod {
    Post,
    Put,
    Get,
    Patch,
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
    let result = match method {
        HttpMethod::Post => Client::builder().build()?.post(url),
        HttpMethod::Put => Client::builder().build()?.put(url),
        HttpMethod::Get => Client::builder().build()?.get(url),
        HttpMethod::Patch => Client::builder().build()?.patch(url),
    };

    result.json(&data).send().await
}
