use std::sync::Arc;

use eframe::*;
use egui::*;

use crate::impl_resource;

struct AppInternal {}

impl AppInternal {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct App(Arc<AppInternal>);

impl App {
    pub fn new() -> Self {
        Self(Arc::new(AppInternal::new()))
    }
}

impl_resource!(App, 0);

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.horizontal(|ui| {
                ui.label("This is a simple egui app.");
            });
        });
    }
}
