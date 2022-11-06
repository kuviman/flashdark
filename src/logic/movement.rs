use super::*;

impl Game {
    pub fn update_movement(&mut self, delta_time: f32) {
        let walk_speed = 3.0;
        let mut mov = vec2(0.0, 0.0);
        if self.geng.window().is_key_pressed(geng::Key::W)
            || self.geng.window().is_key_pressed(geng::Key::Up)
        {
            mov.y += 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::A)
            || self.geng.window().is_key_pressed(geng::Key::Left)
        {
            mov.x -= 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::S)
            || self.geng.window().is_key_pressed(geng::Key::Down)
        {
            mov.y -= 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::D)
            || self.geng.window().is_key_pressed(geng::Key::Right)
        {
            mov.x += 1.0;
        }
        let mov = mov.clamp_len(..=1.0);
        let target_vel = mov.rotate(self.camera.rot_h) * walk_speed;
        let accel = 50.0;
        self.player.vel += (target_vel - self.player.vel.xy())
            .clamp_len(..=accel * delta_time)
            .extend(0.0);
        // if self.geng.window().is_key_pressed(geng::Key::Space) {
        //     self.player.pos.z += delta_time * walk_speed;
        // }
        // if self.geng.window().is_key_pressed(geng::Key::LCtrl) {
        let gravity = 5.0;
        self.player.vel.z -= gravity * delta_time;
        // }
        self.player.pos += self.player.vel * delta_time;

        // Collisions
        for _ in 0..1 {
            let mut check = |obj: &Obj, matrix: Mat4<f32>| {
                let v = vector_from_obj(obj, matrix, self.player.pos);
                let radius = 0.25;
                if v.len() < radius {
                    let n = v.normalize_or_zero();
                    self.player.vel -= n * Vec3::dot(n, self.player.vel);
                    self.player.pos += n * (radius - v.len());
                }
            };
            check(&self.assets.level.obj, Mat4::identity());
            for interactable in &self.interactables {
                check(
                    &interactable.data.obj,
                    interactable.data.typ.matrix(interactable.progress),
                );
            }
        }
    }
}
