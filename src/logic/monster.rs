use super::*;

pub struct Monster {
    pub pos: Vec3<f32>,
    pub dir: Vec3<f32>,
    pub next_pathfind_pos: Vec3<f32>,
    pub next_target_pos: Vec3<f32>,
}

impl Monster {
    pub fn new(assets: &Assets, navmesh: &NavMesh) -> Self {
        let pos = *navmesh.waypoints.choose(&mut global_rng()).unwrap();
        Self {
            pos,
            dir: vec3(0.0, -1.0, 0.0),
            next_pathfind_pos: pos,
            next_target_pos: pos,
        }
    }
}

impl Game {
    pub fn update_monster(&mut self, delta_time: f32) {
        if (self.monster.pos - self.monster.next_target_pos).len() < 0.1 {
            self.monster.next_target_pos =
                *self.navmesh.waypoints.choose(&mut global_rng()).unwrap();
            self.monster.next_pathfind_pos = self.monster.pos;
        }
        if self.player.flashdark_on {
            self.monster.next_target_pos =
                self.navmesh.waypoints[self.navmesh.closest_waypoint(self.player.pos)];
            self.monster.next_pathfind_pos = self.monster.pos;
        }
        if (self.monster.pos - self.monster.next_pathfind_pos).len() < 0.1 {
            self.monster.next_pathfind_pos = self
                .navmesh
                .pathfind(self.monster.pos, self.monster.next_target_pos);
        }
        let dv = self.monster.next_pathfind_pos - self.monster.pos;
        self.monster.pos += dv.clamp_len(..=delta_time);
        if dv.len() > EPS {
            self.monster.dir = dv.xy().normalize().extend(0.0);
        }
    }
    pub fn draw_monster(&mut self, framebuffer: &mut ugli::Framebuffer) {
        let texture = if Vec2::dot(
            self.monster.dir.xy(),
            vec2(0.0, -1.0).rotate(self.camera.rot_h),
        ) > 0.0
        {
            &self.assets.ghost_front
        } else {
            &self.assets.ghost
        };
        self.draw_billboard(framebuffer, texture, self.monster.pos, 1.5, 0.0);
    }
}
