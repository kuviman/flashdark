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

        self.draw_particles(framebuffer);

        // UI ---

        let camera2d = geng::Camera2d {
            center: Vec2::ZERO,
            rotation: 0.0,
            fov: 10.0,
        };
        if !self.main_menu {
            self.geng.draw_2d(
                framebuffer,
                &camera2d,
                &draw_2d::TexturedQuad::new(
                    AABB::point(vec2(-5.0, -4.2)).extend_uniform(2.0),
                    &self.assets.flashdark,
                ),
            );
        }
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

        if !self.main_menu && !self.in_settings {
            // PawnMan20 is like totally hot
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
        }

        if !self.main_menu && self.intro_t > 0.0 {
            let alpha = ((self.intro_t - 2.0) / 3.0).clamp(0.0, 1.0);
            // self.camera.fov
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

        if self.game_over {
            let alpha = ((self.game_over_t - 1.5) / 4.0).clamp(0.0, 1.0);
            self.geng.draw_2d(
                framebuffer,
                &camera2d,
                &draw_2d::Quad::new(
                    AABB::point(Vec2::ZERO).extend_uniform(100.0),
                    Rgba::new(0.0, 0.0, 0.0, alpha),
                ),
            );
        }

        // self.geng.draw_2d(
        //     framebuffer,
        //     &camera2d,
        //     &draw_2d::TexturedQuad::new(
        //         AABB::point(Vec2::ZERO).extend_uniform(2.0),
        //         self.shadow_calc
        //             .as_ref()
        //             .unwrap()
        //             .shadow_maps
        //             .get(&LightId(0))
        //             .unwrap(),
        //     ),
        // );

        if self.main_menu || self.in_settings {
            if !self.main_menu {
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::Quad::new(
                        AABB::point(Vec2::ZERO).extend_uniform(100.0),
                        Rgba::new(0.0, 0.0, 0.0, 0.5),
                    ),
                );
            }

            let mouse_pos = camera2d.screen_to_world(
                self.framebuffer_size,
                self.geng.window().mouse_pos().map(|x| x as f32),
            );
            self.ui_mouse_pos = mouse_pos;

            let rect_for = |pos: Vec2<f32>, size: f32, texture: &ugli::Texture| -> AABB<f32> {
                AABB::point(pos).extend_symmetric(
                    texture.size().map(|x| x as f32) / texture.size().y as f32 * size,
                )
            };

            let mut hovered = None;
            let mut to_draw = Vec::new();
            let mut new_hover_ui_action = None;
            let mut draw_icon =
                |pos: Vec2<f32>, size: f32, texture, action: Option<UiAction>| -> bool {
                    let rect = rect_for(pos, size, texture);
                    let mut color = Rgba::WHITE;
                    let mut this_hovered = false;
                    if let Some(action) = action {
                        if rect.contains(mouse_pos)
                            || (self
                                .geng
                                .window()
                                .is_button_pressed(geng::MouseButton::Left)
                                && self.hover_ui_action == Some(action))
                        {
                            color = Rgba::BLACK;
                            hovered = Some(pos);
                            new_hover_ui_action = Some(action);
                            this_hovered = true;
                        }
                    }
                    to_draw.push((rect, texture, color));
                    this_hovered
                };

            // PawnMan: "I have a suggestion"
            let mut draw_controls = false;
            if self.in_settings {
                draw_icon(vec2(0.0, 3.0), 1.0, &self.assets.ui.title, None);
                draw_icon(
                    vec2(0.0, 0.1),
                    0.2,
                    &self.assets.ui.label_mouse_sensitivity,
                    None,
                );
                let slider_width = 0.1 * self.assets.ui.slider_line.size().x as f32
                    / self.assets.ui.slider_line.size().y as f32;
                draw_icon(vec2(0.0, -0.5), 0.1, &self.assets.ui.slider_line, None);
                draw_icon(
                    vec2(
                        -slider_width + slider_width * 2.0 * self.settings.mouse_sens,
                        -0.5,
                    ),
                    0.4,
                    &self.assets.ui.slider_handle1,
                    Some(UiAction::ChangeMouseSens),
                );
                if self.hover_ui_action == Some(UiAction::ChangeMouseSens)
                    && self
                        .geng
                        .window()
                        .is_button_pressed(geng::MouseButton::Left)
                {
                    self.settings.mouse_sens =
                        ((mouse_pos.x - (-slider_width)) / (slider_width * 2.0)).clamp(0.0, 1.0);
                }
                draw_icon(
                    vec2(0.0, -1.5),
                    0.2,
                    &self.assets.ui.label_soundvolume,
                    None,
                );
                draw_icon(vec2(0.0, -2.1), 0.1, &self.assets.ui.slider_line, None);
                draw_icon(
                    vec2(
                        -slider_width + slider_width * 2.0 * self.settings.volume,
                        -2.1,
                    ),
                    0.4,
                    &self.assets.ui.slider_handle2,
                    Some(UiAction::ChangeVolume),
                );
                if self.hover_ui_action == Some(UiAction::ChangeVolume)
                    && self
                        .geng
                        .window()
                        .is_button_pressed(geng::MouseButton::Left)
                {
                    self.settings.volume =
                        ((mouse_pos.x - (-slider_width)) / (slider_width * 2.0)).clamp(0.0, 1.0);
                }
                if !self.main_menu {
                    draw_icon(
                        vec2(-5.0, -4.0),
                        0.5,
                        &self.assets.ui.icon_home,
                        Some(UiAction::Home),
                    );
                }
                draw_icon(
                    vec2(5.0, -4.0),
                    0.5,
                    &self.assets.ui.icon_back,
                    Some(UiAction::Back),
                );
                if draw_icon(
                    vec2(5.0, 1.0),
                    0.5,
                    &self.assets.ui.icon_controls,
                    Some(UiAction::None),
                ) {
                    to_draw.pop();
                    draw_controls = true;
                }
            } else if self.main_menu {
                draw_icon(vec2(0.0, 3.0), 1.0, &self.assets.ui.title, None);
                draw_icon(
                    vec2(0.0, 0.1),
                    0.5,
                    &self.assets.ui.play,
                    Some(UiAction::Play),
                );
                draw_icon(
                    vec2(-5.0, -4.0),
                    0.5,
                    &self.assets.ui.icon_settings,
                    Some(UiAction::Settings),
                );
                #[cfg(not(target_arch = "wasm32"))]
                draw_icon(
                    vec2(5.0, -4.0),
                    0.5,
                    &self.assets.ui.icon_door,
                    Some(UiAction::Exit),
                );
            }

            if !self
                .geng
                .window()
                .is_button_pressed(geng::MouseButton::Left)
                && self.hover_ui_action != new_hover_ui_action
            {
                if new_hover_ui_action.is_some() {
                    self.assets.sfx.flash_on.play();
                } else {
                    self.assets.sfx.flash_off.play();
                }
                self.hover_ui_action = new_hover_ui_action;
            }

            if let Some(pos) = hovered {
                let texture = &self.assets.ui.flashlight;
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedQuad::new(
                        AABB::point(Vec2::ZERO).extend_symmetric(
                            texture.size().map(|x| x as f32) / texture.size().y as f32,
                        ),
                        texture,
                    )
                    .translate(vec2(1.0, 0.0))
                    .scale_uniform(if draw_controls { 4.0 } else { 2.0 })
                    .transform(Mat3::rotate((pos - vec2(0.0, -3.0)).arg() + f32::PI))
                    .translate(pos),
                );
            } else {
                let texture = &self.assets.ui.icon_flashlight;
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedQuad::new(
                        AABB::point(Vec2::ZERO).extend_symmetric(
                            texture.size().map(|x| x as f32) / texture.size().y as f32,
                        ),
                        texture,
                    )
                    .scale_uniform(0.1)
                    .transform(Mat3::rotate(-f32::PI / 3.0))
                    .translate(mouse_pos),
                );
            }
            for (rect, texture, color) in to_draw {
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedQuad::colored(rect, texture, color),
                );
            }
            if draw_controls {
                let texture = &self.assets.ui.label_controls;
                let rect = rect_for(vec2(5.0, 1.0), 1.0, texture);
                self.geng.draw_2d(
                    framebuffer,
                    &camera2d,
                    &draw_2d::TexturedQuad::new(rect, texture),
                );
            }
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
            // visitor.visit(&format!("u_lights[{i}].shadow_map"), light.shadow_map);
            visitor.visit(&format!("u_lights_shadow_maps[{i}]"), light.shadow_map);
            visitor.visit(
                &format!("u_lights[{i}].shadow_size"),
                &light.shadow_map.size(),
            );
            visitor.visit(&format!("u_lights[{i}].intensity"), &light.intensity);
        }
    }
}
