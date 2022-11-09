use super::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TargetType {
    Player,
    Noise,
    Flashdark,
    Rng,
}

pub struct Monster {
    pub pos: Vec3<f32>,
    pub dir: Vec3<f32>,
    pub target_type: TargetType,
    pub next_pathfind_pos: Vec3<f32>,
    pub next_target_pos: Vec3<f32>,
    pub speed: f32,
    pub loop_sound: geng::SoundEffect,
    pub scream_time: f32,
}

impl Drop for Monster {
    fn drop(&mut self) {
        self.loop_sound.pause();
    }
}

impl Monster {
    pub fn new(assets: &Assets, navmesh: &NavMesh) -> Self {
        let pos = *navmesh.waypoints.choose(&mut global_rng()).unwrap();
        Self {
            scream_time: 0.0,
            pos,
            target_type: TargetType::Rng,
            dir: vec3(0.0, -1.0, 0.0),
            next_pathfind_pos: pos,
            next_target_pos: pos,
            speed: 1.0,
            loop_sound: {
                let mut effect = assets.sfx.ghostLoop.effect();
                effect.set_max_distance(assets.config.max_sound_distance);
                effect.play();
                effect
            },
        }
    }
}

impl Game {
    pub fn can_see(&self, pos: Vec3<f32>, target: Vec3<f32>) -> bool {
        let check = |obj: &Obj, matrix: Mat4<f32>| -> bool {
            if let Some(ray_t) = intersect_ray_with_obj(
                obj,
                matrix,
                geng::CameraRay {
                    from: pos,
                    dir: (target - pos).normalize_or_zero(),
                },
            ) {
                if ray_t < (target - pos).len() {
                    return false;
                }
            }
            true
        };
        if !check(&self.assets.level.obj, Mat4::identity()) {
            return false;
        }
        for interactable in &self.interactables {
            if !check(
                &interactable.data.obj,
                interactable.data.typ.matrix(interactable.progress),
            ) {
                return false;
            }
        }
        true
    }
    pub fn monster_sees_player(&self) -> bool {
        if Vec2::dot(
            self.monster.dir.xy(),
            (self.player.pos - self.monster.pos).xy(),
        ) < 0.0
        {
            return false;
        }
        self.can_see(
            self.monster.pos + vec3(0.0, 0.0, 1.3),
            self.player.pos + vec3(0.0, 0.0, 1.0),
        )
    }
    pub fn monster_walk_to(&mut self, pos: Vec3<f32>, target_type: TargetType) {
        if target_type != self.monster.target_type {
            match target_type {
                TargetType::Player => {
                    self.monster.scream_time = 1.0;
                    let mut effect = self.assets.sfx.ghostScream.effect();
                    effect.set_position(self.monster.pos.map(|x| x as f64));
                    effect.set_max_distance(self.assets.config.max_sound_distance * 5.0);
                    effect.play();
                }
                TargetType::Noise | TargetType::Flashdark => {
                    let mut effect = self
                        .assets
                        .sfx
                        .ghostAlarmed
                        .choose(&mut global_rng())
                        .unwrap()
                        .effect();
                    effect.set_position(self.monster.pos.map(|x| x as f64));
                    effect.set_max_distance(self.assets.config.max_sound_distance);
                    effect.play();
                }
                TargetType::Rng => {}
            };
            self.monster.target_type = target_type;
        }
        self.monster.next_target_pos = pos; // TODO ??? self.navmesh.waypoints[self.navmesh.closest_waypoint(pos)];
        self.monster.next_pathfind_pos = self.monster.pos;
        if let TargetType::Player = target_type {
            let s = 0.5;
            let s_speed = 5.0;
            let t = 3.0;
            let t_speed = 3.0;
            let k = (((pos - self.monster.pos).len() - s) / (t - s)).clamp(0.0, 1.0);
            self.monster.speed = s_speed * (1.0 - k) + t_speed * k;
        } else {
            self.monster.speed = 1.0;
        }
    }
    pub fn check_monster_sfx(&mut self, pos: Vec3<f32>) {
        if (pos - self.monster.pos).len() < self.assets.config.max_sound_distance as f32 {
            self.monster_walk_to(pos, TargetType::Noise);
        }
    }
    pub fn update_monster(&mut self, delta_time: f32) {
        if (self.monster.pos - self.monster.next_target_pos).xy().len() < 0.1 {
            self.monster_walk_to(
                *self.navmesh.waypoints.choose(&mut global_rng()).unwrap(),
                TargetType::Rng,
            );
        }
        if self.monster_sees_player() {
            self.monster_walk_to(self.player.pos, TargetType::Player);
        } else if self.player.flashdark_on {
            self.monster_walk_to(self.player.pos, TargetType::Flashdark);
        }
        if (self.monster.pos - self.player.pos).len() < 0.5 {
            self.transision = Some(geng::Transition::Switch(Box::new(Game::new(
                &self.geng,
                &self.assets,
            ))));
        }
        if (self.monster.pos - self.monster.next_pathfind_pos).len() < 0.1 {
            self.monster.next_pathfind_pos = self
                .navmesh
                .pathfind(self.monster.pos, self.monster.next_target_pos);
        }

        self.monster.scream_time -= delta_time;
        if self.monster.scream_time < 0.0 {
            let next_pos = if self.can_see(self.monster.pos, self.monster.next_target_pos) {
                self.monster
                    .next_target_pos
                    .xy()
                    .extend(self.monster.next_pathfind_pos.z)
            } else {
                self.monster.next_pathfind_pos
            };
            self.monster.pos +=
                (next_pos - self.monster.pos).clamp_len(..=delta_time * self.monster.speed);
        }

        for (id, interactable) in self.interactables.iter().enumerate() {
            if !interactable.data.obj.meshes[0].name.starts_with("D") {
                continue;
            }
            if interactable.open {
                continue;
            }
            let v = vector_from_obj(
                &interactable.data.obj,
                interactable.data.typ.matrix(interactable.progress),
                self.monster.pos,
            );
            let radius = 0.25;
            if v.len() < radius {
                self.click_interactable(id, false);
                break;
                // let n = v.normalize_or_zero();
                // self.player.vel -= n * Vec3::dot(n, self.player.vel);
                // self.player.pos += n * (radius - v.len());
            }
        }

        let look_at_pos = match self.monster.target_type {
            TargetType::Player => self.monster.next_target_pos,
            _ => self.monster.next_pathfind_pos,
        };
        let dv = look_at_pos - self.monster.pos;
        if dv.len() > EPS {
            let target_dir = dv.xy().normalize();
            self.monster.dir = nlerp2(
                self.monster.dir.xy(),
                target_dir,
                (delta_time / 0.5).min(1.0),
            )
            .extend(0.0);
        }

        self.monster
            .loop_sound
            .set_position(self.monster.pos.map(|x| x as f64));
    }
    pub fn draw_monster(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let texture = if Vec2::dot(
            self.monster.dir.xy(),
            vec2(0.0, 1.0).rotate(self.camera.rot_h),
        ) > 0.0
        {
            &self.assets.ghost
        } else {
            &self.assets.ghost_front
        };
        if self.monster_sees_player() {
            // texture = &self.assets.hand;
        }
        self.draw_billboard(framebuffer, texture, self.monster.pos, 1.5, 0.0);
        // self.draw_sprite(
        //     framebuffer,
        //     texture,
        //     self.monster.pos + self.monster.dir * 0.4,
        //     0.3,
        //     0.0,
        // );
    }
}
