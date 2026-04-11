//! UI 渲染模块。
//!
//! 包含应用程序的各个界面面板的渲染函数。
//! 每个子模块负责一个独立的 UI 区域，采用函数式渲染模式
//! （接收 `&mut ApiClientApp` 和 `&mut Ui` 参数）。
//!
//! # 模块结构
//!
//! - `request_panel` - 请求配置面板（方法、URL、请求头、请求体）
//! - `response_panel` - 响应展示面板（状态码、响应体、响应头）
//! - `history_panel` - 请求历史记录面板

pub mod history_panel;
pub mod request_panel;
pub mod response_panel;
