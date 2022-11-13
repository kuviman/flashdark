use super::*;

impl Game {
    pub fn update_flashdark(&mut self, delta_time: f32) {
        self.player.flashdark_strength = (self.player.flashdark_strength
            + if self.player.flashdark_on { 1.0 } else { -1.0 } * delta_time / 0.3)
            .clamp(0.0, 1.0);

        self.player.flashdark_pos =
            self.player.pos + vec2(-0.2, 0.0).rotate(self.player.rot_h).extend(0.8);

        // let new_dir =
        //     (self.look().position_or_inf() - self.player.flashdark_pos).normalize_or_zero();
        let new_dir = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0)
            .dir
            .normalize_or_zero();
        if Vec3::dot(new_dir, self.player.flashdark_dir) < 0.0 {
            self.player.flashdark_dir = new_dir;
        } else {
            self.player.flashdark_dir = nlerp3(
                self.player.flashdark_dir,
                new_dir,
                (delta_time / 0.1).min(1.0),
            );
        }
    }

    pub fn toggle_flashdark(&mut self) {
        self.player.flashdark_on = !self.player.flashdark_on;
        if self.player.flashdark_on {
            self.assets.sfx.flashOn.play();
        } else {
            self.assets.sfx.flashOff.play();
        }

        // Key puzzle
        if self.key_puzzle_state == KeyPuzzleState::LightOut {
            self.key_puzzle_state = KeyPuzzleState::Finish;
            self.ambient_light = self.assets.config.ambient_light_inside_house;

            for (name, data) in &self.assets.level.items {
                if !name.contains("StudyKey") {
                    continue;
                }
                for (mesh_index, spawn) in data.spawns.iter().enumerate() {
                    self.items.push(Item {
                        name: name.clone(),
                        mesh_index,
                        parent_interactable: spawn.parent_interactable.clone(),
                        pos: spawn.pos,
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
                open: false,
                extra_hacky_library_moving_closet_progress: 0.0,
                progress: 0.0,
                data: data.clone(),
                config: self.assets.config.interactables["I_HintKey"].clone(),
            });
        }
    }
}
