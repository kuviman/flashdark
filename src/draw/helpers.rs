use super::*;

impl Game {
    pub fn draw_texture(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        program: &ugli::Program,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        let size = vec2(
            size * texture.size().x as f32 / texture.size().y as f32,
            size,
        );
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: Vec2<f32>,
        }
        ugli::draw(
            framebuffer,
            program,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
                vec![
                    Vertex {
                        a_pos: vec2(0.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    Vertex {
                        a_pos: vec2(0.0, 1.0),
                    },
                ]
            }),
            (
                ugli::uniforms! {
                    u_pos: pos,
                    u_size: size,
                    u_rot: rot,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    pub fn draw_billboard(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.billboard,
            texture,
            pos,
            size,
            rot,
        );
    }

    pub fn draw_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.sprite,
            texture,
            pos,
            size,
            rot,
        );
    }

    pub fn draw_horizontal_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.horizontal_sprite,
            texture,
            pos,
            size,
            rot,
        );
    }

    pub fn draw_vertical_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.vertical_sprite,
            texture,
            pos,
            size,
            rot,
        );
    }

    pub fn draw_obj(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        obj: &Obj,
        matrix: Mat4<f32>,
        color: Rgba<f32>,
    ) {
        for mesh in &obj.meshes {
            let mut matrix = matrix;
            if mesh.name == "PlayerSpawn" {
                continue;
            }
            if mesh.name.starts_with("B_") {
                // TODO: only once
                let mut sum = Vec3::ZERO;
                for v in &*mesh.geometry {
                    sum += v.a_v;
                }
                let center = sum / mesh.geometry.len() as f32;
                matrix = matrix
                    * Mat4::translate(center)
                    * Mat4::rotate_z(self.camera.rot_h)
                    * Mat4::translate(-center);
            }
            let texture =
                mesh.material
                    .texture
                    .as_deref()
                    .unwrap_or(if mesh.name.ends_with("_Dark") {
                        &self.transparent_black_texture
                    } else {
                        &self.white_texture
                    });
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_flashdark_pos: self.player.flashdark_pos,
                        u_flashdark_dir: self.player.flashdark_dir,
                        u_flashdark_angle: f32::PI / 4.0,
                        u_flashdark_strength: self.player.flashdark_strength,
                        u_model_matrix: matrix,
                        u_color: color,
                        u_texture: texture,
                        u_texture_matrix: Mat3::identity(),
                        u_dark_texture: mesh.material.dark_texture.as_deref().unwrap_or(texture),
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
            );
        }
    }

    pub fn obj_shadow(
        &self,
        light: &Light,
        framebuffer: &mut ugli::Framebuffer,
        obj: &Obj,
        matrix: Mat4<f32>,
    ) {
        for mesh in &obj.meshes {
            // let mut matrix = matrix;
            if mesh.name == "PlayerSpawn" {
                continue;
            }
            if mesh.name.starts_with("B_") {
                // // TODO: only once
                // let mut sum = Vec3::ZERO;
                // for v in &*mesh.geometry {
                //     sum += v.a_v;
                // }
                // let center = sum / mesh.geometry.len() as f32;
                // matrix = matrix
                //     * Mat4::translate(center)
                //     * Mat4::rotate_z(self.camera.rot_h)
                //     * Mat4::translate(-center);
                continue; // Ignore billboards for lighting for now
            }
            ugli::draw(
                framebuffer,
                &self.assets.shaders.shadow,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_model_matrix: matrix,
                    },
                    geng::camera3d_uniforms(light, framebuffer.size().map(|x| x as f32)),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
            );
        }
    }
}
