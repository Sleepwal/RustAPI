use crate::app::ApiClientApp;
use crate::models::HttpMethod;
use egui::{ComboBox, TextEdit, Ui};

pub fn render(app: &mut ApiClientApp, ui: &mut Ui) {
    ui.heading("Request");
    ui.add_space(10.0);

    // Method and URL row
    ui.horizontal(|ui: &mut Ui| {
        // Method selector
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

        // URL input - 使用剩余空间
        ui.add(
            TextEdit::singleline(&mut app.request.url)
                .hint_text("Enter URL...")
                .desired_width(ui.available_width() - 80.0),
        );

        ui.add_space(10.0);

        // Send button - 固定宽度
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

    // Headers section
    ui.collapsing("Headers", |ui: &mut Ui| {
        // Existing headers
        let mut to_remove = None;
        for (index, header) in app.request.headers.iter_mut().enumerate() {
            ui.horizontal(|ui: &mut Ui| {
                ui.add(
                    TextEdit::singleline(&mut header.key)
                        .hint_text("Key")
                        .desired_width(200.0),
                );
                ui.add(
                    TextEdit::singleline(&mut header.value)
                        .hint_text("Value")
                        .desired_width(f32::INFINITY),
                );
                if ui.button("×").clicked() {
                    to_remove = Some(index);
                }
            });
        }

        if let Some(index) = to_remove {
            app.remove_header(index);
        }

        // Add new header
        ui.horizontal(|ui: &mut Ui| {
            ui.add(
                TextEdit::singleline(&mut app.new_header_key)
                    .hint_text("Key")
                    .desired_width(200.0),
            );
            ui.add(
                TextEdit::singleline(&mut app.new_header_value)
                    .hint_text("Value")
                    .desired_width(f32::INFINITY),
            );
            if ui.button("+ Add").clicked() {
                app.add_header();
            }
        });
    });

    ui.add_space(10.0);

    // Body section (only for POST, PUT, PATCH)
    match app.request.method {
        HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch => {
            ui.collapsing("Body", |ui: &mut Ui| {
                ui.horizontal(|ui: &mut Ui| {
                    ui.label("Request Body:");
                    if ui.button("Format JSON").clicked() {
                        app.format_request_body();
                    }
                });
                ui.add_space(5.0);
                ui.add(
                    TextEdit::multiline(&mut app.request.body)
                        .desired_width(f32::INFINITY)
                        .desired_rows(10)
                        .code_editor(),
                );
            });
        }
        _ => {}
    }
}
