//! API Client - 基于 egui/eframe 构建的图形化 HTTP API 测试工具。
//!
//! 本应用程序提供一个类似 Postman 的轻量级桌面 GUI，用于发送 HTTP 请求并查看响应。
//! 使用 egui 作为即时模式 GUI 框架，reqwest 作为 HTTP 客户端，poll-promise 实现异步请求。
//!
//! # 架构
//!
//! ```text
//! ┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
//! │  main.rs     │────>│  ApiClientApp   │────>│  http.rs     │
//! │  (入口)      │     │  (应用状态)     │     │  (HTTP发送)  │
//! └──────────────┘     └────────┬────────┘     └──────────────┘
//!                               │
//!                      ┌────────┴────────┐
//!                      │   ui/           │
//!                      │  (界面渲染)     │
//!                      └─────────────────┘
//! ```
//!
//! # 模块说明
//!
//! - `app`    - 应用核心状态与逻辑，实现 eframe::App trait
//! - `http`   - HTTP 请求发送与响应处理
//! - `models` - 数据模型定义（请求、响应、历史记录等）
//! - `ui`     - 界面渲染模块（请求面板、响应面板）

mod app;
mod http;
mod models;
mod ui;

use app::ApiClientApp;
use egui::ViewportBuilder;

/// 应用程序入口函数。
///
/// 使用 `#[tokio::main]` 属性宏将 async main 转换为 tokio 运行时入口，
/// 以便应用内部可以使用 reqwest 进行异步 HTTP 请求。
///
/// # 启动流程
///
/// 1. 配置 eframe 原生窗口选项（窗口大小、最小尺寸）
/// 2. 调用 `eframe::run_native` 启动 GUI 事件循环
/// 3. 在创建回调中实例化 `ApiClientApp`
///
/// # 窗口配置
///
/// - 默认窗口大小：900×700 像素
/// - 最小窗口大小：600×400 像素（防止界面元素挤压变形）
#[tokio::main]
async fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "API Client",
        options,
        Box::new(|cc| Ok(Box::new(ApiClientApp::new(cc)))),
    )
}
