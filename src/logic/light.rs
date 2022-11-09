use super::*;

/// A directional point source of light.
pub struct Light {
    pub fov: f32,
    pub pos: Vec3<f32>,
    pub rot_h: f32,
    pub rot_v: f32,
}

impl geng::AbstractCamera3d for Light {
    fn view_matrix(&self) -> Mat4<f32> {
        Mat4::rotate_x(-self.rot_v)
            * Mat4::rotate_y(-self.rot_h)
            * Mat4::rotate_x(-f32::PI / 2.0)
            * Mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: Vec2<f32>) -> Mat4<f32> {
        Mat4::perspective(
            self.fov,
            framebuffer_size.x / framebuffer_size.y,
            0.1,
            100.0,
        )
    }
}
