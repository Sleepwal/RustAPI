use crate::models::{ApiRequest, ApiResponse, HttpMethod};
use std::time::Instant;

pub async fn send_request(request: ApiRequest) -> Result<ApiResponse, String> {
    let client = reqwest::Client::new();
    let start = Instant::now();

    let method = match request.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    };

    let mut req_builder = client.request(method, &request.url);

    // Add headers
    for header in &request.headers {
        if !header.key.is_empty() {
            req_builder = req_builder.header(&header.key, &header.value);
        }
    }

    // Add body for methods that support it
    match request.method {
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
            if !request.body.is_empty() {
                // Check if Content-Type header is already set
                let has_content_type = request.headers.iter().any(|h| h.key.eq_ignore_ascii_case("Content-Type"));
                
                // If not set, try to detect content type
                if !has_content_type {
                    if serde_json::from_str::<serde_json::Value>(&request.body).is_ok() {
                        // Looks like JSON
                        req_builder = req_builder.header("Content-Type", "application/json");
                    } else {
                        // Default to plain text
                        req_builder = req_builder.header("Content-Type", "text/plain");
                    }
                }
                
                req_builder = req_builder.body(request.body.clone());
            }
        }
        _ => {}
    }

    let response = req_builder
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let duration_ms = start.elapsed().as_millis() as u64;
    let status = response.status();
    let status_code = status.as_u16();
    let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

    // Collect headers
    let mut headers = Vec::new();
    for (key, value) in response.headers() {
        headers.push((
            key.to_string(),
            value.to_str().unwrap_or("[binary]").to_string(),
        ));
    }

    // Get body
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // Try to format JSON if possible
    let formatted_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        serde_json::to_string_pretty(&json).unwrap_or(body)
    } else {
        body
    };

    Ok(ApiResponse {
        status: status_code,
        status_text,
        headers,
        body: formatted_body,
        duration_ms,
    })
}
