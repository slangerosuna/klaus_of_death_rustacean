// Constants
const SCREEN_WIDTH: u32 = 320;  // Screen resolution width
const SCREEN_HEIGHT: u32 = 200; // Screen resolution height
const MAP_SIZE: u32 = 64;       // Size of the game map
var<private> COLORS: array<vec4<f32>, 8> = array(
    vec4<f32>(1.0, 0.0, 0.0, 1.0), // Red
    vec4<f32>(0.0, 1.0, 0.0, 1.0), // Green
    vec4<f32>(0.0, 0.0, 1.0, 1.0), // Blue
    vec4<f32>(1.0, 1.0, 0.0, 1.0), // Yellow
    vec4<f32>(1.0, 0.0, 1.0, 1.0), // Magenta
    vec4<f32>(0.0, 1.0, 1.0, 1.0), // Cyan
    vec4<f32>(1.0, 1.0, 1.0, 1.0), // White
    vec4<f32>(0.0, 0.0, 0.0, 1.0)  // Black 
);
 
// Input Buffers
@group(0) @binding(0) var<storage, read> map: array<u32>; // Game map (2D grid as a 1D array)
@group(0) @binding(1) var<uniform> player: vec4<f32>;     // Player position (x, y) and direction (dirX, dirY)
@group(0) @binding(2) var frame_buffer: texture_storage_2d<rgba8unorm, write>;

// Compute Shader
@compute @workgroup_size(16, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x; // Column index
    if (x >= SCREEN_WIDTH) {
        return; // Ensure we don't process out-of-bounds columns
    }

    // Player information
    let player_pos = vec2<f32>(player.x, player.y);
    let player_dir = vec2<f32>(player.z, player.w);

    // Camera plane for FOV
    let camera_plane = vec2<f32>(-player_dir.y, player_dir.x) * 0.66;

    // Calculate the ray direction for this column
    let screen_x = (2.0 * f32(x) / f32(SCREEN_WIDTH)) - 1.0;
    let ray_dir = player_dir + screen_x * camera_plane;

    // Raycasting algorithm
    var map_x = i32(player_pos.x);
    var map_y = i32(player_pos.y);

    var delta_dist : vec2<f32>;

    if ray_dir.x == 0.0 {
        delta_dist.x = 1e30;
    } else {
        delta_dist.x = (1.0 / ray_dir.x);
    }
    if ray_dir.y == 0.0 {
        delta_dist.y = 1e30;
    } else {
        delta_dist.y = (1.0 / ray_dir.y);
    }

    var side_dist = vec2<f32>(0.0, 0.0);
    var step = vec2<i32>(0, 0);

    // Determine the step direction and initial sideDist
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

    // Perform DDA (Digital Differential Analysis)
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

        // Check if we've hit a wall
        let map_index = map_y * i32(MAP_SIZE) + map_x;
        if (map[map_index] > 0) {
            hit = true;
        }
    }

    // Calculate the distance to the wall
    var perp_wall_dist : f32;
    if side == 0 {
        perp_wall_dist = (f32(map_x) - player_pos.x + (1.0 - f32(step.x)) / 2.0) / ray_dir.x;
    } else {
        perp_wall_dist = (f32(map_y) - player_pos.y + (1.0 - f32(step.y)) / 2.0) / ray_dir.y;
    }

    // Calculate the height of the wall slice
    let line_height = i32(f32(SCREEN_HEIGHT) / perp_wall_dist);

    // Calculate the top and bottom of the slice
    let draw_start = u32(max(0, -line_height / 2 + i32(SCREEN_HEIGHT) / 2));
    let draw_end = u32(min(i32(SCREEN_HEIGHT), line_height / 2 + i32(SCREEN_HEIGHT) / 2));

    // Get the wall color (using map value)
    let color_idx = map[map_y * i32(MAP_SIZE) + map_x];
    let color = COLORS[color_idx];

    // Write the column to the framebuffer
    for (var y = draw_start; y < draw_end; y++) {
        let pixel_index = y * SCREEN_WIDTH + x;
        textureStore(frame_buffer, vec2<u32>(x, y), color);
    }
}