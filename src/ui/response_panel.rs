//! 响应展示面板模块。
//!
//! 渲染 HTTP 响应的展示界面，包括：
//! - 错误消息提示（请求失败时）
//! - 状态码和耗时信息（带颜色标识）
//! - Body/Headers 标签页切换
//! - 响应体内容（只读代码编辑器样式）
//! - 响应头键值对表格
//!
//! # UI 布局
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//!  Response
//!
//!  Status: 200 OK          Duration: 150 ms
//!
//!  [Body]  [Headers]       ← 标签页切换
//!  ───────────────────────
//!  ┌─────────────────────────────────────────────┐
//!  │ {                                           │
//!  │   "url": "https://httpbin.org/get",         │
//!  │   "headers": { ... }                        │
//!  │ }                                           │
//!  └─────────────────────────────────────────────┘
//! └─────────────────────────────────────────────────┘
//!
//! # 状态颜色规则
//!
//! - 2xx（成功）→ 绿色
//! - 3xx（重定向）→ 黄色
//! - 4xx/5xx（客户端/服务端错误）→ 红色

use crate::app::{ApiClientApp, ResponseTab};
use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};

/// 渲染响应展示面板。
///
/// 根据当前应用状态显示不同的内容：
/// - 有错误消息时：显示红色错误提示
/// - 有响应数据时：显示状态码、标签页和响应内容
/// - 请求进行中时：显示加载指示器
/// - 无响应且无请求时：显示占位提示文本
///
/// # 参数
///
/// * `app` - 可变引用应用状态，用于读取响应数据和切换标签页
/// * `ui` - egui UI 上下文，用于绘制界面元素
pub fn render(app: &mut ApiClientApp, ui: &mut Ui) {
    ui.heading("Response");
    ui.add_space(10.0);

    // ── 错误消息展示 ──
    if let Some(error) = &app.error_message {
        ui.colored_label(Color32::RED, format!("Error: {}", error));
        ui.add_space(10.0);
    }

    // ── 响应数据展示 ──
    if let Some(response) = &app.response {
        // 提前克隆数据，避免在闭包中存在可变/不可变借用冲突
        let status = response.status;
        let status_text = response.status_text.clone();
        let duration_ms = response.duration_ms;
        let body = response.body.clone();
        let headers = response.headers.clone();

        // ── 状态行：状态码 + 耗时 ──
        // 根据状态码范围选择颜色
        let status_color = if status >= 200 && status < 300 {
            Color32::GREEN    // 2xx 成功
        } else if status >= 400 {
            Color32::RED      // 4xx/5xx 错误
        } else {
            Color32::YELLOW   // 3xx 重定向或其他
        };

        ui.horizontal(|ui: &mut Ui| {
            // 状态码以粗体+颜色显示
            ui.label(
                RichText::new(format!("Status: {} {}", status, status_text))
                    .color(status_color)
                    .strong(),
            );
            // 请求耗时
            ui.label(format!("Duration: {} ms", duration_ms));
        });

        ui.add_space(10.0);

        // ── 标签页切换按钮 ──
        ui.horizontal(|ui: &mut Ui| {
            // Body 标签页
            if ui
                .selectable_label(app.active_response_tab == ResponseTab::Body, "Body")
                .clicked()
            {
                app.active_response_tab = ResponseTab::Body;
            }
            // Headers 标签页
            if ui
                .selectable_label(app.active_response_tab == ResponseTab::Headers, "Headers")
                .clicked()
            {
                app.active_response_tab = ResponseTab::Headers;
            }
        });

        ui.add_space(5.0);
        ui.separator();
        ui.add_space(5.0);

        // ── 标签页内容区域 ──
        match app.active_response_tab {
            ResponseTab::Body => {
                // 响应体内容展示
                // 使用 ScrollArea 包裹以支持长内容的滚动浏览
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui: &mut Ui| {
                        // TextEdit 需要 &mut String，但响应体应为只读
                        // 因此克隆一份用于显示，并设置 interactive(false) 禁止编辑
                        let mut body_clone = body;
                        ui.add(
                            TextEdit::multiline(&mut body_clone)
                                .desired_width(f32::INFINITY)
                                .desired_rows(15)
                                .code_editor()       // 使用等宽字体和代码编辑器样式
                                .interactive(false),  // 只读模式
                        );
                    });
            }
            ResponseTab::Headers => {
                // 响应头键值对展示
                // 使用 Grid 组件实现表格布局，带斑马纹
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui: &mut Ui| {
                        egui::Grid::new("response_headers_grid")
                            .num_columns(2)
                            .striped(true)  // 交替行背景色，提升可读性
                            .show(ui, |ui: &mut Ui| {
                                for (key, value) in &headers {
                                    // 请求头名称以粗体显示，便于快速扫描
                                    ui.label(RichText::new(key).strong());
                                    ui.label(value);
                                    ui.end_row();
                                }
                            });
                    });
            }
        }
    } else if app.is_requesting() {
        // ── 请求进行中：显示加载指示器 ──
        ui.horizontal(|ui: &mut Ui| {
            ui.spinner();
            ui.label("Sending request...");
        });
    } else {
        // ── 无响应且无请求：显示占位提示 ──
        ui.label("No response yet. Send a request to see the response here.");
    }
}
