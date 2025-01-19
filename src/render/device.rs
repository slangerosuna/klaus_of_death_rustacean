use egui_wgpu::RenderState;
use std::{borrow::Borrow, collections::HashMap, path::PathBuf};
use wgpu::*;

use crate::impl_resource;

pub struct GpuDevice {
    pub render_state: RenderState,
    pub shaders: HashMap<String, ShaderModule>,
    pub output_tex: wgpu::Texture,
}
impl_resource!(GpuDevice, 1);

#[inline]
pub fn pad_to_multiple_of_256(n: u32) -> u32 {
    (n + 255) & !255
}

fn gather_all_files(root: PathBuf) -> Vec<PathBuf> {
    let read_dir = std::fs::read_dir(root).unwrap();
    let mut files = Vec::new();

    for entry in read_dir {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            files.extend(gather_all_files(path.clone()));
        } else {
            files.push(path);
        }
    }

    files
}

impl GpuDevice {
    pub async fn new(
        render_state: RenderState,
        shaders_dir: String,
    ) -> Option<(Self, egui::TextureId)> {
        let mut shaders = HashMap::new();

        let files = gather_all_files(PathBuf::from(&shaders_dir));

        for file in files {
            let file_extension = file.extension().unwrap().to_str().unwrap().to_string();

            let shader = match file_extension.as_str() {
                "wgsl" => render_state
                    .device
                    .create_shader_module(ShaderModuleDescriptor {
                        label: None,
                        source: ShaderSource::Wgsl(
                            std::fs::read_to_string(file.clone()).unwrap().into(),
                        ),
                    }),
                _ => continue,
            };

            let relative_file = file
                .strip_prefix(&shaders_dir)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .strip_suffix(&format!(".{}", file_extension))
                .unwrap()
                .to_string();

            // replace `\\` with `/` for windows`
            let relative_file = relative_file.replace("\\", "/");

            #[cfg(debug_assertions)]
            print!("Loaded shader: {}\n", relative_file);
            shaders.insert(relative_file, shader);
        }

        let size = (1920, 1080);

        let output_tex = render_state.device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d {
                width: size.0,
                height: size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::all(),
            view_formats: &[TextureFormat::Rgba8Unorm],
        });

        let texture_view = output_tex.create_view(&Default::default());
        let tex_id = {
            let mut renderer = render_state.renderer.write();
            renderer.register_native_texture(
                render_state.device.borrow(),
                &texture_view,
                FilterMode::Nearest,
            )
        };

        Some((
            Self {
                render_state,
                shaders,
                output_tex,
            },
            tex_id,
        ))
    }
}
