use crate::models::{ApiRequest, ApiResponse, RequestHistory};
use crate::ui;
use poll_promise::Promise;

pub struct ApiClientApp {
    pub request: ApiRequest,
    pub response: Option<ApiResponse>,
    pub history: RequestHistory,
    pub pending_request: Option<Promise<Result<ApiResponse, String>>>,
    pub error_message: Option<String>,
    pub active_response_tab: ResponseTab,
    pub new_header_key: String,
    pub new_header_value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    Body,
    Headers,
}

impl Default for ApiClientApp {
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
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    pub fn send_request(&mut self) {
        if self.request.url.is_empty() {
            self.error_message = Some("URL cannot be empty".to_string());
            return;
        }

        self.error_message = None;
        let request = self.request.clone();

        self.pending_request = Some(Promise::spawn_async(async move {
            crate::http::send_request(request).await
        }));
    }

    pub fn check_response(&mut self) {
        if let Some(promise) = &self.pending_request {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(response) => {
                        self.response = Some(response.clone());
                        self.error_message = None;
                    }
                    Err(err) => {
                        self.error_message = Some(err.clone());
                        self.response = None;
                    }
                }
                self.pending_request = None;
            }
        }
    }

    pub fn is_requesting(&self) -> bool {
        self.pending_request.is_some()
    }

    pub fn add_header(&mut self) {
        if !self.new_header_key.is_empty() {
            self.request.headers.push(crate::models::Header {
                key: self.new_header_key.clone(),
                value: self.new_header_value.clone(),
            });
            self.new_header_key.clear();
            self.new_header_value.clear();
        }
    }

    pub fn remove_header(&mut self, index: usize) {
        if index < self.request.headers.len() {
            self.request.headers.remove(index);
        }
    }

    pub fn format_request_body(&mut self) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.request.body) {
            self.request.body = serde_json::to_string_pretty(&json).unwrap_or_default();
        }
    }
}

impl eframe::App for ApiClientApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check for completed requests
        self.check_response();

        // Keep refreshing while request is pending
        if self.is_requesting() {
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
        }

        egui::CentralPanel::default().show(ctx, |ui: &mut egui::Ui| {
            ui.heading("API Client");
            ui.add_space(10.0);

            // Request panel
            ui::request_panel::render(self, ui);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Response panel
            ui::response_panel::render(self, ui);
        });
    }
}
