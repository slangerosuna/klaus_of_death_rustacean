use std::sync::Arc;

use eframe::egui;
use egui_wgpu::WgpuConfiguration;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use wgpu::{DeviceDescriptor, Features};

pub mod app;
pub mod device;

use app::App;
use device::GpuDevice;

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub worker_threads: usize,
    pub inner_size: [f32; 2],
}

lazy_static! {
    pub static ref RESOURCES_DIR: String = {
        let bin_dir = std::env::current_exe().expect("Can't find path to executable");
        let bin_dir = bin_dir.parent().unwrap();

        let resources_dir = format!("{}/kod_resources", bin_dir.display());

        resources_dir
    };
    pub static ref CONFIG: Config = {
        let config = get_resource_string("config.toml");
        toml::from_str(&config).unwrap()
    };
    pub static ref RT: Runtime = {
        Builder::new_multi_thread()
            .worker_threads(CONFIG.worker_threads)
            .enable_all()
            .build()
            .unwrap()
    };
}

pub fn get_resource_string(resource: &str) -> String {
    let path = format!("{}/{}", *RESOURCES_DIR, resource);
    std::fs::read_to_string(path).unwrap()
}

pub fn get_resource_bin(resource: &str) -> Vec<u8> {
    let path = format!("{}/{}", *RESOURCES_DIR, resource);
    std::fs::read(path).unwrap()
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size(&CONFIG.inner_size),
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: WgpuConfiguration {
            device_descriptor: Arc::new(|_| DeviceDescriptor {
                required_features: Features::default()
                    | Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES,
                ..Default::default()
            }),
            ..Default::default()
        },

        ..Default::default()
    };

    eframe::run_native(
        "Klaus of Death",
        options,
        Box::new(|cc| {
            let render_state = cc.wgpu_render_state.clone().unwrap();

            let shaders_dir = format!("{}/shaders", *RESOURCES_DIR,);
            let gpu = RT
                .block_on(GpuDevice::new(render_state, shaders_dir))
                .unwrap();

            let app = App::new(gpu);

            Ok(Box::new(app))
        }),
    )?;

    Ok(())
}
