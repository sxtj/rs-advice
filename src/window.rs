use eframe::egui;
use tray_icon::{Icon, TrayIconBuilder, TrayIconEvent, MouseButton, MouseButtonState, menu::{Menu}};
use std::time::Duration;
use image::GenericImageView;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct Slip {
    advice: String,
}

#[derive(Deserialize)]
struct AdviceResponse {
    slip: Slip,
}

fn fetch_advice() -> String {
    let client = Client::new();
    client
        .get("https://api.adviceslip.com/advice")
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json::<AdviceResponse>())
        .map(|r| r.slip.advice)
        .expect("could not fetch advice")
}

fn load_icon() -> Icon {
    let image = image::load_from_memory(include_bytes!("../cylinder.png"))
        .expect("failed to load embedded tray icon");
    let (width, height) = image.dimensions();
    let rgba = image.into_rgba8().into_raw();
    Icon::from_rgba(rgba, width, height).expect("failed to create icon")
}

pub struct MyApp {
    _tray_icon: tray_icon::TrayIcon,
    is_visible: bool,
    needs_visibility_init: bool,
    window_pos: egui::Pos2,
    window_size: egui::Vec2,
    advice: String,
}

impl MyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {

        let icon = load_icon();
        let tray_menu = Menu::new();
        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("idea")
            .with_icon(icon)
            .build()
            .unwrap();

        let window_size = egui::Vec2::new(360.0, 240.0);
        let margin: f32 = 12.0;

        Self {
            _tray_icon: tray_icon,
            is_visible: true,
            needs_visibility_init: true,
            window_pos: egui::pos2(0.0, margin),
            window_size,
            advice: fetch_advice(),
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.needs_visibility_init {
            self.is_visible = ctx
                .input(|i| i.viewport().minimized)
                .map(|m| !m)
                .unwrap_or(true);
            self.needs_visibility_init = false;
        }

        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            match event {
                TrayIconEvent::Click { button, button_state, rect, .. } => {
                    if button != MouseButton::Left || button_state != MouseButtonState::Up {
                        continue;
                    }
                    let want_visible = !self.is_visible;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Visible(want_visible));
                    if want_visible {
                        let nppp = ctx.input(|i| i.viewport().native_pixels_per_point).unwrap_or(1.0);
                        let icon_x = (rect.position.x as f32) / nppp;
                        let icon_y = (rect.position.y as f32) / nppp;
                        let icon_w = (rect.size.width as f32) / nppp;
                        let icon_h = (rect.size.height as f32) / nppp;
                        let margin = 12.0;
                        let x = icon_x + icon_w - self.window_size.x - margin;
                        let y = icon_y + icon_h + margin;
                        self.window_pos = egui::pos2(x, y);
                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(self.window_size));
                        ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(self.window_pos));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                    }
                    self.is_visible = want_visible;
                    break;
                }
                _ => {}
            }
        }
        ctx.request_repaint_after(Duration::from_millis(100));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(&self.advice);
        });
    }
}

