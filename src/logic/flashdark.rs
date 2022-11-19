use super::*;

impl Game {
    pub fn update_flashdark(&mut self, delta_time: f32) {
        self.player.flashdark.strength = (self.player.flashdark.strength
            + if self.player.flashdark.on { 1.0 } else { -1.0 } * delta_time / 0.3)
            .clamp(0.0, 1.0);

        self.player.flashdark.pos = self.player.pos
            + vec2(-0.2, 0.0)
                .rotate(self.player.rot_h)
                .extend(self.player.height - 0.2);

        // let new_dir =
        //     (self.look().position_or_inf() - self.player.flashdark.pos).normalize_or_zero();
        self.player.flashdark.rot_v +=
            (self.player.rot_v - self.player.flashdark.rot_v) * (delta_time / 0.1).min(1.0);
        self.player.flashdark.rot_h +=
            normalize_angle(self.player.rot_h - self.player.flashdark.rot_h)
                * (delta_time / 0.1).min(1.0);
        self.player.flashdark.dir = (Mat4::rotate_z(self.player.flashdark.rot_h)
            * Mat4::rotate_x(self.player.flashdark.rot_v)
            * vec4(0.0, 1.0, 0.0, 1.0))
        .xyz();

        let light = self.lights.get_mut(&LightId(0)).unwrap();
        light.pos = self.player.flashdark.pos;
        light.rot_h = self.player.flashdark.rot_h;
        light.rot_v = self.player.flashdark.rot_v;
        light.intensity = self.player.flashdark.strength;

        // actually flicker LUL
        if self.player.flashdark.on && self.intro_t < 0.0 {
            self.monster.next_flashdark_flicker_time -= delta_time;
            if self.monster.next_flashdark_flicker_time < 0.5 {
                self.lights.get_mut(&LightId(0)).unwrap().flicker_time =
                    self.monster.next_flashdark_flicker_time;
            }
            if self.monster.next_flashdark_flicker_time < 0.0 {
                self.monster.next_flashdark_flicker_time =
                    self.assets.config.flashdark_flicker_interval;
                if global_rng().gen_bool(self.assets.config.flashdark_turn_off_probability as f64) {
                    self.toggle_flashdark();
                }
            }
        }
    }

    pub fn toggle_flashdark(&mut self) {
        self.player.flashdark.on = !self.player.flashdark.on;
        if self.player.flashdark.on {
            self.assets.sfx.flash_on.play();
        } else {
            self.assets.sfx.flash_off.play();
        }

        self.monster.next_flashdark_flicker_time = self.assets.config.flashdark_flicker_interval;
        self.check_monster_sfx(self.player.pos);

        // Key puzzle
        if self.key_puzzle_state == KeyPuzzleState::LightOut {
            self.key_puzzle_state = KeyPuzzleState::Ready;
            self.ambient_light = self.assets.config.ambient_light_inside_house;

            self.assets.sfx.light_flicker.play();
            for light in &mut self.lights {
                light.flicker_time = 0.5;
            }

            for (name, data) in &self.assets.level.items {
                if !name.contains("StudyKey") {
                    continue;
                }
                for (mesh_index, spawn) in data.spawns.iter().enumerate() {
                    self.items.push(Item {
                        name: name.clone(),
                        matrix: Mat4::translate(spawn.pos),
                        mesh_index,
                        parent_interactable: spawn.parent_interactable.clone(),
                    });
                }
            }
            let data = self
                .assets
                .level
                .interactables
                .iter()
                .find(|i| i.obj.meshes[0].name == "I_HintKey")
                .unwrap();
            self.interactables.push(InteractableState {
                open_other_way: false,
                open: false,
                extra_hacky_library_moving_closet_progress: 0.0,
                progress: 0.0,
                data: data.clone(),
                config: self.assets.config.interactables["I_HintKey"].clone(),
            });
        }
    }
}
