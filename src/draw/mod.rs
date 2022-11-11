use super::*;

mod helpers;

pub use helpers::*;

impl Game {
    pub fn draw_impl(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(
            framebuffer,
            Some(self.assets.config.sky_color),
            Some(1.0),
            None,
        );

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
                interactable.data.typ.matrix(interactable.progress),
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
                &self.assets.ghost_crawling,
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
            let texture_aabb = if name.contains("StudyKey") {
                AABB::point(Vec2::ZERO).extend_positive(vec2(0.25, 0.25))
            } else {
                data.texture_aabb
            };
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

        let reticle_texture = (|| {
            match look.target {
                None => &self.assets.reticle,
                Some(target) => match target.object {
                    Object::StaticLevel => &self.assets.reticle,
                    Object::Interactable(id) => {
                        let interactable = &self.interactables[id];
                        if let Some(requirement) = &interactable.config.require_item {
                            if self.player.item.as_ref() != Some(requirement) {
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
}
