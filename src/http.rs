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

use crate::models::{ApiRequest, ApiResponse, HttpMethod};
use std::time::Instant;

/// 发送 HTTP 请求并返回格式化的响应。
///
/// 本函数是应用的核心网络功能，负责将 `ApiRequest` 转换为实际的 HTTP 请求，
/// 发送后收集响应信息并构造 `ApiResponse`。
///
/// # 参数
///
/// * `request` - 包含方法、URL、请求头和请求体的完整请求描述。
///
/// # 返回值
///
/// * `Ok(ApiResponse)` - 请求成功，返回包含状态码、响应头、格式化响应体和耗时的响应结构体
/// * `Err(String)` - 请求失败，返回描述错误的字符串（网络错误、响应体读取失败等）
///
/// # Content-Type 自动检测逻辑
///
/// 当请求方法为 POST/PUT/PATCH 且用户未显式设置 `Content-Type` 请求头时：
/// - 请求体能被解析为有效 JSON → 自动设置 `application/json`
/// - 其他情况 → 默认设置 `text/plain`
///
/// # 响应体格式化
///
/// 如果响应体是有效的 JSON，会自动使用 `serde_json::to_string_pretty` 进行美化格式化，
/// 便于在 UI 中阅读。
///
/// # 示例
///
/// ```ignore
/// let request = ApiRequest {
///     method: HttpMethod::Get,
///     url: "https://httpbin.org/get".to_string(),
///     ..Default::default()
/// };
/// let response = send_request(request).await?;
/// println!("Status: {} {}", response.status, response.status_text);
/// ```
pub async fn send_request(request: ApiRequest) -> Result<ApiResponse, String> {
    // 步骤1：创建 HTTP 客户端
    // 每次请求创建新客户端，简化实现；生产环境可考虑复用客户端实例以提升性能
    let client = reqwest::Client::new();

    // 步骤2：记录请求开始时间，用于计算总耗时
    let start = Instant::now();

    // 步骤3：将应用层的 HttpMethod 枚举映射为 reqwest 的 Method 类型
    let method = match request.method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    };

    // 步骤4：使用方法和 URL 创建请求构建器
    let mut req_builder = client.request(method, &request.url);

    // 步骤5：添加自定义请求头
    // 跳过 key 为空的请求头，避免发送无效头部
    for header in &request.headers {
        if !header.key.is_empty() {
            req_builder = req_builder.header(&header.key, &header.value);
        }
    }

    // 步骤6：为支持请求体的方法添加请求体和 Content-Type
    match request.method {
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
            if !request.body.is_empty() {
                // 检查用户是否已显式设置 Content-Type 请求头
                let has_content_type = request.headers.iter().any(|h| h.key.eq_ignore_ascii_case("Content-Type"));

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

    // 步骤7：发送请求，将网络错误转换为用户友好的错误消息
    let response = req_builder
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    // 步骤8：计算请求总耗时（从发送到接收到响应头）
    let duration_ms = start.elapsed().as_millis() as u64;

    // 步骤9：提取 HTTP 状态信息
    let status = response.status();
    let status_code = status.as_u16();
    // canonical_reason() 返回标准的状态文本（如 200 → "OK"），未知状态码返回 None
    let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

    // 步骤10：收集响应头
    // 二进制头部值（无法转为 UTF-8 字符串）会被替换为 "[binary]"
    let mut headers = Vec::new();
    for (key, value) in response.headers() {
        headers.push((
            key.to_string(),
            value.to_str().unwrap_or("[binary]").to_string(),
        ));
    }

    // 步骤11：读取响应体为文本
    let body = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    // 步骤12：尝试将响应体格式化为美化的 JSON
    // 如果不是有效的 JSON，保留原始文本
    let formatted_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
        serde_json::to_string_pretty(&json).unwrap_or(body)
    } else {
        body
    };

    // 步骤13：构造并返回完整的响应结构体
    Ok(ApiResponse {
        status: status_code,
        status_text,
        headers,
        body: formatted_body,
        duration_ms,
    })
}
