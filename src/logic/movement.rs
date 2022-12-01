use super::*;

impl Game {
    pub fn update_movement(&mut self, delta_time: f32) {
        if self.lock_controls || self.intro_t > 0.0 {
            return;
        }
        self.player.rot_h = normalize_angle(self.player.rot_h);

        const CROUCH_TIME: f32 = 0.2;
        if self.player.crouch {
            self.show_crouch_tutorial = false;
            self.player.height += (0.5 - self.player.height).clamp(-delta_time / CROUCH_TIME, 1.0);
        } else {
            self.player.height += (1.0 - self.player.height).clamp(-1.0, delta_time / CROUCH_TIME);
        }

        let mut walk_speed = 3.0;
        walk_speed *= self.player.height;
        if self.player.god_mode && self.geng.window().is_key_pressed(geng::Key::LShift) {
            // TODO: disable
            walk_speed *= 3.0;
        }
        let mut mov = vec2(0.0, 0.0);
        if self
            .assets
            .config
            .controls
            .move_forward
            .iter()
            .any(|button| button.is_pressed(&self.geng))
        {
            mov.y += 1.0;
        }
        if self
            .assets
            .config
            .controls
            .move_left
            .iter()
            .any(|button| button.is_pressed(&self.geng))
        {
            mov.x -= 1.0;
        }
        if self
            .assets
            .config
            .controls
            .move_backward
            .iter()
            .any(|button| button.is_pressed(&self.geng))
        {
            mov.y -= 1.0;
        }
        if self
            .assets
            .config
            .controls
            .move_right
            .iter()
            .any(|button| button.is_pressed(&self.geng))
        {
            mov.x += 1.0;
        }
        let mov = mov.clamp_len(..=1.0);
        let target_vel = mov.rotate(self.camera.rot_h) * walk_speed;
        let accel = 50.0;
        self.player.vel += (target_vel - self.player.vel.xy())
            .clamp_len(..=accel * delta_time)
            .extend(0.0);
        if self.player.god_mode {
            if self.geng.window().is_key_pressed(geng::Key::Space) {
                self.player.pos.z += delta_time * walk_speed;
            }
            if self.geng.window().is_key_pressed(geng::Key::LCtrl) {
                self.player.pos.z -= delta_time * walk_speed;
            }
            self.player.vel.z = 0.0;
        } else {
            let gravity = 15.0;
            self.player.vel.z -= gravity * delta_time;
        }
        self.player.pos += self.player.vel * delta_time;

        if self.player.height == 1.0 && !self.player.god_mode {
            self.player.next_footstep -= self.player.vel.len() * delta_time;
            if self.player.next_footstep < 0.0 {
                self.player.next_footstep = self.assets.config.footstep_dist;
                self.assets
                    .sfx
                    .footsteps
                    .choose(&mut global_rng())
                    .unwrap()
                    .play()
                    .set_volume(0.5);
                if self.player_inside_house {
                    self.assets
                        .sfx
                        .footstep_creaks
                        .choose(&mut global_rng())
                        .unwrap()
                        .play()
                        .set_volume(0.5);
                } else {
                    self.assets
                        .sfx
                        .footsteps_grass
                        .choose(&mut global_rng())
                        .unwrap()
                        .play()
                        .set_volume(0.5);
                }
                self.check_monster_sfx(self.player.pos);
            }
        }

        // Collisions
        if !self.player.god_mode {
            for _ in 0..1 {
                let mut check = |obj: &Obj, matrix: Mat4<f32>| {
                    for tri in obj
                        .meshes
                        .iter()
                        .filter(|mesh| {
                            if mesh.name.starts_with("B_SmallGrass")
                                || mesh.name.starts_with("B_TallGrass")
                                || mesh.name.starts_with("B_Tree")
                            {
                                return false;
                            }
                            true
                        })
                        .flat_map(|mesh| mesh.geometry.chunks(3))
                    {
                        let v = vector_from_triangle(
                            [tri[0].a_v, tri[1].a_v, tri[2].a_v]
                                .map(|pos| (matrix * pos.extend(1.0)).xyz()),
                            self.player.pos,
                        );
                        let radius = 0.25;
                        if v.len() < radius {
                            let n = v.normalize_or_zero();
                            self.player.vel -= n * Vec3::dot(n, self.player.vel);
                            self.player.pos += n * (radius - v.len());
                        }
                    }
                };
                check(&self.level.obj, Mat4::identity());
                for interactable in &self.interactables {
                    check(&interactable.data.obj, interactable.matrix());
                }
                if self.player.vel.z > 1.0 {
                    self.player.vel.z = 1.0;
                }
            }
        }
    }
}
