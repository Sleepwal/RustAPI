use crate::app::{ApiClientApp, ResponseTab};
use egui::{Color32, RichText, ScrollArea, TextEdit, Ui};

pub fn render(app: &mut ApiClientApp, ui: &mut Ui) {
    ui.heading("Response");
    ui.add_space(10.0);

    // Show error message if any
    if let Some(error) = &app.error_message {
        ui.colored_label(Color32::RED, format!("Error: {}", error));
        ui.add_space(10.0);
    }

    // Show response if available
    if let Some(response) = &app.response {
        let status = response.status;
        let status_text = response.status_text.clone();
        let duration_ms = response.duration_ms;
        let body = response.body.clone();
        let headers = response.headers.clone();

        // Status line
        let status_color = if status >= 200 && status < 300 {
            Color32::GREEN
        } else if status >= 400 {
            Color32::RED
        } else {
            Color32::YELLOW
        };

        ui.horizontal(|ui: &mut Ui| {
            ui.label(
                RichText::new(format!("Status: {} {}", status, status_text))
                    .color(status_color)
                    .strong(),
            );
            ui.label(format!("Duration: {} ms", duration_ms));
        });

        ui.add_space(10.0);

        // Response tabs
        ui.horizontal(|ui: &mut Ui| {
            if ui
                .selectable_label(app.active_response_tab == ResponseTab::Body, "Body")
                .clicked()
            {
                app.active_response_tab = ResponseTab::Body;
            }
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

        // Tab content
        match app.active_response_tab {
            ResponseTab::Body => {
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui: &mut Ui| {
                        // Use a placeholder mutable variable for the read-only view
                        let mut body_clone = body;
                        ui.add(
                            TextEdit::multiline(&mut body_clone)
                                .desired_width(f32::INFINITY)
                                .desired_rows(15)
                                .code_editor()
                                .interactive(false),
                        );
                    });
            }
            ResponseTab::Headers => {
                ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui: &mut Ui| {
                        egui::Grid::new("response_headers_grid")
                            .num_columns(2)
                            .striped(true)
                            .show(ui, |ui: &mut Ui| {
                                for (key, value) in &headers {
                                    ui.label(RichText::new(key).strong());
                                    ui.label(value);
                                    ui.end_row();
                                }
                            });
                    });
            }
        }
    } else if app.is_requesting() {
        ui.horizontal(|ui: &mut Ui| {
            ui.spinner();
            ui.label("Sending request...");
        });
    } else {
        ui.label("No response yet. Send a request to see the response here.");
    }
}
