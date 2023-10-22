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

fn method_to_uppercase(method: &HttpMethod) -> String {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
        HttpMethod::Put => "PUT",
        HttpMethod::Delete => "DELETE",
        _ => "UNKNOWN",
    }
    .to_string()
    .to_uppercase()
}

pub async fn request<T>(url: &str, data: &T, method: HttpMethod) -> Result<Response, Error>
where
    T: Serialize + Deserialize<'static> + Debug,
{
    let request_body = serde_json::to_string(data).unwrap();
    let request_size = request_body.len();

    log::info!(
        "HTTP {:?} {}. Size: {} bytes",
        method_to_uppercase(&method),
        url,
        request_size
    );

    let client = Client::builder()
        // .timeout(Duration::from_secs(60))
        // .user_agent("rs-algo-scanner")
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
