//! 数据模型模块。
//!
//! 定义应用程序中使用的核心数据结构，包括 HTTP 方法枚举、
//! API 请求/响应结构体、请求头以及历史记录相关类型。
//!
//! # 数据流
//!
//! ```text
//! ApiRequest ──── send_request() ────> ApiResponse
//!     │                                      │
//!     │  (方法、URL、请求头、请求体)          │  (状态码、响应头、响应体、耗时)
//!     │                                      │
//!     └──────── HistoryItem ◄────────────────┘
//!              (时间戳 + 请求 + 响应)
//! ```

#[allow(unused_imports)]
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 最大历史记录数量限制。
///
/// 防止无限增长导致内存泄漏。当达到限制时，
/// 最旧的记录会被自动移除。
pub const MAX_HISTORY_SIZE: usize = 100;

/// API 请求错误类型。
///
/// 结构化的错误枚举，替代简单的 String 错误，
/// 便于错误处理和用户友好的错误消息展示。
#[derive(Debug, Clone)]
pub enum ApiError {
    /// URL 格式无效。
    InvalidUrl(String),
    /// 网络请求失败（连接错误、超时等）。
    NetworkError(String),
    /// 读取响应体失败。
    ResponseReadError(String),
}

impl ApiError {
    /// 将错误转换为用户友好的显示消息。
    ///
    /// # 返回值
    ///
    /// 返回简短的错误描述，适合在 UI 中显示。
    pub fn user_message(&self) -> String {
        match self {
            ApiError::InvalidUrl(url) => format!("Invalid URL format: {}", url),
            ApiError::NetworkError(msg) => format!("Network error: {}", msg),
            ApiError::ResponseReadError(msg) => format!("Failed to read response: {}", msg),
        }
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
            ApiError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            ApiError::ResponseReadError(msg) => write!(f, "Response read error: {}", msg),
        }
    }
}

/// HTTP 请求方法枚举。
///
/// 支持常见的 7 种 HTTP 方法，默认为 GET。
/// 通过 `#[derive(Default)]` 使 `HttpMethod::Get` 成为默认值，
/// 便于新建请求时自动选择 GET 方法。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HttpMethod {
    #[default]
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl HttpMethod {
    /// 所有支持的 HTTP 方法列表。
    ///
    /// 用于在 UI 中渲染方法选择下拉框，确保所有方法都能被遍历展示。
    pub const ALL: [HttpMethod; 7] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Head,
        HttpMethod::Options,
    ];

    /// 将 HTTP 方法转换为大写的静态字符串表示。
    ///
    /// # 返回值
    ///
    /// 返回对应 HTTP 方法的大写名称，如 `"GET"`、`"POST"` 等。
    /// 使用 `&'static str` 避免每次调用都分配新的字符串。
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}

impl std::fmt::Display for HttpMethod {
    /// 实现 Display trait，使 HttpMethod 可以直接用于格式化输出。
    ///
    /// 委托给 `as_str()` 方法，保持一致的字符串表示。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// API 请求结构体。
///
/// 描述一个完整的 HTTP 请求所需的所有信息。
/// 作为应用核心状态的一部分，由 UI 面板读写。
///
/// # 不变量
///
/// - `url` 在发送请求前不应为空（由 `send_request` 方法校验）
/// - `body` 仅在 POST/PUT/PATCH 方法时有效，其他方法会被忽略
/// - `headers` 中的 `key` 不应为空字符串（发送时自动跳过空 key）
#[derive(Debug, Clone)]
pub struct ApiRequest {
    /// HTTP 请求方法，默认为 GET。
    pub method: HttpMethod,
    /// 请求目标 URL，包含协议前缀（如 `https://`）。
    pub url: String,
    /// 自定义请求头列表。
    pub headers: Vec<Header>,
    /// 请求体内容，通常为 JSON 格式的字符串。
    pub body: String,
    /// 请求超时时间（秒），默认为 30 秒。
    pub timeout_secs: u64,
}

impl Default for ApiRequest {
    fn default() -> Self {
        Self {
            method: HttpMethod::default(),
            url: String::new(),
            headers: Vec::new(),
            body: String::new(),
            timeout_secs: 30, // 默认 30 秒超时
        }
    }
}

impl ApiRequest {
    /// 验证 URL 格式并自动补全协议前缀。
    ///
    /// # 验证规则
    ///
    /// - URL 不能为空
    /// - 必须以 `http://` 或 `https://` 开头
    /// - 如果缺少协议前缀，自动添加 `https://`
    ///
    /// # 返回值
    ///
    /// * `Ok(String)` - 验证通过，返回补全后的 URL（如果需要补全）
    /// * `Err(String)` - 验证失败，返回错误描述
    pub fn validate_and_normalize_url(url: &str) -> Result<String, String> {
        if url.is_empty() {
            return Err("URL cannot be empty".to_string());
        }

        // 如果已经有协议前缀，直接使用
        let normalized_url = if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            // 自动补全 https:// 前缀
            format!("https://{}", url)
        };

        // 使用 url crate 验证格式（reqwest 依赖已包含）
        if let Err(e) = normalized_url.parse::<url::Url>() {
            return Err(format!("Invalid URL format: {}", e));
        }

        Ok(normalized_url)
    }
}

/// HTTP 请求头键值对。
///
/// 表示一个自定义请求头，如 `Content-Type: application/json`。
/// 在 UI 中以可编辑的行形式展示，支持添加和删除。
#[derive(Debug, Clone, Default)]
pub struct Header {
    /// 请求头名称（如 `"Content-Type"`）。
    pub key: String,
    /// 请求头值（如 `"application/json"`）。
    pub value: String,
}

/// API 响应结构体。
///
/// 描述 HTTP 响应的完整信息，包括状态码、响应头和格式化后的响应体。
/// 由 `http::send_request` 函数构造并返回。
///
/// # 注意
///
/// - `body` 已经过 JSON 格式化处理（如果响应是有效的 JSON）
/// - `headers` 中的二进制值会被替换为 `"[binary]"` 字符串
/// - `duration_ms` 包含从发送请求到接收完整响应的总耗时
#[derive(Debug, Clone, Default)]
pub struct ApiResponse {
    /// HTTP 状态码（如 200、404、500）。
    pub status: u16,
    /// HTTP 状态文本（如 "OK"、"Not Found"）。
    pub status_text: String,
    /// 响应头列表，每个元素为 (键, 值) 的元组。
    /// 使用 Arc 包装以避免多次 clone。
    pub headers: Arc<Vec<(String, String)>>,
    /// 响应体内容，JSON 响应会自动格式化为美化输出。
    pub body: String,
    /// 请求耗时，单位为毫秒。
    pub duration_ms: u64,
}

/// 请求历史记录容器。
///
/// 存储所有已发送请求的历史条目，用于实现请求历史浏览功能。
/// 目前仅存储数据，尚未在 UI 中展示。
#[derive(Debug, Clone)]
pub struct RequestHistory {
    /// 历史条目列表，按时间顺序排列。
    pub requests: Vec<HistoryItem>,
}

impl Default for RequestHistory {
    /// 创建空的历史记录容器。
    fn default() -> Self {
        Self {
            requests: Vec::new(),
        }
    }
}

impl RequestHistory {
    /// 添加新的历史记录条目。
    ///
    /// 当历史记录数量达到 `MAX_HISTORY_SIZE` 限制时，
    /// 会自动移除最旧的记录（FIFO 策略）。
    ///
    /// # 参数
    ///
    /// * `item` - 要添加的历史记录条目。
    pub fn add(&mut self, item: HistoryItem) {
        self.requests.push(item);
        // 如果超过限制，移除最旧的记录
        if self.requests.len() > MAX_HISTORY_SIZE {
            // 保留最新的 MAX_HISTORY_SIZE 条记录
            let remove_count = self.requests.len() - MAX_HISTORY_SIZE;
            self.requests.drain(0..remove_count);
        }
    }

    /// 清空所有请求历史记录。
    pub fn clear(&mut self) {
        self.requests.clear();
    }

    /// 获取当前历史记录数量。
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// 检查历史记录是否为空。
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

/// 单条历史记录条目。
///
/// 将一次完整的请求-响应对记录下来，包含时间戳、请求和可选的响应。
/// 响应为 `None` 表示请求失败（网络错误等）。
#[derive(Debug, Clone)]
pub struct HistoryItem {
    /// 请求发送的时间戳。
    pub timestamp: std::time::SystemTime,
    /// 发送的请求内容。
    pub request: ApiRequest,
    /// 接收到的响应，请求失败时为 `None`。
    /// 使用 Arc 包装以避免大型数据的 clone。
    pub response: Option<Arc<ApiResponse>>,
}
