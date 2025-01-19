const SCREEN_WIDTH = 1920;
const SCREEN_HEIGHT = 1080;

const ROOF_COLOR = vec4<f32>(0.16, 0.16, 0.18, 1.0);
const FLOOR_COLOR = vec4<f32>(0.2, 0.15, 0.06, 1.0);

const TEX_BOUNDS = vec2<u32>(16, 16);

struct InputData {
    @location(0) tex: i32,
    @location(1) tex_coord: f32,
    @location(2) depth: f32,
    @location(3) side: i32,
}
@group(0) @binding(0) var<storage, read> in_data : array<InputData>;
@group(0) @binding(1) var textures : texture_storage_2d_array<rgba8unorm, read>;
@group(0) @binding(2) var frame_buffer : texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) id : vec3<u32>) {
    let data = in_data[id.x];
    let y = id.y;

    let line_height = i32(f32(SCREEN_HEIGHT) / data.depth);

    let draw_start = -line_height / 2 + i32(SCREEN_HEIGHT) / 2;
    let draw_end = line_height / 2 + i32(SCREEN_HEIGHT) / 2;
    
    let tex_position = f32(i32(y) - draw_start) / f32(draw_end - draw_start);
    if tex_position <= 0.0 {
        textureStore(frame_buffer, vec2<u32>(id.x, y), ROOF_COLOR);
        
        return;
    } else if tex_position >= 1.0 {
        textureStore(frame_buffer, vec2<u32>(id.x, y), FLOOR_COLOR);
        
        return;
    }

    let uv = vec2<f32>(data.tex_coord, tex_position);
    let tex_coords = vec2<u32>(uv * vec2<f32>(TEX_BOUNDS));
    var color = textureLoad(textures, tex_coords, data.tex);
    if data.side == 1 {
        color = color * 0.8;
    }

    textureStore(frame_buffer, vec2<u32>(id.x, y), color);
}