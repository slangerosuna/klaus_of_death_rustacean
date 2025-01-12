use crate::*;
use wgpu::*;
use wgpu::util::*;

pub struct Map{
    pub buffer: Buffer,
    pub bitmap: Box<[u8; 64 * 64 / 8]>,
}
impl_resource!(Map, 2);

impl Map {
    pub fn load(gpu: &GpuDevice, resource: &str) -> Self {
        let data = get_resource_string(resource);
        let data = data.split_ascii_whitespace().map(|x| x.parse::<u32>().expect("Invalid Map File")).collect::<Vec<_>>();

        let buffer = gpu.render_state.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(data.as_slice()),
            usage: BufferUsages::STORAGE,
        });

        let mut bitmap = [0u8; 64 * 64 / 8];

        for (i, byte) in data.iter().enumerate() {
            bitmap[i / 8] |= ((*byte != 0) as u8) << ((i as u8) % 8);
        }


        Map{
            buffer,
            bitmap: Box::new(bitmap),
        }
    }

    /*pub fn intersects_point(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= 64 || y >= 64 {
            return true;
        }

        let byte = self.bitmap[(y as usize) * 8 + (x as usize) / 8];
        let bit = (byte >> (x % 8)) & 1;

        bit == 1
    }

    pub fn intersects_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<Direction>{
        let mut dirs = Vec::new();

        let min_x = (x - w/2.).ceil() as i32;
        let max_x = (x + w/2.).ceil() as i32;
        let min_y = (y - h/2.).ceil() as i32;
        let max_y = (y + h/2.).ceil() as i32;

        for i in min_x..=max_x {
            for j in min_y..=max_y {
                if self.intersects_point(i, j) {
                    let x_dif = (x - i as f32).abs();
                    let y_dif = (y - j as f32).abs();

                    if x_dif > y_dif {
                        if x > i as f32 {
                            dirs.push(Direction::Right);
                        } else {
                            dirs.push(Direction::Left);
                        }
                    } else {
                        if y > j as f32 {
                            dirs.push(Direction::Down);
                        } else {
                            dirs.push(Direction::Up);
                        }
                    }
                }
            }
        }

        dirs
    }*/

    pub fn intersects_rect(&self, x: f32, y: f32, w: f32, h: f32) -> Vec<Direction> {
        let mut dirs = Vec::new();

        let left = (x - w / 2.0).floor() as i32;
        let right = (x + w / 2.0).floor() as i32;
        let top = (y - h / 2.0).floor() as i32;
        let bottom = (y + h / 2.0).floor() as i32;

        for i in left..=right {
            for j in top..=bottom {
            if i < 0 || j < 0 || i >= 64 || j >= 64 {
                continue;
            }

            let byte = self.bitmap[(j as usize) * 8 + (i as usize) / 8];
            let bit = (byte >> (i % 8)) & 1;

            if bit == 1 {
                let i = i as f32 + 0.5;
                let j = j as f32 + 0.5;
                let x_dif = (x - i as f32).abs();
                let y_dif = (y - j as f32).abs();

                if x_dif > y_dif {
                    if x > i as f32 {
                        dirs.push(Direction::Right);
                    } else {
                        dirs.push(Direction::Left);
                    }
                } else {
                    if y > j as f32 {
                        dirs.push(Direction::Down);
                    } else {
                        dirs.push(Direction::Up);
                    }
                }
            }
        }
    }

        dirs
    }
}

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}