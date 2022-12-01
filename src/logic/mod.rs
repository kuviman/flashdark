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

        self.update_lights(delta_time);

        // Update audio listener
        self.geng
            .audio()
            .set_listener_position(self.camera.pos.map(|x| x as f64));
        self.geng.audio().set_listener_orientation(
            { Mat4::rotate_z(self.camera.rot_h) * vec4(0.0, 1.0, 0.0, 1.0) }
                .xyz()
                .map(|x| x as f64),
            vec3(0.0, 0.0, 1.0),
        );

        if self.main_menu || self.in_settings {
            return;
        }
        self.update_movement(delta_time);
        self.update_camera(delta_time);
        self.update_flashdark(delta_time);
        self.update_interactables(delta_time);
        self.update_monster(delta_time);

        // Intro
        if self.intro_t > 0.0 {
            if !self.geng.window().pressed_keys().is_empty() {
                self.intro_skip_t += delta_time;
            } else {
                self.intro_skip_t -= delta_time;
            }
            self.intro_skip_t = self.intro_skip_t.clamp(0.0, 1.0);
            if self.intro_skip_t >= 1.0 {
                self.intro_t = self.intro_t.min(0.1);
            }
            let gate_open_time = 5.0;
            let a = self.intro_t > gate_open_time;
            self.intro_t -= delta_time;
            if a && self.intro_t < gate_open_time {
                for i in &mut self.interactables {
                    if i.data.obj.meshes[0].name.contains("FenceDoor") {
                        i.open = true;
                    }
                }
            }
            if self.intro_t < 0.0 {
                unsafe {
                    INTRO_SEEN = true;
                }
                if let Some(mut sfx) = self.intro_sfx.take() {
                    sfx.stop();
                }
                for i in &mut self.interactables {
                    if i.data.obj.meshes[0].name.contains("FenceDoor") {
                        i.open = false;
                        i.progress = 0.0;
                    }
                }
                self.music = Some(self.assets.music.outside.play());
            }
            self.player.pos = self.level.spawn_point - vec3(0.0, self.intro_t, 0.0);
        }

        // Activating the swing
        if let Some(target) = self.look().target {
            if let Object::Interactable(id) = target.object {
                let interactable = &self.interactables[id];
                if interactable.data.obj.meshes[0].name == "I_FusePlaceholder" {
                    if !self.fuse_spawned {
                        self.fuse_spawned = true;
                        let name = "Fuse";
                        let data = &self.level.items[name];
                        let spawn_index = global_rng().gen_range(0..data.spawns.len());
                        let spawn = &data.spawns[spawn_index];
                        self.items.push(Item {
                            name: name.to_owned(),
                            matrix: Mat4::translate(spawn.pos),
                            mesh_index: spawn_index,
                            parent_interactable: None,
                        });
                        let mut swing_sfx = self.assets.sfx.swing_loop.effect();
                        swing_sfx.set_position(
                            self.level.trigger_cubes["SwingingSwing"]
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
            let pos = self.level.trigger_cubes["SwingingSwing"]
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
        if self.fuse_placed && self.cutscene_t < 5.0 {
            let tv_pos = self.level.trigger_cubes["GhostSpawn"].center();
            let camera_dir = self
                .camera
                .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0)
                .dir
                .normalize_or_zero();
            let tv_dir = (tv_pos - self.camera.pos).normalize_or_zero();
            let trigger_box = &self.level.trigger_cubes["TVLookTrigger"];
            if true
            // trigger_box.horizontal_aabb().contains(self.player.pos.xy())
            // && Vec3::dot(camera_dir, tv_dir)
            //     > (self.assets.config.tv_detection_angle * f32::PI / 180.0).cos()
            {
                self.cutscene_t += delta_time;
                self.monster.next_flashdark_flicker_time = 10.0;
                self.player.flashdark.on = true;
                self.player.flashdark.dark = ((self.cutscene_t - 0.5) / 4.0).clamp(0.0, 1.0);
                // if self.cutscene_t > 1.4 {
                //     self.player.flashdark.dark = 1.0;
                // }
                self.player.height = 1.0; // Uncrouch
                self.lock_controls = true;

                // This if is because ofskippi
                if self.cutscene_t < 4.0 {
                    let target_rot_h = tv_dir.xy().arg() - f32::PI / 2.0;
                    let target_rot_v = vec2(tv_dir.xy().len(), tv_dir.z).arg();
                    let t = (delta_time / 0.3).min(1.0);
                    self.player.rot_h += (target_rot_h - self.player.rot_h) * t;
                    self.player.rot_v += (target_rot_v - self.player.rot_v) * t;
                    self.player.pos += ((trigger_box.center().xy() + vec2(0.20, 0.0))
                        .extend(self.player.pos.z)
                        - self.player.pos)
                        * t;
                }

                // End of the cutscene
                if self.cutscene_t >= 5.0 {
                    self.lock_controls = false;
                    self.click_interactable(
                        self.interactables
                            .iter()
                            .position(|interactable| {
                                interactable.data.obj.meshes[0].name == "D_DoorMain"
                            })
                            .unwrap(),
                        false,
                        self.level.trigger_cubes["HouseEntrance"].center(),
                    );
                    self.monster_spawned = true;
                }
            }
        }

        // Bat
        if !self.bat_go
            && self.level.trigger_cubes["BatTrigger"]
                .horizontal_aabb()
                .contains(self.player.pos.xy())
            && normalize_angle(self.player.rot_h - f32::PI / 2.0).abs() < 1.0
        {
            self.bat_go = true;
            self.assets.sfx.bat.play();
        }
        if self.bat_go {
            self.bat_t += delta_time / 0.8;
        }

        // Entering the house
        if self.level.trigger_cubes["HouseEntrance"]
            .horizontal_aabb()
            .contains(self.player.pos.xy())
        {
            self.player_inside_house = true;
            let door_id = self
                .interactables
                .iter()
                .position(|interactable| interactable.data.obj.meshes[0].name == "D_DoorMain")
                .unwrap();
            if self.interactables[door_id].open {
                self.show_crouch_tutorial = true;
                self.click_interactable(door_id, false, Vec3::ZERO);
                self.monster_walk_to(
                    self.navmesh.waypoints[self
                        .navmesh
                        .closest_waypoint(self.level.room_data["Kitchen"].center())],
                    TargetType::Noise,
                );
                if let Some(mut sfx) = self.swing_sfx.take() {
                    sfx.stop();
                }
                self.ambient_light = self.assets.config.ambient_light_inside_house;
            }
        } else {
            self.show_crouch_tutorial = false;
        }

        // Enter study room
        if self.level.trigger_cubes["TriggerStudyEntrance"]
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
            if unsafe { SEEN_LIGHT_OUT } {
                self.light_out();
            } else {
                self.key_puzzle_state = KeyPuzzleState::Entered;
            }
        }

        // GF clock
        self.gf_clock_timer -= delta_time;
        if self.gf_clock_timer < 0.0
            && self.level.trigger_cubes["GrandClock"]
                .horizontal_aabb()
                .contains(self.player.pos.xy())
        {
            self.gf_clock_timer = 60.0;
            let mut sfx = self.assets.sfx.grand_clock.effect();
            sfx.set_position(
                find_center(
                    &self
                        .level
                        .obj
                        .meshes
                        .iter()
                        .find(|m| m.name == "S_GrandfatherClock")
                        .unwrap()
                        .geometry,
                )
                .map(|x| x as f64),
            );
            sfx.set_max_distance(self.assets.config.max_sound_distance);
            sfx.play();
        }

        // Creepy singing
        self.creepy_singing_timer -= delta_time;
        if self.creepy_singing_timer < 0.0
            && self.level.room_data["PlayRoom"]
                .horizontal_aabb()
                .contains(self.player.pos.xy())
        {
            self.creepy_singing_timer = 120.0;
            if self
                .interactables
                .iter()
                .any(|i| i.data.obj.meshes[0].name == "B_SingingGirl")
            {
                let mut sfx = self.assets.music.creepy_singing.effect();
                sfx.set_position(
                    find_center(
                        &self
                            .level
                            .interactables
                            .iter()
                            .map(|i| &i.obj.meshes[0])
                            .find(|m| m.name == "B_SingingGirl")
                            .unwrap()
                            .geometry,
                    )
                    .map(|x| x as f64),
                );
                sfx.set_max_distance(self.assets.config.max_sound_distance);
                sfx.play();
                self.creepy_sing_sfx = Some(sfx);
            }
            {
                let mut sfx = self.assets.music.music_box.effect();
                sfx.set_position(
                    find_center(
                        &self
                            .level
                            .obj
                            .meshes
                            .iter()
                            .find(|m| m.name == "S_MusicBox")
                            .unwrap()
                            .geometry,
                    )
                    .map(|x| x as f64),
                );
                sfx.set_max_distance(self.assets.config.max_sound_distance);
                sfx.play();
                self.music_box_sfx = Some(sfx);
            }
        }
    }

    pub fn handle_clicks(&mut self, event: &geng::Event) {
        if let geng::Event::MouseDown { button, .. } = *event {
            if !self.main_menu && !self.in_settings {
                self.geng.window().lock_cursor();
            }
        }

        if self
            .assets
            .config
            .controls
            .drop_item
            .iter()
            .any(|button| button.matches(event))
        {
            self.drop_item();
        }

        if self
            .assets
            .config
            .controls
            .interact
            .iter()
            .any(|button| button.matches(event))
        {
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
        if self
            .assets
            .config
            .controls
            .toggle_flashdark
            .iter()
            .any(|button| button.matches(event))
        {
            self.toggle_flashdark(false);
        }
    }
}
