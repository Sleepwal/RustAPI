//! HTTP 请求发送模块。
//!
//! 使用 `reqwest` 库发送 HTTP 请求并处理响应。
//! 支持自动 Content-Type 检测和 JSON 响应格式化。
//!
//! # 请求处理流程
//!
//! ```text
//! ApiRequest
//!     │
//!     ▼
//! ┌─────────────────────────┐
//! │ 1. 创建 reqwest Client  │
//! │ 2. 映射 HttpMethod      │
//! │ 3. 构建请求构建器       │
//! │ 4. 添加自定义请求头     │
//! │ 5. 自动检测 Content-Type│
//! │ 6. 添加请求体           │
//! │ 7. 发送请求             │
//! └───────────┬─────────────┘
//!             │
//!             ▼
//! ┌─────────────────────────┐
//! │ 8. 计算耗时             │
//! │ 9. 提取状态码和状态文本 │
//! │ 10. 收集响应头          │
//! │ 11. 读取并格式化响应体  │
//! └───────────┬─────────────┘
//!             │
//!             ▼
//!       Result<ApiResponse, String>
//! ```

use crate::models::{ApiError, ApiRequest, ApiResponse, HttpMethod};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

/// 发送 HTTP 请求并返回格式化的响应。
///
/// 本函数是应用的核心网络功能，负责将 `ApiRequest` 转换为实际的 HTTP 请求，
/// 发送后收集响应信息并构造 `ApiResponse`。
///
/// # 参数
///
/// * `client` - 复用的 reqwest HTTP 客户端实例
/// * `request` - 包含方法、URL、请求头和请求体的完整请求描述。
///
/// # 返回值
///
/// * `Ok(ApiResponse)` - 请求成功，返回包含状态码、响应头、格式化响应体和耗时的响应结构体
/// * `Err(String)` - 请求失败，返回描述错误的字符串（网络错误、响应体读取失败等）
pub async fn send_request(client: &Client, request: ApiRequest) -> Result<ApiResponse, ApiError> {
    // 步骤1：记录请求开始时间，用于计算总耗时
    let start = Instant::now();

    // 步骤1.5：创建带超时的请求
    let timeout_duration = Duration::from_secs(request.timeout_secs);

    // 步骤2：将应用层的 HttpMethod 枚举映射为 reqwest 的 Method 类型
    let method = match request.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    };

    // 步骤3：使用方法和 URL 创建请求构建器
    let mut req_builder = client.request(method, &request.url);

    // 步骤4：添加自定义请求头
    // 跳过 key 为空的请求头，避免发送无效头部
    for header in &request.headers {
        if !header.key.is_empty() {
            req_builder = req_builder.header(&header.key, &header.value);
        }
    }

    // 步骤5：为支持请求体的方法添加请求体和 Content-Type
    match request.method {
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
            if !request.body.is_empty() {
                // 检查用户是否已显式设置 Content-Type 请求头
                let has_content_type = request
                    .headers
                    .iter()
                    .any(|h| h.key.eq_ignore_ascii_case("Content-Type"));

                // 如果用户未设置 Content-Type，自动检测并设置
                if !has_content_type {
                    if serde_json::from_str::<serde_json::Value>(&request.body).is_ok() {
                        // 请求体可被解析为有效 JSON，设置 Content-Type 为 application/json
                        req_builder = req_builder.header("Content-Type", "application/json");
                    } else {
                        // 非 JSON 内容，默认使用 text/plain
                        req_builder = req_builder.header("Content-Type", "text/plain");
                    }
                }

                // 将请求体添加到请求构建器
                req_builder = req_builder.body(request.body.clone());
            }
        }
        _ => {
            // GET、DELETE、HEAD、OPTIONS 方法不支持请求体，跳过
        }
    }

    // 步骤6：发送请求，将网络错误转换为用户友好的错误消息
    let response = tokio::time::timeout(timeout_duration, req_builder.send())
        .await
        .map_err(|_| {
            ApiError::NetworkError(format!("Request timed out after {}s", request.timeout_secs))
        })?
        .map_err(|e| ApiError::NetworkError(e.to_string()))?;

    // 步骤7：计算请求总耗时（从发送到接收到响应头）
    let duration_ms = start.elapsed().as_millis() as u64;

    // 步骤8：提取 HTTP 状态信息
    let status = response.status();
    let status_code = status.as_u16();
    // canonical_reason() 返回标准的状态文本（如 200 → "OK"），未知状态码返回 None
    let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

    // 步骤9：收集响应头
    // 二进制头部值（无法转为 UTF-8 字符串）会被替换为 "[binary]"
    let mut headers = Vec::new();
    for (key, value) in response.headers() {
        headers.push((
            key.to_string(),
            value.to_str().unwrap_or("[binary]").to_string(),
        ));
    }

    // 步骤10：读取响应体为文本
    let body = response
        .text()
        .await
        .map_err(|e| ApiError::ResponseReadError(e.to_string()))?;

    // 步骤11：尝试将响应体格式化为美化的 JSON
    // 如果不是有效的 JSON，保留原始文本
    let formatted_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        serde_json::to_string_pretty(&json).unwrap_or(body)
    } else {
        body
    };

    // 步骤12：构造并返回完整的响应结构体
    Ok(ApiResponse {
        status: status_code,
        status_text,
        headers: Arc::new(headers),
        body: formatted_body,
        duration_ms,
    })
}
