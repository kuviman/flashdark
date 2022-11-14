use super::*;

mod camera;
mod flashdark;
mod interactables;
mod items;
mod light;
mod monster;
mod movement;
mod navmesh;
mod player;

pub use camera::*;
pub use flashdark::*;
pub use interactables::*;
pub use items::*;
pub use light::*;
pub use monster::*;
pub use movement::*;
pub use navmesh::*;
pub use player::*;

impl Game {
    pub fn update_impl(&mut self, delta_time: f32) {
        let delta_time = delta_time.min(1.0 / 30.0);
        self.time += delta_time;

        self.update_movement(delta_time);
        self.update_camera(delta_time);
        self.update_flashdark(delta_time);
        self.update_interactables(delta_time);
        self.update_monster(delta_time);

        // Activating the swing
        if let Some(target) = self.look().target {
            if let Object::Interactable(id) = target.object {
                let interactable = &self.interactables[id];
                if interactable.data.obj.meshes[0].name == "I_FusePlaceholder" {
                    if !self.fuse_spawned {
                        self.fuse_spawned = true;
                        let name = "Fuse";
                        let data = &self.assets.level.items[name];
                        let spawn_index = global_rng().gen_range(0..data.spawns.len());
                        let spawn = &data.spawns[spawn_index];
                        self.items.push(Item {
                            name: name.to_owned(),
                            mesh_index: spawn_index,
                            parent_interactable: None,
                            pos: spawn.pos,
                        });
                        let mut swing_sfx = self.assets.sfx.swingLoop.effect();
                        swing_sfx.set_position(
                            self.assets.level.trigger_cubes["SwingingSwing"]
                                .center()
                                .xy()
                                .extend(self.camera.pos.z)
                                .map(|x| x as f64),
                        );
                        swing_sfx.play(); // TODO: swing
                        self.swing_sfx = Some(swing_sfx);
                    }
                }
            }
        }
        if let Some(sfx) = &mut self.swing_sfx {
            let pos = self.assets.level.trigger_cubes["SwingingSwing"]
                .center()
                .xy()
                .extend(self.camera.pos.z);
            let ref_distance = (pos - self.camera.pos)
                .len()
                .max(1.0)
                .min(self.current_swing_ref_distance);
            self.current_swing_ref_distance = ref_distance;
            sfx.set_ref_distance(ref_distance as f64);
            sfx.set_max_distance(ref_distance as f64 + self.assets.config.max_sound_distance - 1.0);
        }

        // Activate the monster cutscene
        if self.fuse_placed && self.cutscene_t < 3.0 {
            let tv_pos = self.assets.level.trigger_cubes["GhostSpawn"].center();
            let camera_dir = self
                .camera
                .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0)
                .dir
                .normalize_or_zero();
            let tv_dir = (tv_pos - self.camera.pos).normalize_or_zero();
            let trigger_box = &self.assets.level.trigger_cubes["TVLookTrigger"];
            if true
            // trigger_box.horizontal_aabb().contains(self.player.pos.xy())
            // && Vec3::dot(camera_dir, tv_dir)
            //     > (self.assets.config.tv_detection_angle * f32::PI / 180.0).cos()
            {
                self.cutscene_t += delta_time;

                if self.cutscene_t < 0.2 {
                    self.player.flashdark.on = false;
                } else if self.cutscene_t < 0.6 {
                    self.player.flashdark.on = true;
                } else if self.cutscene_t < 0.8 {
                    self.player.flashdark.on = false;
                } else if self.cutscene_t < 1.2 {
                    self.player.flashdark.on = true;
                } else if self.cutscene_t < 1.4 {
                    self.player.flashdark.on = false;
                } else if self.cutscene_t < 1.8 {
                    self.player.flashdark.on = true;
                }
                if self.cutscene_t > 1.4 {
                    self.player.flashdark.dark = 1.0;
                }
                self.player.height = 1.0; // Uncrouch
                self.lock_controls = true;
                let target_rot_h = tv_dir.xy().arg() - f32::PI / 2.0;
                let target_rot_v = vec2(tv_dir.xy().len(), tv_dir.z).arg();
                let t = (delta_time / 0.3).min(1.0);
                self.player.rot_h += (target_rot_h - self.player.rot_h) * t;
                self.player.rot_v += (target_rot_v - self.player.rot_v) * t;
                self.player.pos +=
                    (trigger_box.center().xy().extend(self.player.pos.z) - self.player.pos) * t;

                // End of the cutscene
                if self.cutscene_t >= 3.0 {
                    self.lock_controls = false;
                    self.click_interactable(
                        self.interactables
                            .iter()
                            .position(|interactable| {
                                interactable.data.obj.meshes[0].name == "D_DoorMain"
                            })
                            .unwrap(),
                        false,
                        self.assets.level.trigger_cubes["HouseEntrance"].center(),
                    );
                    self.monster_spawned = true;
                }
            }
        }

        // Entering the house
        if self.assets.level.trigger_cubes["HouseEntrance"]
            .horizontal_aabb()
            .contains(self.player.pos.xy())
        {
            let door_id = self
                .interactables
                .iter()
                .position(|interactable| interactable.data.obj.meshes[0].name == "D_DoorMain")
                .unwrap();
            if self.interactables[door_id].open {
                self.click_interactable(door_id, false, Vec3::ZERO);
                self.ambient_light = self.assets.config.ambient_light_inside_house;
            }
        }

        // Enter study room
        if self.assets.level.trigger_cubes["TriggerStudyEntrance"]
            .horizontal_aabb()
            .contains(self.player.pos.xy())
            && self.key_puzzle_state == KeyPuzzleState::Begin
        {
            let door_id = self
                .interactables
                .iter()
                .position(|interactable| interactable.data.obj.meshes[0].name == "D_DoorStudy")
                .unwrap();
            if self.interactables[door_id].open {
                self.click_interactable(door_id, false, Vec3::ZERO);
            }
            self.key_puzzle_state = KeyPuzzleState::Entered;
        }
    }

    pub fn handle_clicks(&mut self, event: &geng::Event) {
        if let geng::Event::MouseDown { button, .. } = *event {
            self.geng.window().lock_cursor();

            match button {
                geng::MouseButton::Left => {
                    if let Some(target) = self.look().target {
                        match target.object {
                            Object::StaticLevel => {}
                            Object::Interactable(id) => {
                                self.click_interactable(id, true, self.player.pos);
                            }
                            Object::Item(id) => {
                                self.click_item(id);
                            }
                        }
                    }
                }
                geng::MouseButton::Right => {
                    self.toggle_flashdark();
                }
                _ => {}
            }
        }
    }
}
