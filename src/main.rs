#![feature(sync_unsafe_cell)]
#![feature(trait_upcasting)]
#![feature(downcast_unchecked)]

use std::sync::Arc;
use std::ops::Deref;

use eframe::egui;
use egui_wgpu::WgpuConfiguration;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::runtime::{Builder, Runtime};
use wgpu::{DeviceDescriptor, Features};

pub mod app;
pub mod core;
pub mod networking;
pub mod render;

use app::App;
pub use core::*;
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
            let app = App::new();

            RT.spawn(unsafe { SendBox::new(init_game(render_state, app.clone())) });

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
    T: futures::Future + 'static,
{
    type Output = T::Output;

    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context,
    ) -> std::task::Poll<Self::Output> {
        self.0.as_mut().poll(cx)
    }
}

async fn init_game(render_state: egui_wgpu::RenderState, app: App) {
    let mut scheduler = Scheduler::new(0.01);
    let mut game_state = core::GameState::new(&mut scheduler, &CONFIG);

    let shaders_dir = format!("{}/shaders", *RESOURCES_DIR,);
    let gpu = RT
        .block_on(GpuDevice::new(render_state, shaders_dir))
        .unwrap();
    let networking = Networking::new(NetworkingCreationInfo {
        ..Default::default()
    });

    game_state.add_resource(gpu);
    game_state.add_resource(app.clone());
    game_state.add_resource(networking);

    scheduler.init(&mut game_state).await;

    let fixed_update_scheduler = unsafe { &*(&scheduler as *const Scheduler) };
    let fixed_update_future = fixed_update_scheduler.loop_fixed_update(&mut game_state as *mut _);
    let mut fixed_update_future = unsafe { SendBox::new(fixed_update_future) };
    let fixed_update_future =
        unsafe { std::pin::Pin::new_unchecked(&mut *(&mut fixed_update_future as *mut _)) };

    
}
