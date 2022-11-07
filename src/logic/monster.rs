use super::*;

pub struct Monster {
    pub pos: Vec3<f32>,
    pub dir: Vec3<f32>,
    pub next_pathfind_pos: Vec3<f32>,
    pub next_target_pos: Vec3<f32>,
    pub speed: f32,
}

impl Monster {
    pub fn new(assets: &Assets, navmesh: &NavMesh) -> Self {
        let pos = *navmesh.waypoints.choose(&mut global_rng()).unwrap();
        Self {
            pos,
            dir: vec3(0.0, -1.0, 0.0),
            next_pathfind_pos: pos,
            next_target_pos: pos,
            speed: 1.0,
        }
    }
}

impl Game {
    pub fn monster_sees_player(&self) -> bool {
        if Vec2::dot(
            self.monster.dir.xy(),
            (self.player.pos - self.monster.pos).xy(),
        ) < 0.0
        {
            return false;
        }
        if let Some(ray_t) = intersect_ray_with_obj(
            &self.assets.level.obj,
            Mat4::identity(),
            geng::CameraRay {
                from: self.monster.pos,
                dir: (self.player.pos - self.monster.pos).normalize_or_zero(),
            },
        ) {
            if ray_t < (self.player.pos - self.monster.pos).len() {
                return false;
            }
        }
        true
    }
    pub fn monster_walk_to(&mut self, pos: Vec3<f32>, player: bool) {
        self.monster.next_target_pos = self.navmesh.waypoints[self.navmesh.closest_waypoint(pos)];
        self.monster.next_pathfind_pos = self.monster.pos;
        if player {
            let s = 0.5;
            let s_speed = 5.0;
            let t = 3.0;
            let t_speed = 1.0;
            let k = (((pos - self.monster.pos).len() - s) / (t - s)).clamp(0.0, 1.0);
            self.monster.speed = s_speed * (1.0 - k) + t_speed * k;
        } else {
            self.monster.speed = 1.0;
        }
    }
    pub fn update_monster(&mut self, delta_time: f32) {
        if (self.monster.pos - self.monster.next_target_pos).len() < 0.1 {
            self.monster_walk_to(
                *self.navmesh.waypoints.choose(&mut global_rng()).unwrap(),
                false,
            );
        }
        if self.player.flashdark_on {
            self.monster_walk_to(self.player.pos, true);
        }
        if self.monster_sees_player() {
            self.monster_walk_to(self.player.pos, true);
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
        let dv = self.monster.next_pathfind_pos - self.monster.pos;
        self.monster.pos += dv.clamp_len(..=delta_time * self.monster.speed);
        if dv.len() > EPS {
            self.monster.dir = dv.xy().normalize().extend(0.0);
        }
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
