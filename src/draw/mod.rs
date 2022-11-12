use super::*;

mod helpers;

pub use helpers::*;

const SHADOW_MAP_SIZE: Vec2<usize> = vec2(1024, 1024);

pub struct ShadowCalculation {
    light_shadow_map: ugli::Texture,
    light_depth_buffer: ugli::Renderbuffer<ugli::DepthComponent>,
    camera_shadow_map: ugli::Texture,
}

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

        self.update_shadows(framebuffer);

        // self.geng.draw_2d(
        //     framebuffer,
        //     &geng::PixelPerfectCamera,
        //     &draw_2d::TexturedQuad::new(
        //         AABB::ZERO.extend_positive(framebuffer.size().map(|x| x as f32)),
        //         &self.shadow_map.as_ref().unwrap().0,
        //     ),
        // );
        // return;

        let look = self.look();

        let light = Light {
            fov: self.camera.fov, // 0.5,
            pos: self.player.flashdark_pos,
            rot_h: self.camera.rot_h, // self.player.rot_h,
            rot_v: self.camera.rot_v, // self.player.rot_v,
        };
        self.draw_obj(
            framebuffer,
            &self.assets.level.obj,
            &light,
            Mat4::identity(),
            Rgba::WHITE,
        );

        for (id, interactable) in self.interactables.iter().enumerate() {
            let highlight =
                look.target.as_ref().map(|target| target.object) == Some(Object::Interactable(id));
            self.draw_obj(
                framebuffer,
                &interactable.data.obj,
                &light,
                interactable.data.typ.matrix(interactable.progress),
                if highlight {
                    Rgba::new(0.8, 0.8, 1.0, 1.0)
                } else {
                    Rgba::WHITE
                },
            );
        }

        for (id, item) in self.items.iter().enumerate() {
            let data = &self.assets.level.items[&item.name].spawns[item.mesh_index];
            let texture = &*data.mesh.material.texture.as_deref().unwrap();
            let dark_texture = data
                .mesh
                .material
                .dark_texture
                .as_deref()
                .unwrap_or(texture);
            let highlight =
                look.target.as_ref().map(|target| target.object) == Some(Object::Item(id));
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
                        u_model_matrix: self.item_matrix(item),
                        u_color: color,
                        u_texture: texture,
                        u_texture_matrix: Mat3::identity(), // data.texture_matrix,
                        u_dark_texture: dark_texture,
                        u_shadow_map: &self.shadow_calc.light_shadow_map,
                        u_shadow_size: self.shadow_calc.light_shadow_map.size(),
                        u_light_matrix: light.matrix(self.shadow_calc.light_shadow_map.size().map(|x| x as f32)),
                        u_light_source: light.pos,
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

        self.draw_monster(framebuffer);

        self.draw_debug_navmesh(framebuffer);

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

        // Draw shadow map
        let aabb = AABB::point(vec2(50.0, 500.0)).extend_positive(self.framebuffer_size * 0.2);
        self.geng.draw_2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw_2d::Quad::new(aabb, Rgba::WHITE),
        );
        self.geng.draw_2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw_2d::TexturedQuad::new(aabb, &self.shadow_calc.light_shadow_map),
        );
    }

    fn update_shadows(&mut self, framebuffer: &ugli::Framebuffer) {
        // Update resolution
        if self.shadow_calc.camera_shadow_map.size() != framebuffer.size() {
            self.shadow_calc.camera_shadow_map =
                ugli::Texture::new_with(self.geng.ugli(), framebuffer.size(), |_| Rgba::BLACK);
        }

        // Create a temprorary framebuffer
        let mut shadow_framebuffer = ugli::Framebuffer::new(
            self.geng.ugli(),
            ugli::ColorAttachment::Texture(&mut self.shadow_calc.light_shadow_map),
            ugli::DepthAttachment::Renderbuffer(&mut self.shadow_calc.light_depth_buffer),
        );
        ugli::clear(&mut shadow_framebuffer, Some(Rgba::WHITE), Some(1.0), None);

        // Render shadows into the texture
        let light = Light {
            fov: self.camera.fov, // 0.5,
            pos: self.player.flashdark_pos,
            rot_h: self.camera.rot_h, // self.player.rot_h,
            rot_v: self.camera.rot_v, // self.player.rot_v,
        };
        obj_shadow(
            &light,
            &mut shadow_framebuffer,
            &self.assets.level.obj,
            Mat4::identity(),
            &self.assets.shaders.shadow,
        );
    }
}

impl ShadowCalculation {
    pub fn new(geng: &Geng) -> Self {
        Self {
            light_shadow_map: {
                let mut texture =
                    ugli::Texture::new_with(geng.ugli(), SHADOW_MAP_SIZE, |_| Rgba::WHITE);
                texture.set_filter(ugli::Filter::Nearest);
                texture
            },
            light_depth_buffer: ugli::Renderbuffer::new(geng.ugli(), SHADOW_MAP_SIZE),
            camera_shadow_map: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::BLACK),
        }
    }
}
