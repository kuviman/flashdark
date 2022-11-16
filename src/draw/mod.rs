use super::*;

mod helpers;

pub use helpers::*;

const SHADOW_MAP_SIZE: Vec2<usize> = vec2(1024, 1024);
const MAX_LIGHTS: usize = 100;

pub struct ShadowCalculation {
    shadow_maps: HashMap<LightId, ugli::Texture>,
    depth_buffers: HashMap<LightId, ugli::Renderbuffer<ugli::DepthComponent>>,
}

pub struct LightsUniform<'a> {
    u_lights: Vec<LightUniform<'a>>,
}

struct LightUniform<'a> {
    pos: Vec3<f32>,
    matrix: Mat4<f32>,
    shadow_map: &'a ugli::Texture,
    intensity: f32,
}

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(
            framebuffer,
            None, // Some(self.assets.config.sky_color),
            Some(1.0),
            None,
        );
        self.draw_skybox_mesh(
            framebuffer,
            &self.assets.level.skybox,
            Mat4::translate(self.camera.pos),
        );

        self.update_shadows();

        let look = self.look();

        self.draw_obj(
            framebuffer,
            &self.assets.level.obj,
            Mat4::identity(),
            Rgba::WHITE,
        );

        for (id, interactable) in self.interactables.iter().enumerate() {
            if interactable.config.transparent {
                continue;
            }
            // let highlight =
            //     look.target.as_ref().map(|target| target.object) == Some(Object::Interactable(id));
            self.draw_obj(
                framebuffer,
                &interactable.data.obj,
                interactable.matrix(),
                Rgba::WHITE,
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
            // let highlight =
            //     look.target.as_ref().map(|target| target.object) == Some(Object::Item(id));
            // let color = if highlight {
            //     Rgba::new(0.8, 0.8, 1.0, 1.0)
            // } else {
            //     Rgba::WHITE
            // };
            self.draw_mesh(framebuffer, &data.mesh, self.item_matrix(item), Rgba::WHITE);
        }

        self.draw_monster(framebuffer);

        // TV cutscene
        if self.fuse_placed && self.cutscene_t < 3.0 && self.lock_controls {
            let t = self.cutscene_t / 3.0;
            self.draw_billboard(
                framebuffer,
                &self.assets.ghost.crawling,
                self.assets.level.trigger_cubes["GhostSpawn"].center()
                    + vec3(1.0 - t, 0.0, 0.0) * 0.5,
                t,
                0.0,
            );
        }

        self.draw_debug_navmesh(framebuffer);

        // UI ---

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
            if name.contains("StudyKey") {
                let texture = if self.player.flashdark.on {
                    data.spawns[0]
                        .mesh
                        .material
                        .dark_texture
                        .as_deref()
                        .unwrap()
                } else {
                    data.spawns[0].mesh.material.texture.as_deref().unwrap()
                };
                let key_config = &self.assets.level.key_configs[name];

                let transform = Mat3::translate(vec2(5.0, -4.2))
                    * Mat3::scale_uniform(2.0)
                    * Mat3::rotate(-f32::PI * 0.7);

                let texture_aabb = AABB::point(Vec2::ZERO)
                    .extend_positive(vec2(0.25, 0.25 / 2.0))
                    .translate(vec2(
                        0.25 * key_config.top_color as f32,
                        0.75 + 0.25 * key_config.top_shape as f32 + 0.25 / 2.0,
                    ));
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedPolygon::new(
                        vec![
                            draw_2d::TexturedVertex {
                                a_pos: vec2(-1.0, 0.0),
                                a_vt: texture_aabb.bottom_left(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(1.0, 0.0),
                                a_vt: texture_aabb.bottom_right(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(1.0, 1.0),
                                a_vt: texture_aabb.top_right(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(-1.0, 1.0),
                                a_vt: texture_aabb.top_left(),
                                a_color: Rgba::WHITE,
                            },
                        ],
                        texture,
                    )
                    .transform(transform),
                );
                let texture_aabb = AABB::point(Vec2::ZERO)
                    .extend_positive(vec2(0.25, 0.25 / 2.0))
                    .translate(vec2(
                        0.25 * key_config.bottom_color as f32,
                        0.75 + 0.25 * key_config.bottom_shape as f32,
                    ));
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedPolygon::new(
                        vec![
                            draw_2d::TexturedVertex {
                                a_pos: vec2(-1.0, -1.0),
                                a_vt: texture_aabb.bottom_left(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(1.0, -1.0),
                                a_vt: texture_aabb.bottom_right(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(1.0, 0.0),
                                a_vt: texture_aabb.top_right(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(-1.0, 0.0),
                                a_vt: texture_aabb.top_left(),
                                a_color: Rgba::WHITE,
                            },
                        ],
                        texture,
                    )
                    .transform(transform),
                );
            } else {
                let texture_aabb = data.texture_aabb;
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedPolygon::new(
                        vec![
                            draw_2d::TexturedVertex {
                                a_pos: vec2(-1.0, -1.0),
                                a_vt: texture_aabb.bottom_left(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(1.0, -1.0),
                                a_vt: texture_aabb.bottom_right(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(1.0, 1.0),
                                a_vt: texture_aabb.top_right(),
                                a_color: Rgba::WHITE,
                            },
                            draw_2d::TexturedVertex {
                                a_pos: vec2(-1.0, 1.0),
                                a_vt: texture_aabb.top_left(),
                                a_color: Rgba::WHITE,
                            },
                        ],
                        data.spawns[0].mesh.material.texture.as_deref().unwrap(),
                    )
                    .scale(vec2(
                        2.0 * texture_aabb.width() / texture_aabb.height(),
                        2.0,
                    ))
                    .translate(vec2(5.0, -4.2)),
                );
            }
        }

        let reticle_texture = (|| {
            match look.target {
                None => &self.assets.reticle,
                Some(target) => match target.object {
                    Object::StaticLevel => &self.assets.reticle,
                    Object::Interactable(id) => {
                        let interactable = &self.interactables[id];

                        // Copypasta mmmm
                        let mut requirement = interactable.config.require_item.as_deref();
                        if interactable.data.obj.meshes[0]
                            .name
                            .starts_with("I_LoosePlank")
                        {
                            requirement = Some("Crowbar");
                        }

                        if let Some(requirement) = requirement {
                            if self.player.item.as_deref() != Some(requirement) {
                                return &self.assets.require_item;
                                // self.assets.level.items[requirement].spawns[0]
                                //     .mesh
                                //     .material
                                //     .texture
                                //     .unwrap();
                            }
                        }
                        &self.assets.hand
                    }
                    Object::Item(id) => &self.assets.hand,
                },
            }
        })();
        self.geng.draw_2d(
            framebuffer,
            &camera2d,
            &draw_2d::TexturedQuad::new(
                AABB::point(Vec2::ZERO).extend_uniform(0.5),
                reticle_texture,
            ),
        );

        if self.intro_t > 0.0 {
            let alpha = ((self.intro_t - 2.0) / 3.0).clamp(0.0, 1.0);
            self.geng.draw_2d(
                framebuffer,
                &camera2d,
                &draw_2d::Quad::new(
                    AABB::point(Vec2::ZERO).extend_uniform(100.0),
                    Rgba::new(0.0, 0.0, 0.0, alpha),
                ),
            );
            self.geng.draw_2d(
                framebuffer,
                &camera2d,
                &draw_2d::Quad::new(
                    AABB::point(vec2(0.0, -4.0))
                        .extend_symmetric(vec2(self.intro_skip_t, 0.1) * 3.0),
                    Rgba::WHITE,
                ),
            );
        }
    }

    fn update_shadows(&mut self) {
        let mut shadow_calc = self.shadow_calc.take().unwrap();
        for light in &self.lights {
            if light.id.0 != 0 && shadow_calc.shadow_maps.contains_key(&light.id) {
                continue;
            }
            // Get shadow map texture and depth buffer for the light
            let shadow_map = shadow_calc.shadow_maps.entry(light.id).or_insert_with(|| {
                let mut texture =
                    ugli::Texture::new_with(self.geng.ugli(), SHADOW_MAP_SIZE, |_| Rgba::WHITE);
                texture.set_filter(ugli::Filter::Nearest);
                texture
            });
            let depth_buffer = shadow_calc
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
            // Level
            self.obj_shadow(
                &light,
                &mut shadow_framebuffer,
                &self.assets.level.obj,
                Mat4::identity(),
                &self.assets.shaders.shadow,
                &self.white_texture,
                Some(ugli::CullFace::Back),
            );

            // Interactables
            for interactable in &self.interactables {
                self.obj_shadow(
                    &light,
                    &mut shadow_framebuffer,
                    &interactable.data.obj,
                    interactable.matrix(),
                    &self.assets.shaders.shadow,
                    &self.white_texture,
                    None,
                );
            }
        }
        self.shadow_calc = Some(shadow_calc);
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

impl<'a> ugli::Uniforms for LightsUniform<'a> {
    fn walk_uniforms<C>(&self, visitor: &mut C)
    where
        C: ugli::UniformVisitor,
    {
        visitor.visit("u_lights_count", &self.u_lights.len());
        for (i, light) in self.u_lights.iter().enumerate().take(MAX_LIGHTS) {
            visitor.visit(&format!("u_lights[{i}].pos"), &light.pos);
            visitor.visit(&format!("u_lights[{i}].matrix"), &light.matrix);
            visitor.visit(&format!("u_lights[{i}].shadow_map"), light.shadow_map);
            visitor.visit(
                &format!("u_lights[{i}].shadow_size"),
                &light.shadow_map.size(),
            );
            visitor.visit(&format!("u_lights[{i}].intensity"), &light.intensity);
        }
    }
}
