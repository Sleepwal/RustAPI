//! 请求历史记录面板模块。
//!
//! 渲染已发送请求的历史列表，包括：
//! - 可折叠的历史记录区域
//! - 每条记录显示时间戳、HTTP 方法、URL、状态码
//! - 点击记录可恢复请求配置到当前编辑器
//! - 支持清空历史记录按钮
//!
//! # UI 布局
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//!  ▶ History (3)                      [Clear All]
//!    ┌─────────────────────────────────────────────┐
//!    │ [GET]    https://httpbin.org/get    200 OK  │
//!    │ [POST]   https://httpbin.org/post   201 OK  │
//!    │ [GET]    https://invalid-url        Error   │
//!    └─────────────────────────────────────────────┘
//! └─────────────────────────────────────────────────┘

use crate::app::ApiClientApp;
use crate::models::HttpMethod;
use egui::{Color32, FontFamily, FontId, ScrollArea, Ui};
use std::time::SystemTime;

/// 获取 Small 文本的 FontId。
fn small_font(ui: &Ui) -> FontId {
    let body_font = ui.style().text_styles.get(&egui::TextStyle::Body).cloned()
        .unwrap_or(FontId::new(14.0, FontFamily::Proportional));
    FontId::new(body_font.size * 0.85, body_font.family)
}

/// 获取 Small 文本的 Monospace FontId。
fn small_mono_font(ui: &Ui) -> FontId {
    FontId::new(small_font(ui).size, FontFamily::Monospace)
}

/// 格式化时间戳为可读字符串（相对时间，如 "2m ago"）。
///
/// # 参数
///
/// * `time` - 系统时间戳。
///
/// # 返回值
///
/// 返回相对时间的简短描述，如 "now"、"30s"、"2m"、"1h" 等。
fn format_relative_time(time: SystemTime) -> String {
    if let Ok(duration) = time.elapsed() {
        let secs = duration.as_secs();
        if secs < 5 {
            "now".to_string()
        } else if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m", secs / 60)
        } else {
            format!("{}h", secs / 3600)
        }
    } else {
        "future".to_string()
    }
}

/// 获取 HTTP 方法对应的颜色。
///
/// 不同方法用不同颜色标识，便于快速识别。
fn method_color(method: HttpMethod) -> Color32 {
    match method {
        HttpMethod::Get => Color32::GREEN,
        HttpMethod::Post => Color32::YELLOW,
        HttpMethod::Put => Color32::from_rgb(0, 150, 255),
        HttpMethod::Delete => Color32::RED,
        HttpMethod::Patch => Color32::from_rgb(200, 100, 255),
        HttpMethod::Head => Color32::GRAY,
        HttpMethod::Options => Color32::DARK_GRAY,
    }
}

/// 渲染请求历史记录面板。
///
/// 在可折叠区域中展示历史请求列表，点击某条记录可将其
/// 请求配置恢复到当前编辑器。
///
/// # 参数
///
/// * `app` - 可变引用应用状态，用于读取历史记录和加载请求配置
/// * `ui` - egui UI 上下文，用于绘制界面元素
pub fn render(app: &mut ApiClientApp, ui: &mut Ui) {
    let history_count = app.history.len();

    ui.horizontal(|ui: &mut Ui| {
        ui.heading("History");
        ui.label(format!("({})", history_count));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui: &mut Ui| {
            if !app.history.is_empty()
                && ui.button("Clear All").clicked()
            {
                app.clear_history();
            }
        });
    });

    ui.add_space(5.0);

    if app.history.is_empty() {
        ui.label("No request history yet. Send a request to see it here.");
        return;
    }

    // 历史列表用 ScrollArea 包裹，限制最大高度
    ScrollArea::vertical()
        .id_source("history_scroll")
        .max_height(200.0)
        .show(ui, |ui: &mut Ui| {
            // 反向遍历，最新的请求显示在最上面
            let mut clicked_index = None;
            for (idx_reverse, item) in app.history.requests.iter().rev().enumerate() {
                let idx = app.history.len() - 1 - idx_reverse;
                let time_str = format_relative_time(item.timestamp);
                let method = item.request.method;
                let url = &item.request.url;

                // 截断 URL 以适应宽度
                let url_display = if url.len() > 50 {
                    format!("...{}", &url[url.len() - 47..])
                } else {
                    url.clone()
                };

                // 状态显示
                let status_display = match &item.response {
                    Some(resp) => format!("{} {}", resp.status, resp.status_text),
                    None => "Error".to_string(),
                };
                let status_color = match &item.response {
                    Some(resp) => {
                        let status = resp.status;
                        if (200..300).contains(&status) {
                            Color32::GREEN
                        } else if status >= 400 {
                            Color32::RED
                        } else {
                            Color32::YELLOW
                        }
                    }
                    None => Color32::RED,
                };

                // 历史记录行 - 颜色
                let method_col = method_color(method);

                // 先分配一个可交互的矩形区域来检测点击
                let desired_size = egui::vec2(ui.available_width(), 18.0);
                let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::click());

                // 悬停时高亮背景
                if response.hovered() {
                    let bg_rect = response.rect;
                    painter.rect_filled(bg_rect, 2.0, ui.visuals().widgets.noninteractive.bg_fill);
                }

                // 在矩形区域内绘制内容
                let rect = response.rect;
                let text_height = rect.height();
                let y = rect.min.y;
                let font = small_font(ui);
                let mono_font = small_mono_font(ui);

                // 时间戳
                let time_w = ui.fonts(|f| f.glyph_width(&font, '0')) * 8.0;
                let mut x = rect.min.x + 4.0;
                painter.text(
                    egui::pos2(x, y + text_height / 2.0),
                    egui::Align2::LEFT_CENTER,
                    &time_str,
                    font.clone(),
                    Color32::GRAY,
                );
                x += time_w + 8.0;

                // HTTP 方法
                let method_text = format!("[{}]", method.as_str());
                let method_w = ui.fonts(|f| f.glyph_width(&mono_font, '[')) * method_text.len() as f32;
                painter.text(
                    egui::pos2(x, y + text_height / 2.0),
                    egui::Align2::LEFT_CENTER,
                    &method_text,
                    mono_font.clone(),
                    method_col,
                );
                x += method_w + 8.0;

                // URL
                painter.text(
                    egui::pos2(x, y + text_height / 2.0),
                    egui::Align2::LEFT_CENTER,
                    &url_display,
                    font.clone(),
                    ui.visuals().text_color(),
                );

                // 右侧状态码
                let _status_w = ui.fonts(|f| f.glyph_width(&font, '0')) * status_display.len() as f32;
                painter.text(
                    egui::pos2(rect.max.x - 4.0, y + text_height / 2.0),
                    egui::Align2::RIGHT_CENTER,
                    &status_display,
                    font,
                    status_color,
                );

                // 设置鼠标指针样式
                if response.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if response.clicked() {
                    clicked_index = Some(idx);
                }
            }

            // 在迭代完成后加载选中的历史记录
            if let Some(idx) = clicked_index {
                app.load_history_item(idx);
            }
        });
}
