use super::*;

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

        self.camera.pos = self.player.pos + vec3(0.0, 0.0, 1.0);
        self.camera.rot_h = self.player.rot_h;
        self.camera.rot_v = self.player.rot_v;
        self.draw_obj(
            framebuffer,
            &self.assets.level.obj,
            Mat4::identity(),
            Rgba::WHITE,
        );

        let mut ray = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
        ray.dir = ray.dir.normalize_or_zero();

        let mut ray = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
        ray.dir = ray.dir.normalize_or_zero();
        let max_t =
            intersect_ray_with_obj(&self.assets.level.obj, Mat4::identity(), ray).unwrap_or(1e9);
        for (data, state) in izip![&self.assets.level.interactables, &self.interactables] {
            let mut highlight = false;
            if let Some(t) = intersect_ray_with_obj(&data.obj, data.typ.matrix(state.progress), ray)
            {
                if t < max_t {
                    highlight = true;
                }
            }
            self.draw_obj(
                framebuffer,
                &data.obj,
                data.typ.matrix(state.progress),
                if highlight {
                    Rgba::new(0.8, 0.8, 1.0, 1.0)
                } else {
                    Rgba::WHITE
                },
            );
        }
    }

    fn draw_texture(
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

    fn draw_billboard(
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

    fn draw_sprite(
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

    fn draw_horizontal_sprite(
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

    fn draw_vertical_sprite(
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

    fn draw_obj(
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
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_flashdark_pos: self.player.pos + vec2(-0.2, 0.0).rotate(self.player.rot_h).extend(0.8),
                        u_flashdark_dir: (Mat4::rotate_z(self.player.rot_h) * Mat4::rotate_x(self.player.rot_v) * vec4(0.0, 1.0, 0.0, 1.0)).xyz(),
                        u_flashdark_angle: f32::PI / 6.0,
                        u_flashdark_strength: self.player.flashdark_strength,
                        u_model_matrix: matrix,
                        u_color: color,
                        u_texture: mesh.material.texture.as_deref().unwrap_or(&self.white_texture),
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
}
