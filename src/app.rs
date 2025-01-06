use std::pin::Pin;

use eframe::*;
use egui::*;

use crate::*;
use crate::utils::*;

pub struct App {
    pub game_state: Pin<Box<core::GameState>>,
    pub scheduler: Pin<Box<core::Scheduler>>,
    output_image: TextureId,
}

create_system!(rotate_system, get_rotate_system;
    uses Player, Transform);
pub async fn rotate_system(game_state: &mut GameState, t: f64, _dt: f64) {
    let mut player = &mut game_state
        .get_entities_with_mut::<Player>(Player::get_component_type())[0];
    let player = player
        .get_component_mut::<Transform>(Transform::get_component_type())
        .unwrap();

    player.rotation = t as f32;
}

impl App {
    pub async fn new(render_state: egui_wgpu::RenderState) -> Self {
        let scheduler = Scheduler::new(0.01);
        let mut scheduler = Box::pin(scheduler);
        let game_state = core::GameState::new(&mut *scheduler, &CONFIG);
        let mut game_state = Box::pin(game_state);

        let shaders_dir = format!("{}/shaders", *RESOURCES_DIR,);
        let (gpu, output_image) = GpuDevice::new(render_state, shaders_dir).await.unwrap();
        let networking = Networking::new(NetworkingCreationInfo {
            ..Default::default()
        });

        game_state.add_resource(gpu);
        game_state.add_resource(networking);

        scheduler.add_system_without_execution_order_generation(crate::render::get_init_system(), SystemType::Init);

        scheduler.add_system_without_execution_order_generation(crate::render::get_render_system(), SystemType::Update);
        scheduler.add_system_without_execution_order_generation(get_rotate_system(), SystemType::Update);

        scheduler.generate_execution_order();

        scheduler.init(&mut *game_state).await;

        /*let fixed_update_scheduler = unsafe { &*(&*scheduler as *const Scheduler) };
        let fixed_update_future = fixed_update_scheduler.loop_fixed_update(&mut *game_state as *mut _);
        let fixed_update_future = unsafe { SendBox::new(fixed_update_future) };

        spawn(fixed_update_future);*/ // TODO: Improve the scheduler so that fixed update can be run at the same time as update, rather than the current implementation with pre-defined task execution groups

        App {
            game_state,
            scheduler,
            output_image,
        }
    }
}

fn largest_16_9_rect(container: Rect) -> Rect {
    let container_width = container.width();
    let container_height = container.height();

    let max_width_based_on_height = container_height * (16.0 / 9.0);
    let max_height_based_on_width = container_width * (9.0 / 16.0);

    // Determine the actual width and height
    let (width, height) = if max_width_based_on_height <= container_width {
        (max_width_based_on_height, container_height)
    } else {
        (container_width, max_height_based_on_width)
    };

    let x_center = container.center().x;
    let y_center = container.center().y;

    let min_x = x_center - width / 2.0;
    let min_y = y_center - height / 2.0;
    let max_x = x_center + width / 2.0;
    let max_y = y_center + height / 2.0;

    Rect::from_min_max(egui::pos2(min_x, min_y), egui::pos2(max_x, max_y))
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        RT.block_on(self.scheduler.update(&mut *self.game_state));

        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_rect = ui.max_rect();
            let rect = largest_16_9_rect(panel_rect);

            let image = Image::new((self.output_image, rect.size()));

            ui.put(rect, image);
        });

        ctx.request_repaint();
    }
}
