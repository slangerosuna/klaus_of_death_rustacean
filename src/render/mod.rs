pub mod device;
pub(crate) mod map;

mod textures_loader;
use textures_loader::*;

use crate::*;
use wgpu::*;

pub struct RenderState {
    ray_pipeline: ComputePipeline,
    player_buffer: Buffer,
    ray_bind_group: BindGroup,
    render_pipeline: ComputePipeline,
    render_bind_group: BindGroup,
}
impl_resource!(RenderState, 3);

create_system!(init, get_init_system);
async fn init(game_state: &mut GameState, _time: f64, _dt: f64) {
    let gpu = game_state.get_resource::<GpuDevice>().unwrap();
    let map = Map::load(gpu, "map/default.map");

    let ray_pipeline_desc = ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: gpu.shaders.get("ray_calc").unwrap(),
        entry_point: "main",
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    };

    let ray_pipeline = gpu.render_state.device.create_compute_pipeline(&ray_pipeline_desc);

    let player_buffer = gpu.render_state.device.create_buffer(&BufferDescriptor{
        label: None,
        size: size_of::<f32>() as u64 * 4,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    let bind_group_layout = ray_pipeline.get_bind_group_layout(0);

    let Map{ buffer, .. } = &map;
    let tex = &gpu.output_tex;
    let view = tex.create_view(&TextureViewDescriptor::default());

    let ray_output_buffer = gpu.render_state.device.create_buffer(&BufferDescriptor{
        label: None,
        size: 1920 * 4 * 4,
        usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let ray_bind_group = gpu.render_state.device.create_bind_group(&BindGroupDescriptor{
        label: None,
        layout: &bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: player_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: ray_output_buffer.as_entire_binding(),
            },
        ],
    });

    let render_pipeline_desc = ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: gpu.shaders.get("render").unwrap(),
        entry_point: "main",
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    };

    let render_pipeline = gpu.render_state.device.create_compute_pipeline(&render_pipeline_desc);

    let render_bind_group_layout = render_pipeline.get_bind_group_layout(0);

    let texture_loader: TextureLoader = get_resource_string("map/textures.txt").into();
    let textures = texture_loader.load(gpu).await;

    let textures = textures.create_view(&TextureViewDescriptor::default());

    let render_bind_group = gpu.render_state.device.create_bind_group(&BindGroupDescriptor{
        label: None,
        layout: &render_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: ray_output_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::TextureView(&textures), // var textures : texture_storage_2d_array<rgba8unorm, read>;
            },
            BindGroupEntry {
                binding: 2,
                resource: BindingResource::TextureView(&view),
            },
        ],
    });

    game_state.add_resource(map);
    game_state.add_resource(RenderState { ray_pipeline, player_buffer, ray_bind_group, render_pipeline, render_bind_group });

    let entity = game_state.create_entity("Player".to_string());
    entity.add_component(game_state, Player, Player::get_component_type());
    entity.add_component(game_state, Transform { position: [15.0, 20.0], rotation: 0.0, scale: [1.0, 1.0]}, Transform::get_component_type())
}

use map::Map;
use crate::utils::*;

create_system!(render, get_render_system;
    uses RenderState, Map, Player, Transform);
pub async fn render(game_state: &mut GameState, _t: f64, _dt: f64) {
    let gpu = game_state.get_resource::<GpuDevice>().unwrap();
    let player = game_state
        .get_entities_with::<Player>(Player::get_component_type())
        .first()
        .unwrap()
        .get_component::<Transform>(Transform::get_component_type())
        .unwrap();
    let player = [player.position[0], player.position[1], f32::sin(player.rotation), f32::cos(player.rotation)];
    let RenderState { ray_pipeline, player_buffer, ray_bind_group, render_pipeline, render_bind_group} = game_state.get_resource::<RenderState>().unwrap();

    gpu.render_state.queue.write_buffer(player_buffer, 0, bytemuck::cast_slice(&player));

    let mut encoder = gpu.render_state.device.create_command_encoder(&CommandEncoderDescriptor::default());

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(ray_pipeline);
        compute_pass.set_bind_group(0, &ray_bind_group, &[]);
        compute_pass.dispatch_workgroups(1920 / 8, 1, 1);
    }
    
    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(render_pipeline);
        compute_pass.set_bind_group(0, &render_bind_group, &[]);
        compute_pass.dispatch_workgroups(1920 / 16, 1080 / 16, 1);
    }

    gpu.render_state.queue.submit(std::iter::once(encoder.finish()));
}