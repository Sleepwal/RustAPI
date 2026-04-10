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
#[derive(Debug, Clone, Default)]
pub struct ApiRequest {
    /// HTTP 请求方法，默认为 GET。
    pub method: HttpMethod,
    /// 请求目标 URL，包含协议前缀（如 `https://`）。
    pub url: String,
    /// 自定义请求头列表。
    pub headers: Vec<Header>,
    /// 请求体内容，通常为 JSON 格式的字符串。
    pub body: String,
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
    pub headers: Vec<(String, String)>,
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
    pub response: Option<ApiResponse>,
}
