const SCREEN_WIDTH: u32 = 1920;
const MAP_SIZE: u32 = 64; // Size of the game map (MAP_SIZE x MAP_SIZE)

@group(0) @binding(0) var<storage, read> map: array<u32>;
@group(0) @binding(1) var<uniform> player: vec4<f32>; // Player position (x, y) and direction (dirX, dirY)

struct OutputData {
    @location(0) tex: i32,
    @location(1) tex_coord: f32,
    @location(2) depth: f32,
    @location(3) side: i32,
}
@group(0) @binding(2) var<storage, read_write> out_buffer: array<OutputData>; 

@compute @workgroup_size(8, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x; 

    let player_pos = vec2<f32>(player.x, player.y);
    let player_dir = vec2<f32>(player.z, player.w);

    let camera_plane = vec2<f32>(-player_dir.y, player_dir.x) * 0.66;

    let screen_x = (2.0 * f32(x) / f32(SCREEN_WIDTH)) - 1.0;
    let ray_dir = player_dir + screen_x * camera_plane;

    var map_x = i32(player_pos.x);
    var map_y = i32(player_pos.y);

    var delta_dist : vec2<f32>;

    if ray_dir.x == 0.0 {
        delta_dist.x = 1e30;
    } else {
        delta_dist.x = abs(1.0 / ray_dir.x);
    }
    if ray_dir.y == 0.0 {
        delta_dist.y = 1e30;
    } else {
        delta_dist.y = abs(1.0 / ray_dir.y);
    }

    var side_dist = vec2<f32>(0.0, 0.0);
    var step = vec2<i32>(0, 0);

    if (ray_dir.x < 0.0) {
        step.x = -1;
        side_dist.x = (player_pos.x - f32(map_x)) * delta_dist.x;
    } else {
        step.x = 1;
        side_dist.x = (f32(map_x + 1) - player_pos.x) * delta_dist.x;
    }

    if (ray_dir.y < 0.0) {
        step.y = -1;
        side_dist.y = (player_pos.y - f32(map_y)) * delta_dist.y;
    } else {
        step.y = 1;
        side_dist.y = (f32(map_y + 1) - player_pos.y) * delta_dist.y;
    }

    var hit = false;
    var side = 0; // 0: X-side, 1: Y-side
    while (!hit) {
        if (side_dist.x < side_dist.y) {
            side_dist.x += delta_dist.x;
            map_x += step.x;
            side = 0;
        } else {
            side_dist.y += delta_dist.y;
            map_y += step.y;
            side = 1;
        }

        let map_index = map_y * i32(MAP_SIZE) + map_x;
        if (map[map_index] > 0) {
            hit = true;
        }
    }

    var perp_wall_dist : f32;
    if side == 0 {
        perp_wall_dist = (f32(map_x) - player_pos.x + (1.0 - f32(step.x)) / 2.0) / ray_dir.x;
    } else {
        perp_wall_dist = (f32(map_y) - player_pos.y + (1.0 - f32(step.y)) / 2.0) / ray_dir.y;
    }

    var tex_positon : f32;
    if side == 0 {
        tex_positon = player_pos.y + perp_wall_dist * ray_dir.y;
    } else {
        tex_positon = player_pos.x + perp_wall_dist * ray_dir.x;
    }

    tex_positon -= floor(tex_positon);

    let out_data = OutputData(i32(map[map_y * i32(MAP_SIZE) + map_x] - 1), tex_positon, perp_wall_dist, side);
    out_buffer[x] = out_data;
}