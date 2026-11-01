use anyhow::Result;
use eframe::egui;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use crate::{
    config::{Config, Provider},
    platform, providers,
};

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Magic Pointer")
            .with_inner_size([900.0, 680.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Magic Pointer",
        options,
        Box::new(|_| Ok(Box::new(MagicPointerApp::load()))),
    )
    .map_err(|error| anyhow::anyhow!("starting desktop window: {error}"))
}

struct MagicPointerApp {
    config: Config,
    question: String,
    answer: String,
    status: String,
    setup_required: bool,
    show_settings: bool,
    dark_mode: bool,
    pending: Option<Receiver<Result<String, String>>>,
}

impl MagicPointerApp {
    fn load() -> Self {
        match Config::load() {
            Ok(config) => Self {
                setup_required: config.api_key.trim().is_empty(),
                config,
                question: String::new(),
                answer: String::new(),
                status: String::new(),
                show_settings: false,
                dark_mode: true,
                pending: None,
            },
            Err(error) => Self {
                config: Config::default(),
                question: String::new(),
                answer: String::new(),
                status: format!("Configuration error: {error:#}"),
                setup_required: true,
                show_settings: true,
                dark_mode: true,
                pending: None,
            },
        }
    }

    fn save_config(&mut self) {
        match self.config.save() {
            Ok(()) => {
                self.setup_required = self.config.api_key.trim().is_empty();
                self.status = "Settings saved.".into();
            }
            Err(error) => self.status = format!("Could not save settings: {error:#}"),
        }
    }

    fn ask(&mut self, image: Option<Vec<u8>>) {
        let question = self.question.trim().to_owned();
        if question.is_empty() {
            self.status = "Enter a question first.".into();
            return;
        }
        if self.config.api_key.trim().is_empty() {
            self.setup_required = true;
            self.status = "Add an API key in Settings before asking.".into();
            return;
        }
        let config = self.config.clone();
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let result = providers::ask(&config, &question, image.as_deref())
                .map_err(|error| format!("{error:#}"));
            let _ = sender.send(result);
        });
        self.pending = Some(receiver);
        self.status = "Asking the selected AI provider…".into();
    }

    fn capture_and_ask(&mut self) {
        match platform::capture_screen() {
            Ok(image) => self.ask(Some(image)),
            Err(error) => self.status = format!("Screen capture failed: {error:#}"),
        }
    }

    fn show_setup(&mut self, ui: &mut egui::Ui) {
        ui.heading("Welcome to Magic Pointer");
        ui.label("Complete the setup once. Your API key is stored in your user configuration.");
        ui.add_space(12.0);
        self.show_settings_fields(ui);
        ui.add_space(12.0);
        if ui.button("Save and continue").clicked() {
            self.save_config();
        }
    }

    fn show_settings_fields(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Provider");
            egui::ComboBox::from_id_salt("provider")
                .selected_text(self.config.provider.as_str())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.config.provider, Provider::Gemini, "Gemini");
                    ui.selectable_value(&mut self.config.provider, Provider::Openai, "OpenAI");
                    ui.selectable_value(
                        &mut self.config.provider,
                        Provider::Anthropic,
                        "Anthropic",
                    );
                    ui.selectable_value(&mut self.config.provider, Provider::Nvidia, "NVIDIA");
                });
        });
        ui.horizontal(|ui| {
            ui.label("Model");
            ui.text_edit_singleline(&mut self.config.model);
        });
        ui.horizontal(|ui| {
            ui.label("API key");
            ui.add(egui::TextEdit::singleline(&mut self.config.api_key).password(true));
        });
        ui.horizontal(|ui| {
            ui.label("Request timeout (seconds)");
            ui.add(egui::DragValue::new(&mut self.config.timeout_seconds).range(1..=600));
        });
    }
}

impl eframe::App for MagicPointerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(12.0, 12.0);
        style.spacing.button_padding = egui::vec2(16.0, 9.0);
        style.visuals.widgets.inactive.rounding = 8.0.into();
        style.visuals.widgets.hovered.rounding = 8.0.into();
        style.visuals.widgets.active.rounding = 8.0.into();
        ctx.set_style(style);
        if self.dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        } else {
            ctx.set_visuals(egui::Visuals::light());
        }

        if let Some(receiver) = &self.pending {
            match receiver.try_recv() {
                Ok(Ok(answer)) => {
                    self.answer = answer;
                    self.status = "Response received.".into();
                    self.pending = None;
                }
                Ok(Err(error)) => {
                    self.status = format!("Request failed: {error}");
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.status = "Request worker stopped unexpectedly.".into();
                    self.pending = None;
                }
                Err(mpsc::TryRecvError::Empty) => ctx.request_repaint(),
            }
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(egui::RichText::new("Magic Pointer").size(24.0).strong());
                ui.separator();
                ui.label(
                    egui::RichText::new("Your visual AI companion")
                        .italics()
                        .color(ui.visuals().weak_text_color()),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .button(if self.dark_mode {
                            "☀ Light"
                        } else {
                            "☾ Dark"
                        })
                        .clicked()
                    {
                        self.dark_mode = !self.dark_mode;
                    }
                });
            });
        });

        egui::SidePanel::left("navigation")
            .resizable(false)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.add_space(12.0);
                ui.label(egui::RichText::new("WORKSPACE").small().strong());
                ui.add_space(4.0);
                if ui
                    .selectable_label(!self.show_settings, "  Ask Magic Pointer")
                    .clicked()
                {
                    self.show_settings = false;
                }
                if !self.setup_required && ui.button("Settings").clicked() {
                    self.show_settings = true;
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        egui::RichText::new("Native Rust desktop app")
                            .small()
                            .color(ui.visuals().weak_text_color()),
                    );
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(12.0);
            if self.setup_required || self.show_settings {
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .inner_margin(egui::Margin::same(24.0))
                    .show(ui, |ui| self.show_setup(ui));
                if !self.setup_required {
                    ui.separator();
                    if ui.button("Back to question").clicked() {
                        self.show_settings = false;
                    }
                }
                return;
            }

            ui.heading("Ask anything");
            ui.label(
                egui::RichText::new("Ask about your screen or start with a normal question.")
                    .color(ui.visuals().weak_text_color()),
            );
            ui.add_space(8.0);
            egui::Frame::group(ui.style())
                .fill(ui.visuals().faint_bg_color)
                .inner_margin(egui::Margin::same(18.0))
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.question)
                            .hint_text("What would you like to know?")
                            .desired_rows(4)
                            .desired_width(f32::INFINITY),
                    );
                });
            ui.horizontal(|ui| {
                let busy = self.pending.is_some();
                if ui.add_enabled(!busy, egui::Button::new("Ask")).clicked() {
                    self.ask(None);
                }
                if ui
                    .add_enabled(!busy, egui::Button::new("Capture screen and ask"))
                    .clicked()
                {
                    self.capture_and_ask();
                }
            });
            ui.add_space(12.0);
            ui.heading("Response");
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(280.0)
                .show(ui, |ui| {
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().faint_bg_color)
                        .inner_margin(egui::Margin::same(18.0))
                        .show(ui, |ui| {
                            ui.label(if self.answer.is_empty() {
                                "The response will appear here.".into()
                            } else {
                                self.answer.clone()
                            });
                        });
                });
            if !self.status.is_empty() {
                ui.separator();
                ui.label(&self.status);
            }
        });
    }
}
