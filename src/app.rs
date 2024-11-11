use crate::device::GpuDevice;
use egui::*;
use eframe::*;

pub struct App {
    pub gpu: GpuDevice,
}

impl App {
    pub fn new(gpu: GpuDevice) -> Self {
        Self { gpu }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, frame: &mut eframe::Frame) {
        
    }
}
