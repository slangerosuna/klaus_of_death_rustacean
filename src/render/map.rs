use crate::*;
use wgpu::*;
use wgpu::util::*;

pub struct Map(pub Buffer);
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

        Map(buffer)
    }
}
