#![feature(sync_unsafe_cell)]
#![feature(trait_upcasting)]
#![feature(downcast_unchecked)]

use std::sync::Arc;

use eframe::egui;
use egui_wgpu::WgpuConfiguration;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::{runtime::{Builder, Runtime}, spawn};
use wgpu::{DeviceDescriptor, Features};

pub mod app;
pub mod core;
pub mod networking;
pub mod render;
pub mod utils;

use app::App;
pub use core::*;
pub use std::any::Any;
use networking::{Networking, NetworkingCreationInfo};
use render::device::GpuDevice;

lazy_static! {
    pub static ref CONFIG: Config = get_resource_toml("config.toml");
    pub static ref RT: Runtime = {
        Builder::new_multi_thread()
            .worker_threads(CONFIG.worker_threads)
            .enable_all()
            .build()
            .unwrap()
    };
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub worker_threads: usize,
    pub inner_size: [f32; 2],
}

fn main() -> ! {
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
            let app = RT.block_on(App::new(render_state));

            Ok(Box::new(app))
        }),
    ).unwrap();

    std::process::exit(0);
}

struct SendBox<T>(std::pin::Pin<Box<T>>);

unsafe impl<T> Send for SendBox<T> {}

impl<T> SendBox<T> {
    unsafe fn new(t: T) -> Self {
        SendBox(Box::pin(t))
    }
}

impl<T> futures::Future for SendBox<T>
where
    T: futures::Future,
{
    type Output = T::Output;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}
