use super::*;

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub enum TargetType {
    Player,
    Noise,
    Flashdark,
    Rng,
}

pub struct Monster {
    pub stand_still_time: f32,
    pub pos: Vec3<f32>,
    pub dir: Vec3<f32>,
    pub scan_timer_going: bool,
    pub target_type: TargetType,
    pub next_pathfind_pos: Vec3<f32>,
    pub scan_timer: f32,
    pub next_scan_pos: Vec3<f32>,
    pub next_target_pos: Vec3<f32>,
    pub speed: f32,
    pub loop_sound: geng::SoundEffect,
    pub scream_time: f32,
    pub next_flashdark_flicker_time: f32,
    pub pause_time: f32,
    pub detect_timer: f32,
}

impl Drop for Monster {
    fn drop(&mut self) {
        self.stop_sounds();
    }
}

impl Monster {
    pub fn stop_sounds(&mut self) {
        self.loop_sound.stop();
    }
    pub fn new(assets: &Assets) -> Self {
        let pos = assets.level.trigger_cubes["GhostSpawn"].center();
        Self {
            detect_timer: 0.0,
            scan_timer: 0.0,
            scan_timer_going: true,
            next_scan_pos: pos,
            stand_still_time: 0.0,
            next_flashdark_flicker_time: assets.config.flashdark_flicker_interval,
            scream_time: 0.0,
            pos,
            target_type: TargetType::Rng,
            dir: vec3(0.0, -1.0, 0.0),
            next_pathfind_pos: pos,
            next_target_pos: pos,
            speed: 1.0,
            pause_time: 0.0,
            loop_sound: {
                let mut effect = assets.sfx.ghost_loop.effect();
                effect.set_volume(0.0);
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
                self.difficulty.peek_distance,
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
            if !check(&interactable.data.obj, interactable.matrix()) {
                return false;
            }
        }
        true
    }
    pub fn monster_sees_player(&self) -> bool {
        if self.player.god_mode {
            return false;
        }
        let distance = (self.monster.pos - self.player.pos).xy().len();
        if distance > self.difficulty.monster_view_distance {
            return false;
        }
        let fov = if distance < self.difficulty.monster_180_range {
            180.0
        } else {
            self.difficulty.monster_fov
        };
        if Vec2::dot(
            self.monster.dir.xy().normalize_or_zero(),
            (self.player.pos - self.monster.pos)
                .xy()
                .normalize_or_zero(),
        ) < (fov / 2.0 * f32::PI / 180.0).cos()
        {
            return false;
        }
        self.can_see(
            self.monster.pos + vec3(0.0, 0.0, 1.0),
            self.player.pos + vec3(0.0, 0.0, self.player.height),
        )
    }
    pub fn monster_walk_to(&mut self, pos: Vec3<f32>, target_type: TargetType) {
        if target_type != TargetType::Rng {
            self.monster.scan_timer = self.difficulty.monster_scan_time;
            self.monster.next_scan_pos = pos;
        }
        // if target_type != self.monster.target_type {
        if (pos - self.monster.next_target_pos).len() > 0.5
            || target_type != self.monster.target_type
        {
            match target_type {
                TargetType::Player => {
                    if self.monster.speed == 1.0 {
                        self.monster.scream_time = 1.0;
                        let mut effect = self.assets.sfx.ghost_scream.effect();
                        effect.set_position(self.monster.pos.map(|x| x as f64));
                        // effect.set_max_distance(self.assets.config.max_sound_distance * 5.0);
                        effect.play();
                    }
                }
                TargetType::Noise | TargetType::Flashdark => {
                    let mut effect = self
                        .assets
                        .sfx
                        .ghost_alarmed
                        .choose(&mut global_rng())
                        .unwrap()
                        .effect();
                    effect.set_position(self.monster.pos.map(|x| x as f64));
                    // effect.set_max_distance(self.assets.config.max_sound_distance);
                    effect.play();
                }
                TargetType::Rng => {}
            };
            self.monster.target_type = target_type;
        }
        self.monster.next_target_pos = pos; // TODO ??? self.navmesh.waypoints[self.navmesh.closest_waypoint(pos)];
        self.monster.next_pathfind_pos = self.monster.pos;
        if let TargetType::Player = target_type {
            let ((s, s_speed), (t, t_speed)) = self.difficulty.monster_chase_speed;
            let k = (((pos - self.monster.pos).len() - s) / (t - s)).clamp(0.0, 1.0);
            self.monster.speed = s_speed * (1.0 - k) + t_speed * k;
        }
    }
    pub fn check_monster_sfx(&mut self, pos: Vec3<f32>) {
        let player_inside_house = {
            let door_id = self
                .interactables
                .iter()
                .position(|interactable| interactable.data.obj.meshes[0].name == "D_DoorMain")
                .unwrap();
            !self.interactables[door_id].open
        };
        if !player_inside_house {
            return;
        }
        if !self.monster_spawned {
            return;
        }
        if (pos - self.monster.pos).xy().len() < self.difficulty.max_ghost_sound_distance as f32 {
            self.monster_walk_to(pos, TargetType::Noise);
        }
    }
    pub fn update_monster(&mut self, delta_time: f32) {
        if self.game_over {
            return;
        }
        if self.monster.speed == 1.0 {
            if let Some((vol, music)) = &mut self.chase_music {
                *vol -= delta_time as f64;
                if *vol < 0.0 {
                    music.stop();
                    self.chase_music = None;
                } else {
                    music.set_volume(*vol);
                }
            }
        } else {
            if self.chase_music.is_none() && self.monster.scream_time <= 0.0 {
                self.chase_music = Some((0.0, {
                    let mut effect = self
                        .assets
                        .music
                        .chase
                        .choose(&mut global_rng())
                        .unwrap()
                        .effect();
                    effect.set_volume(0.0);
                    effect.play();
                    effect
                }));
            }
            if let Some((vol, music)) = &mut self.chase_music {
                *vol = (*vol + delta_time as f64).min(1.0);
                music.set_volume(*vol);
            }
        }
        let player_inside_house = {
            let door_id = self
                .interactables
                .iter()
                .position(|interactable| interactable.data.obj.meshes[0].name == "D_DoorMain")
                .unwrap();
            !self.interactables[door_id].open
        };
        if !self.monster_spawned {
            return;
        }
        unsafe {
            BEEN_INSIDE_HOUSE = true;
        }
        // Scan timer
        if self.monster.scan_timer_going {
            self.monster.scan_timer -= delta_time;
            // Scan ended
            if self.monster.scan_timer < 0.0 {
                self.monster.speed = 1.0;
                self.monster.scan_timer = self.difficulty.monster_scan_time;
                self.monster.next_scan_pos = loop {
                    // *self.navmesh.waypoints.choose(&mut global_rng()).unwrap();
                    let current_room = self
                        .assets
                        .level
                        .room_data
                        .iter()
                        .position(|room| room.horizontal_aabb().contains(self.monster.pos.xy()))
                        .unwrap_or(0);
                    let room = loop {
                        let room = global_rng().gen_range(0..self.assets.level.room_data.len());
                        if room != current_room {
                            break room;
                        }
                    };
                    let room = &self.assets.level.room_data[room];
                    let room_aabb = room.horizontal_aabb();
                    if let Some(res) = self
                        .navmesh
                        .waypoints
                        .iter()
                        .copied()
                        .filter(|&p| room_aabb.contains(p.xy()))
                        .choose(&mut global_rng())
                    {
                        break res;
                    }
                };
                self.monster.next_target_pos = self.monster.next_scan_pos;
                self.monster.next_pathfind_pos = self.monster.pos;
            }
        }
        self.monster.pause_time -= delta_time * self.monster.speed;
        if (self.monster.pos - self.monster.next_target_pos).xy().len() < 0.1 {
            let mut go_next = true;
            self.monster.scan_timer_going = true;
            if self.monster.target_type == TargetType::Rng {
                self.monster.stand_still_time -= delta_time;
                if self.monster.stand_still_time > 0.0 {
                    go_next = false;
                }
            }
            if go_next {
                self.monster.stand_still_time = {
                    let (a, b) = self.difficulty.ghost_stand_still_time;
                    global_rng().gen_range(a..b)
                };
                self.monster_walk_to(
                    //*self.navmesh.waypoints.choose(&mut global_rng()).unwrap(),
                    self.navmesh.find_close_point(
                        self.monster.next_scan_pos,
                        self.difficulty.monster_scan_radius,
                    ),
                    TargetType::Rng,
                );
            }
        }

        if player_inside_house {
            if self.monster_sees_player() {
                self.monster.detect_timer += if self.player.height > 0.75 {
                    delta_time
                } else {
                    delta_time / self.difficulty.crouch_detect_time_multiplier
                };
                if self.monster.speed != 1.0 {
                    self.monster.detect_timer = self.difficulty.monster_detect_time;
                }
                if self.monster.detect_timer >= self.difficulty.monster_detect_time {
                    self.monster_walk_to(self.player.pos, TargetType::Player);
                }
            } else {
                self.monster.detect_timer -= delta_time;
            }
            self.monster.detect_timer = self
                .monster
                .detect_timer
                .clamp(0.0, self.difficulty.monster_detect_time);
        }

        if (self.monster.pos - self.player.pos).len() < 0.5 && !self.player.god_mode {
            if !self.game_over {
                self.stop_sounds();
                self.game_over_sfx = Some(self.assets.sfx.jumpscare.play());
                self.game_over = true;
            }
            // self.transition = Some(geng::Transition::Switch(Box::new(Game::new(
            //     &self.geng,
            //     &self.assets,
            // ))));
        }
        if (self.monster.pos - self.monster.next_pathfind_pos).len() < 0.1 {
            self.monster.next_pathfind_pos = self
                .navmesh
                .pathfind(self.monster.pos, self.monster.next_target_pos);
        }

        self.monster.scream_time -= delta_time;
        if self.monster.scream_time < 0.0 {
            let target = self.monster.next_target_pos;
            let target = target
                .xy()
                .extend(self.navmesh.waypoints[self.navmesh.closest_waypoint(target)].z);
            if self.can_see(self.monster.pos, target) {
                self.monster.next_pathfind_pos = self
                    .monster
                    .next_target_pos
                    .xy()
                    .extend(self.monster.next_pathfind_pos.z);
            };
            if self.monster.pause_time <= 0.0 {
                self.monster.pos += (self.monster.next_pathfind_pos - self.monster.pos)
                    .clamp_len(..=delta_time * self.monster.speed);
            }
        }

        for (id, interactable) in self.interactables.iter().enumerate() {
            let name = &interactable.data.obj.meshes[0].name;
            if !name.starts_with("D") {
                continue;
            }
            if interactable.open {
                continue;
            }
            let v = vector_from_obj(
                &interactable.data.obj,
                interactable.matrix(),
                self.monster.pos,
            );
            let radius = 0.25;
            if v.len() < radius {
                let mut can_open = true;
                if name == "D_DoorStudy" && !player_inside_house {
                    can_open = false;
                }
                // COPYPASTE YAY
                if interactable.config.disabled
                    || (interactable.data.obj.meshes[0].name == "D_DoorStorage"
                        && !self.storage_unlocked)
                    || (interactable.data.obj.meshes[0]
                        .name
                        .ends_with("S_StudyCloset")
                        && self.key_puzzle_state != KeyPuzzleState::Finish)
                {
                    can_open = false;
                }
                if can_open {
                    self.click_interactable(id, false, self.monster.pos);
                    self.monster.pause_time = 0.6;
                    break;
                } else if self.monster.target_type != TargetType::Player {
                    self.monster.next_target_pos = self.monster.pos;
                    self.monster.scan_timer = 0.0;
                    self.monster.scan_timer_going = true;
                    self.monster.stand_still_time = 0.0;
                    let n = v.normalize_or_zero();
                    self.monster.pos += n * (radius - v.len());
                }
            }
        }

        let look_at_pos = match self.monster.target_type {
            TargetType::Player => self.monster.next_target_pos,
            _ => self.monster.next_pathfind_pos,
        };
        let dv = look_at_pos - self.monster.pos;
        if dv.len() > EPS {
            let target_dir = dv.xy().normalize();
            // self.monster.dir = target_dir.extend(0.0);
            self.monster.dir = nlerp2(
                self.monster.dir.xy(),
                target_dir,
                (delta_time * self.monster.speed / 0.2).min(1.0),
            )
            .extend(0.0);
        }

        self.monster.loop_sound.set_volume(1.0);
        self.monster
            .loop_sound
            .set_position(self.monster.pos.map(|x| x as f64));
    }
    pub fn draw_monster(&mut self, framebuffer: &mut ugli::Framebuffer) {
        if !self.monster_spawned {
            return;
        }
        let textures = if self.monster.speed == 1.0 {
            &self.assets.ghost.normal
        } else {
            &self.assets.ghost.chasing
        };
        let texture = if self.game_over {
            &textures.front
        } else {
            [
                (&textures.left, 90.0),
                (&textures.front, 180.0),
                (&textures.right, 270.0),
                (&textures.back, 0.0),
            ]
            .into_iter()
            .max_by_key(|(_texture, angle)| {
                r32(Vec2::dot(
                    self.monster.dir.xy(),
                    vec2(0.0, 1.0).rotate(self.camera.rot_h + angle * f32::PI / 180.0),
                ))
            })
            .unwrap()
            .0
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
        // let look_at_pos = match self.monster.target_type {
        //     TargetType::Player => self.monster.next_target_pos,
        //     _ => self.monster.next_pathfind_pos,
        // };
        // self.draw_sprite(framebuffer, &self.assets.hand, look_at_pos, 0.3, 0.0);
    }
}
