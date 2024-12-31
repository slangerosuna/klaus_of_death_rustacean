use std::pin::Pin;

use eframe::*;
use egui::*;

use crate::*;

pub struct App {
    pub game_state: Pin<Box<core::GameState>>,
    pub scheduler: Pin<Box<core::Scheduler>>,
}

impl App {
    pub async fn new(render_state: egui_wgpu::RenderState) -> Self {
        let scheduler = Scheduler::new(0.01);
        let mut scheduler = Box::pin(scheduler);
        let game_state = core::GameState::new(&mut *scheduler, &CONFIG);
        let mut game_state = Box::pin(game_state);

        let shaders_dir = format!("{}/shaders", *RESOURCES_DIR,);
        let gpu = GpuDevice::new(render_state, shaders_dir).await.unwrap();
        let networking = Networking::new(NetworkingCreationInfo {
            ..Default::default()
        });

        game_state.add_resource(gpu);
        game_state.add_resource(networking);

        scheduler.init(&mut *game_state).await;

        let fixed_update_scheduler = unsafe { &*(&*scheduler as *const Scheduler) };
        let fixed_update_future = fixed_update_scheduler.loop_fixed_update(&mut *game_state as *mut _);
        let fixed_update_future = unsafe { SendBox::new(fixed_update_future) };

        spawn(fixed_update_future);

        App {
            game_state,
            scheduler,
        }
    }
}


impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        RT.block_on(self.scheduler.update(&mut *self.game_state));

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.horizontal(|ui| {
                ui.label("This is a simple egui app.");
            });
        });
    }
}
