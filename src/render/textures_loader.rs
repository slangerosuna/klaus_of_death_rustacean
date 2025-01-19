use serde::{Deserialize, Serialize};
use wgpu::util::DeviceExt;
use wgpu::*;

use crate::*;

#[derive(Serialize, Deserialize)]
pub struct TextureLoader {
    pub textures: Vec<String>,
}

impl From<String> for TextureLoader {
    fn from(s: String) -> Self {
        let textures = s
            .lines()
            .map(|line| line.to_string())
            .collect::<Vec<String>>();

        TextureLoader { textures }
    }
}

impl TextureLoader {
    pub async fn load(self, gpu: &GpuDevice) -> Texture {
        let mut textures = Vec::new();

        for texture in &self.textures {
            let raw_data = get_resource_bin(&format!("map/textures/{}", texture));
            let texture = image::load_from_memory(&raw_data).unwrap().to_rgba8();

            textures.push(texture);
        }

        let len = textures.len();

        let textures = textures
            .iter()
            .map(|texture| {
                let texture = texture.as_raw();
                texture
            })
            .flatten()
            .cloned()
            .collect::<Vec<u8>>();

        let texture = gpu.render_state.device.create_texture_with_data(
            &gpu.render_state.queue,
            &TextureDescriptor {
                label: None,
                size: Extent3d {
                    width: 16,
                    height: 16,
                    depth_or_array_layers: len as u32,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::COPY_DST | TextureUsages::STORAGE_BINDING,
                view_formats: &[TextureFormat::Rgba8Unorm],
            },
            util::TextureDataOrder::LayerMajor,
            &textures,
        );

        texture
    }
}
