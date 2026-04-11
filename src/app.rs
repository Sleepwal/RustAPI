//! 应用核心状态与逻辑模块。
//!
//! 定义了 `ApiClientApp` 结构体作为应用的全局状态容器，
//! 并实现 `eframe::App` trait 以接入 egui 的事件循环。
//!
//! # 职责
//!
//! - 管理请求/响应状态和 UI 交互状态
//! - 通过 `poll-promise` 实现异步 HTTP 请求（不阻塞 UI 线程）
//! - 协调 UI 面板的渲染
//!
//! # 异步请求机制
//!
//! ```text
//! ┌──────────┐  send_request()  ┌─────────────────┐
//! │ UI 线程  │────────────────>│ poll-promise     │
//! │          │                  │ (异步任务池)     │
//! │          │<────────────────│                   │
//! └──────────┘  check_response() └────────┬────────┘
//!       │                                   │
//!       │  request_repaint_after(100ms)     │
//!       │  (轮询检查完成状态)               ▼
//!       │                          ┌────────────────┐
//!       └─────────────────────────>│ http::send_    │
//!                                  │ request (tokio)│
//!                                  └────────────────┘
//! ```

use crate::models::{ApiRequest, ApiResponse, RequestHistory};
use crate::ui;
use poll_promise::Promise;

/// API 客户端应用的核心状态结构体。
///
/// 持有应用运行时的所有状态，包括当前请求配置、响应数据、
/// 异步请求的 Promise 以及 UI 交互状态。
///
/// # 字段分组
///
/// - **请求相关**：`request`、`new_header_key`、`new_header_value`
/// - **响应相关**：`response`、`error_message`、`active_response_tab`
/// - **异步状态**：`pending_request`
/// - **历史记录**：`history`
pub struct ApiClientApp {
    /// 当前正在编辑的 API 请求配置。
    pub request: ApiRequest,
    /// 最近一次请求的响应结果，无响应时为 `None`。
    pub response: Option<ApiResponse>,
    /// 请求历史记录容器。
    pub history: RequestHistory,
    /// 正在进行的异步请求的 Promise。
    ///
    /// 使用 `poll-promise` 库实现，允许在 egui 的同步渲染循环中
    /// 检查异步任务是否完成，而不会阻塞 UI 线程。
    pub pending_request: Option<Promise<Result<ApiResponse, String>>>,
    /// 错误消息，用于在 UI 中显示请求失败信息。
    pub error_message: Option<String>,
    /// 当前激活的响应面板标签页。
    pub active_response_tab: ResponseTab,
    /// 新增请求头的 Key 输入框内容。
    pub new_header_key: String,
    /// 新增请求头的 Value 输入框内容。
    pub new_header_value: String,
}

/// 响应面板的标签页枚举。
///
/// 控制响应区域显示的内容：响应体或响应头。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    /// 显示响应体内容（默认标签页）。
    Body,
    /// 显示响应头键值对列表。
    Headers,
}

impl Default for ApiClientApp {
    /// 创建带有默认值的应用实例。
    ///
    /// 默认 URL 设置为 `https://httpbin.org/get`，这是一个常用的
    /// HTTP 测试服务，方便用户首次启动时立即测试功能。
    fn default() -> Self {
        Self {
            request: ApiRequest {
                url: "https://httpbin.org/get".to_string(),
                ..Default::default()
            },
            response: None,
            history: RequestHistory::default(),
            pending_request: None,
            error_message: None,
            active_response_tab: ResponseTab::Body,
            new_header_key: String::new(),
            new_header_value: String::new(),
        }
    }
}

impl ApiClientApp {
    /// 创建新的应用实例。
    ///
    /// # 参数
    ///
    /// * `_cc` - eframe 的创建上下文，可用于初始化字体、样式等。
    ///           当前未使用，保留以备将来扩展。
    ///
    /// # 返回值
    ///
    /// 返回使用默认配置初始化的应用实例。
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    /// 发送 HTTP 请求。
    ///
    /// 将当前请求配置克隆后，通过 `poll-promise` 在 tokio 异步运行时中
    /// 发送请求。请求过程中不会阻塞 UI 线程。
    ///
    /// # 前置条件
    ///
    /// URL 不能为空，否则会设置错误消息并直接返回。
    ///
    /// # 行为说明
    ///
    /// - 克隆请求配置：因为异步任务需要拥有请求数据的所有权，
    ///   而 UI 仍需保留请求状态供用户继续编辑
    /// - 清除之前的错误消息：每次发送新请求时重置错误状态
    /// - 设置 `pending_request`：`check_response` 方法会轮询此 Promise
    pub fn send_request(&mut self) {
        if self.request.url.is_empty() {
            self.error_message = Some("URL cannot be empty".to_string());
            return;
        }

        // 清除之前的错误状态
        self.error_message = None;

        // 克隆请求配置，使异步任务拥有独立的数据副本
        let request = self.request.clone();

        // 在 tokio 运行时中异步发送请求
        // Promise::spawn_async 不会阻塞当前线程
        self.pending_request = Some(Promise::spawn_async(async move {
            crate::http::send_request(request).await
        }));
    }

    /// 检查异步请求是否已完成。
    ///
    /// 在每一帧的 `update` 中调用，轮询 `pending_request` 的完成状态。
    /// 如果请求已完成，更新响应或错误消息，并清除 Promise。
    ///
    /// # 行为说明
    ///
    /// - `promise.ready()` 返回 `Some` 表示异步任务已完成
    /// - 请求成功：更新 `response`，清除 `error_message`，保存历史记录
    /// - 请求失败：更新 `error_message`，清除 `response`
    /// - 无论成功失败，都清除 `pending_request`（设为 `None`）
    pub fn check_response(&mut self) {
        if let Some(promise) = &self.pending_request {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(response) => {
                        // 保存历史记录
                        self.history.requests.push(crate::models::HistoryItem {
                            timestamp: std::time::SystemTime::now(),
                            request: self.request.clone(),
                            response: Some(response.clone()),
                        });
                        self.response = Some(response.clone());
                        self.error_message = None;
                    }
                    Err(err) => {
                        // 也保存失败的请求历史
                        self.history.requests.push(crate::models::HistoryItem {
                            timestamp: std::time::SystemTime::now(),
                            request: self.request.clone(),
                            response: None,
                        });
                        self.error_message = Some(err.clone());
                        self.response = None;
                    }
                }
                // 请求已完成，清除 Promise 使 is_requesting() 返回 false
                self.pending_request = None;
            }
        }
    }

    /// 检查当前是否有请求正在进行中。
    ///
    /// 用于 UI 中控制发送按钮的状态（防止重复发送）和显示加载指示器。
    pub fn is_requesting(&self) -> bool {
        self.pending_request.is_some()
    }

    /// 添加一个新的请求头。
    ///
    /// 将 `new_header_key` 和 `new_header_value` 中的内容作为新的
    /// 请求头添加到请求头列表，然后清空输入框。
    ///
    /// # 前置条件
    ///
    /// `new_header_key` 不能为空，空 key 的请求头没有意义。
    pub fn add_header(&mut self) {
        if !self.new_header_key.is_empty() {
            self.request.headers.push(crate::models::Header {
                key: self.new_header_key.clone(),
                value: self.new_header_value.clone(),
            });
            // 添加成功后清空输入框，准备下一次输入
            self.new_header_key.clear();
            self.new_header_value.clear();
        }
    }

    /// 移除指定索引的请求头。
    ///
    /// # 参数
    ///
    /// * `index` - 要移除的请求头在列表中的索引。
    ///
    /// # 安全性
    ///
    /// 会检查索引是否越界，越界时不执行任何操作。
    pub fn remove_header(&mut self, index: usize) {
        if index < self.request.headers.len() {
            self.request.headers.remove(index);
        }
    }

    /// 格式化请求体中的 JSON 内容。
    ///
    /// 尝试将 `request.body` 解析为 JSON 并使用美化格式重新输出。
    /// 如果内容不是有效的 JSON，则不做任何修改。
    ///
    /// # 用途
    ///
    /// 用户点击 "Format JSON" 按钮时调用，使压缩的 JSON 更易阅读。
    pub fn format_request_body(&mut self) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.request.body) {
            self.request.body = serde_json::to_string_pretty(&json).unwrap_or_default();
        }
    }

    /// 加载指定索引的历史记录到当前编辑器。
    ///
    /// # 参数
    ///
    /// * `index` - 历史记录列表中的索引。
    ///
    /// # 安全性
    ///
    /// 会检查索引是否越界，越界时不执行任何操作。
    pub fn load_history_item(&mut self, index: usize) {
        if let Some(item) = self.history.requests.get(index) {
            self.request = item.request.clone();
        }
    }

    /// 清空所有请求历史记录。
    pub fn clear_history(&mut self) {
        self.history.requests.clear();
    }

    /// 根据 HTTP 方法自动更新 URL。
    ///
    /// 使用 httpbin.org 作为测试服务，不同方法对应不同的端点。
    pub fn update_url_for_method(&mut self) {
        use crate::models::HttpMethod;
        self.request.url = match self.request.method {
            HttpMethod::Get => "https://httpbin.org/get".to_string(),
            HttpMethod::Post => "https://httpbin.org/post".to_string(),
            HttpMethod::Put => "https://httpbin.org/put".to_string(),
            HttpMethod::Delete => "https://httpbin.org/delete".to_string(),
            HttpMethod::Patch => "https://httpbin.org/patch".to_string(),
            HttpMethod::Head => "https://httpbin.org/get".to_string(),
            HttpMethod::Options => "https://httpbin.org/get".to_string(),
        };
    }

    /// 根据 HTTP 方法自动更新请求体。
    ///
    /// POST/PUT/PATCH 方法会自动填充示例 JSON 请求体，
    /// 其他方法清空请求体。
    pub fn update_body_for_method(&mut self) {
        use crate::models::HttpMethod;
        self.request.body = match self.request.method {
            HttpMethod::Post => {
                "{\n  \"name\": \"test\",\n  \"email\": \"test@example.com\"\n}".to_string()
            }
            HttpMethod::Put => {
                "{\n  \"id\": 1,\n  \"name\": \"updated_name\"\n}".to_string()
            }
            HttpMethod::Patch => {
                "{\n  \"name\": \"patched_name\"\n}".to_string()
            }
            _ => String::new(),
        };
    }
}

impl eframe::App for ApiClientApp {
    /// egui 主渲染循环，每帧调用一次。
    ///
    /// 负责检查异步请求状态、触发重绘以及渲染整个 UI 布局。
    ///
    /// # 渲染流程
    ///
    /// 1. 检查异步请求是否完成（`check_response`）
    /// 2. 如果有进行中的请求，请求每 100ms 重绘一次以轮询结果
    /// 3. 在中央面板中依次渲染请求面板和响应面板
    ///
    /// # 关于重绘机制
    ///
    /// egui 默认仅在用户交互时重绘。当有异步请求进行时，
    /// 需要主动调用 `request_repaint_after` 确保能及时检测到请求完成。
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查异步请求是否已完成
        self.check_response();

        // 有进行中的请求时，每 100ms 请求一次重绘
        // 这样可以及时检测到 Promise 完成，而不会让用户等待太久
        if self.is_requesting() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        // 渲染主面板
        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.heading("API Client");
            ui.add_space(10.0);

            // 渲染请求配置面板（方法、URL、请求头、请求体）
            ui::request_panel::render(self, ui);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // 渲染请求历史记录面板
            ui::history_panel::render(self, ui);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // 渲染响应展示面板（状态码、响应体、响应头）
            ui::response_panel::render(self, ui);
        });
    }
}
