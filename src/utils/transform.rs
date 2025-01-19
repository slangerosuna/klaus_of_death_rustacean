use crate::*;

#[derive(Debug)]
pub struct Transform {
    pub position: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],
}
impl_component!(Transform, 0);
