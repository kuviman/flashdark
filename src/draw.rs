use super::*;

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

        self.camera.pos = self.player.pos + vec3(0.0, 0.0, 1.0);
        self.camera.rot_h = self.player.rot_h;
        self.camera.rot_v = self.player.rot_v;

        let mut ray = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
        ray.dir = ray.dir.normalize_or_zero();

        let mut look_at_t =
            intersect_ray_with_obj(&self.assets.level.obj, Mat4::identity(), ray).unwrap_or(1e9);
        for interactable in &self.interactables {
            if let Some(t) = intersect_ray_with_obj(
                &interactable.data.obj,
                interactable.data.typ.matrix(interactable.progress),
                ray,
            ) {
                if t < look_at_t {
                    look_at_t = t;
                }
            }
        }

        self.player.flashdark_pos =
            self.player.pos + vec2(-0.2, 0.0).rotate(self.player.rot_h).extend(0.8);
        // self.player.flashdark_dir = (Mat4::rotate_z(self.player.rot_h)
        //     * Mat4::rotate_x(self.player.rot_v)
        //     * vec4(0.0, 1.0, 0.0, 1.0))
        // .xyz();
        // fn nlerp(a: Vec3<f32>, b: Vec3<f32>, t: f32) -> Vec3<f32> {
        //     (a * (1.0 - t) + b * t).normalize_or_zero()
        // }
        // self.player.flashdark_dir = nlerp(
        //     self.player.flashdark_dir,
        //     (ray.from + ray.dir * look_at_t - self.player.flashdark_pos).normalize_or_zero(),
        //     delta_time,
        // );

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
        for interactable in &self.interactables {
            let mut highlight = false;
            if let Some(t) = intersect_ray_with_obj(
                &interactable.data.obj,
                interactable.data.typ.matrix(interactable.progress),
                ray,
            ) {
                if t <= look_at_t + EPS {
                    highlight = true;
                }
            }
            self.draw_obj(
                framebuffer,
                &interactable.data.obj,
                interactable.data.typ.matrix(interactable.progress),
                if highlight {
                    Rgba::new(0.8, 0.8, 1.0, 1.0)
                } else {
                    Rgba::WHITE
                },
            );
        }

        for item in &self.items {
            let data = &self.assets.level.items[&item.name].spawns[item.mesh_index];
            let texture = &*data.mesh.material.texture.as_deref().unwrap();
            let dark_texture = data
                .mesh
                .material
                .dark_texture
                .as_deref()
                .unwrap_or(texture);

            let mut matrix = Mat4::translate(item.pos);
            if let Some(parent) = &item.parent_interactable {
                let parent = self
                    .interactables
                    .iter()
                    .find(|inter| inter.data.obj.meshes[0].name == *parent) // TODO: this is slow
                    .unwrap();
                matrix = parent.data.typ.matrix(parent.progress) * matrix;
            }

            let highlight = || -> bool {
                let pos = (matrix * vec4(0.0, 0.0, 0.0, 1.0)).xyz();
                let t = Vec3::dot(pos - ray.from, ray.dir);
                if t < 0.0 || t > look_at_t {
                    return false;
                }
                Vec3::cross(pos - ray.from, ray.dir).len() < 0.1
            }();

            if false {
                matrix = matrix
                    * Mat4::rotate_z(self.camera.rot_h)
                    * Mat4::rotate_x(self.camera.rot_v + f32::PI / 2.0);
            }

            let color = if highlight {
                Rgba::new(0.8, 0.8, 1.0, 1.0)
            } else {
                Rgba::WHITE
            };
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::TriangleFan,
                &data.mesh.geometry,
                (
                    ugli::uniforms! {
                        u_flashdark_pos: self.player.flashdark_pos,
                        u_flashdark_dir: self.player.flashdark_dir,
                        u_flashdark_angle: f32::PI / 4.0,
                        u_flashdark_strength: self.player.flashdark_strength,
                        u_model_matrix: matrix,
                        u_color: color,
                        u_texture: texture,
                        u_texture_matrix: Mat3::identity(), // data.texture_matrix,
                        u_dark_texture: dark_texture,
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

        let camera2d = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        self.geng.draw_2d(
            framebuffer,
            &camera2d,
            &draw_2d::TexturedQuad::new(
                AABB::point(vec2(-5.0, -4.2)).extend_uniform(2.0),
                &self.assets.flashdark,
            ),
        );
        if let Some(name) = &self.player.item {
            let data = &self.assets.level.items[name];
            self.geng.draw_2d(
                framebuffer,
                &camera2d,
                &draw_2d::TexturedPolygon::new(
                    vec![
                        draw_2d::TexturedVertex {
                            a_pos: vec2(-1.0, -1.0),
                            a_vt: data.texture_aabb.bottom_left(),
                            a_color: Rgba::WHITE,
                        },
                        draw_2d::TexturedVertex {
                            a_pos: vec2(1.0, -1.0),
                            a_vt: data.texture_aabb.bottom_right(),
                            a_color: Rgba::WHITE,
                        },
                        draw_2d::TexturedVertex {
                            a_pos: vec2(1.0, 1.0),
                            a_vt: data.texture_aabb.top_right(),
                            a_color: Rgba::WHITE,
                        },
                        draw_2d::TexturedVertex {
                            a_pos: vec2(-1.0, 1.0),
                            a_vt: data.texture_aabb.top_left(),
                            a_color: Rgba::WHITE,
                        },
                    ],
                    data.spawns[0].mesh.material.texture.as_deref().unwrap(),
                )
                .scale(vec2(
                    2.0 * data.texture_aabb.width() / data.texture_aabb.height(),
                    2.0,
                ))
                .translate(vec2(5.0, -4.2)),
            );
        }

        self.geng.draw_2d(
            framebuffer,
            &camera2d,
            &draw_2d::Ellipse::circle(Vec2::ZERO, 0.02, Rgba::WHITE),
        );
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
}
