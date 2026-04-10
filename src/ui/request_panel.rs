//! 请求配置面板模块。
//!
//! 渲染 HTTP 请求的配置界面，包括：
//! - HTTP 方法选择器和 URL 输入框（同一行布局）
//! - 自定义请求头的可折叠编辑区域
//! - 请求体编辑器（仅 POST/PUT/PATCH 方法显示）
//!
//! # UI 布局
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//!  Request
//!  ┌──────────┐ ┌──────────────────────┐ ┌────────┐
//!  │ GET  ▼   │ │ https://example.com  │ │ Send   │
//!  └──────────┘ └──────────────────────┘ └────────┘
//!
//!  ▶ Headers
//!    ┌──────────┐ ┌───────────────────┐ ┌──┐
//!    │ Key      │ │ Value             │ │ ×│
//!    └──────────┘ └───────────────────┘ └──┘
//!    ┌──────────┐ ┌───────────────────┐ ┌───────┐
//!    │ Key      │ │ Value             │ │+ Add  │
//!    └──────────┘ └───────────────────┘ └───────┘
//!
//!  ▶ Body  (仅 POST/PUT/PATCH)
//!    Request Body:  [Format JSON]
//!    ┌─────────────────────────────────────┐
//!    │ {                                   │
//!    │   "key": "value"                    │
//!    │ }                                   │
//!    └─────────────────────────────────────┘
//! └─────────────────────────────────────────────────┘
//! ```

use crate::app::ApiClientApp;
use crate::models::HttpMethod;
use egui::{ComboBox, TextEdit, Ui};

/// 渲染请求配置面板。
///
/// 将所有请求相关的 UI 元素渲染到给定的 egui UI 上下文中。
/// 包括方法选择器、URL 输入框、发送按钮、请求头编辑区和请求体编辑区。
///
/// # 参数
///
/// * `app` - 可变引用应用状态，用于读写请求配置和触发请求发送
/// * `ui` - egui UI 上下文，用于绘制界面元素
pub fn render(app: &mut ApiClientApp, ui: &mut Ui) {
    ui.heading("Request");
    ui.add_space(10.0);

    // ── 第一行：HTTP 方法选择器 + URL 输入框 + 发送按钮 ──
    ui.horizontal(|ui: &mut Ui| {
        // HTTP 方法下拉选择框
        // 使用 ComboBox 组件，固定宽度 100px
        ComboBox::from_id_source("method_selector")
            .selected_text(app.request.method.to_string())
            .width(100.0)
            .show_ui(ui, |ui: &mut Ui| {
                for method in HttpMethod::ALL {
                    ui.selectable_value(
                        &mut app.request.method,
                        method,
                        method.to_string(),
                    );
                }
            });

        ui.add_space(10.0);

        // URL 输入框
        // 使用 available_width() 减去按钮宽度(80px)，使输入框填满剩余空间
        ui.add(
            TextEdit::singleline(&mut app.request.url)
                .hint_text("Enter URL...")
                .desired_width(ui.available_width() - 80.0),
        );

        ui.add_space(10.0);

        // 发送按钮
        // 请求进行中时显示 "Sending..." 并禁用点击
        let button_text = if app.is_requesting() {
            "Sending..."
        } else {
            "Send"
        };
        if ui.button(button_text).clicked() && !app.is_requesting() {
            app.send_request();
        }
    });

    ui.add_space(15.0);

    // ── 请求头编辑区域（可折叠） ──
    ui.collapsing("Headers", |ui: &mut Ui| {
        // 已有请求头列表
        // 使用 to_remove 模式避免在迭代中修改集合
        let mut to_remove = None;
        for (index, header) in app.request.headers.iter_mut().enumerate() {
            ui.horizontal(|ui: &mut Ui| {
                // 请求头 Key 输入框，固定宽度 200px
                ui.add(
                    TextEdit::singleline(&mut header.key)
                        .hint_text("Key")
                        .desired_width(200.0),
                );
                // 请求头 Value 输入框，填满剩余空间
                ui.add(
                    TextEdit::singleline(&mut header.value)
                        .hint_text("Value")
                        .desired_width(f32::INFINITY),
                );
                // 删除按钮（× 符号）
                if ui.button("×").clicked() {
                    to_remove = Some(index);
                }
            });
        }

        // 在迭代完成后执行删除操作
        if let Some(index) = to_remove {
            app.remove_header(index);
        }

        // 新增请求头输入行
        ui.horizontal(|ui: &mut Ui| {
            // 新请求头 Key 输入框
            ui.add(
                TextEdit::singleline(&mut app.new_header_key)
                    .hint_text("Key")
                    .desired_width(200.0),
            );
            // 新请求头 Value 输入框
            ui.add(
                TextEdit::singleline(&mut app.new_header_value)
                    .hint_text("Value")
                    .desired_width(f32::INFINITY),
            );
            // 添加按钮
            if ui.button("+ Add").clicked() {
                app.add_header();
            }
        });
    });

    ui.add_space(10.0);

    // ── 请求体编辑区域（仅 POST/PUT/PATCH 方法显示） ──
    match app.request.method {
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
            ui.collapsing("Body", |ui: &mut Ui| {
                ui.horizontal(|ui: &mut Ui| {
                    ui.label("Request Body:");
                    // JSON 格式化按钮，方便用户美化压缩的 JSON
                    if ui.button("Format JSON").clicked() {
                        app.format_request_body();
                    }
                });
                ui.add_space(5.0);
                // 多行文本编辑器，启用代码编辑器样式（等宽字体、行号等）
                ui.add(
                    TextEdit::multiline(&mut app.request.body)
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .code_editor(),
                );
            });
        }
        _ => {
            // GET、DELETE、HEAD、OPTIONS 方法不显示请求体编辑区
        }
    }
}
