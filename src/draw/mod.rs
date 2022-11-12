use super::*;

mod helpers;

pub use helpers::*;

const SHADOW_MAP_SIZE: Vec2<usize> = vec2(1024, 1024);

pub struct ShadowCalculation {
    shadow_maps: HashMap<LightId, ugli::Texture>,
    depth_buffers: HashMap<LightId, ugli::Renderbuffer<ugli::DepthComponent>>,
}

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

        self.update_shadows();

        let look = self.look();

        self.draw_obj(
            framebuffer,
            &self.assets.level.obj,
            Mat4::identity(),
            Rgba::WHITE,
        );

        for (id, interactable) in self.interactables.iter().enumerate() {
            let highlight =
                look.target.as_ref().map(|target| target.object) == Some(Object::Interactable(id));
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
            for light in &self.lights {
                let shadow_map = self.shadow_calc.shadow_maps.get(&light.id).unwrap();
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
                            u_shadow_map: shadow_map,
                            u_shadow_size: shadow_map.size(),
                            u_light_matrix: light.matrix(shadow_map.size().map(|x| x as f32)),
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
        let shadow_map = self.shadow_calc.shadow_maps.get(&LightId(0)).unwrap();
        let aabb = AABB::point(vec2(50.0, 500.0))
            .extend_positive(shadow_map.size().map(|x| x as f32) * 0.2);
        self.geng.draw_2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw_2d::Quad::new(aabb, Rgba::WHITE),
        );
        self.geng.draw_2d(
            framebuffer,
            &geng::PixelPerfectCamera,
            &draw_2d::TexturedQuad::new(aabb, shadow_map),
        );
    }

    fn update_shadows(&mut self) {
        for light in &self.lights {
            // Get shadow map texture and depth buffer for the light
            let shadow_map = self
                .shadow_calc
                .shadow_maps
                .entry(light.id)
                .or_insert_with(|| {
                    let mut texture =
                        ugli::Texture::new_with(self.geng.ugli(), SHADOW_MAP_SIZE, |_| Rgba::WHITE);
                    texture.set_filter(ugli::Filter::Nearest);
                    texture
                });
            let depth_buffer = self
                .shadow_calc
                .depth_buffers
                .entry(light.id)
                .or_insert_with(|| ugli::Renderbuffer::new(self.geng.ugli(), SHADOW_MAP_SIZE));
            // Create a temprorary framebuffer for light
            let mut shadow_framebuffer = ugli::Framebuffer::new(
                self.geng.ugli(),
                ugli::ColorAttachment::Texture(shadow_map),
                ugli::DepthAttachment::Renderbuffer(depth_buffer),
            );
            ugli::clear(&mut shadow_framebuffer, Some(Rgba::WHITE), Some(1.0), None);

            // Get the shadow map from the light's perspective
            obj_shadow(
                &light,
                &mut shadow_framebuffer,
                &self.assets.level.obj,
                Mat4::identity(),
                &self.assets.shaders.shadow,
            );
        }
    }
}

impl ShadowCalculation {
    pub fn new() -> Self {
        Self {
            shadow_maps: default(),
            depth_buffers: default(),
        }
    }
}

impl Default for ShadowCalculation {
    fn default() -> Self {
        Self::new()
    }
}
