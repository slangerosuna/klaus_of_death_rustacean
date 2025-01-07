pub mod device;
pub(crate) mod map;

use crate::*;
use wgpu::*;

pub struct RenderState {
    pipeline: ComputePipeline,
    player_buffer: Buffer,
    bind_group: BindGroup,
}
impl_resource!(RenderState, 3);

create_system!(init, get_init_system);
async fn init(game_state: &mut GameState, _time: f64, _dt: f64) {
    let gpu = game_state.get_resource::<GpuDevice>().unwrap();
    let map = Map::load(gpu, "default.map");

    let pipeline_desc = ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: gpu.shaders.get("main").unwrap(),
        entry_point: "main",
        compilation_options: PipelineCompilationOptions::default(),
        cache: None,
    };

    let pipeline = gpu.render_state.device.create_compute_pipeline(&pipeline_desc);

    let player_buffer = gpu.render_state.device.create_buffer(&BufferDescriptor{
        label: None,
        size: size_of::<f32>() as u64 * 4,
        usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
        mapped_at_creation: false,
    });

    let bind_group_layout = pipeline.get_bind_group_layout(0);

    let Map(buffer) = &map;
    let tex = &gpu.output_tex;
    let view = tex.create_view(&TextureViewDescriptor::default());

    let bind_group = gpu.render_state.device.create_bind_group(&BindGroupDescriptor{
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
                resource: BindingResource::TextureView(&view),
            },
        ],
    });

    game_state.add_resource(map);
    game_state.add_resource(RenderState { pipeline, player_buffer, bind_group });

    let entity = game_state.create_entity("Player".to_string());
    entity.add_component(game_state, Player, Player::get_component_type());
    entity.add_component(game_state, Transform { position: [50.0, 50.0], rotation: 0.0, scale: [1.0, 1.0]}, Transform::get_component_type())
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
    let RenderState { pipeline, player_buffer, bind_group} = game_state.get_resource::<RenderState>().unwrap();

    gpu.render_state.queue.write_buffer(player_buffer, 0, bytemuck::cast_slice(&player));

    let mut encoder = gpu.render_state.device.create_command_encoder(&CommandEncoderDescriptor::default());

    {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        compute_pass.set_pipeline(pipeline);
        compute_pass.set_bind_group(0, &bind_group, &[]);
        compute_pass.dispatch_workgroups(320 / 8, 1, 1);
    }

    gpu.render_state.queue.submit(std::iter::once(encoder.finish()));
}