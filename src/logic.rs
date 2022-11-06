use super::*;

impl Game {
    pub fn update_impl(&mut self, delta_time: f32) {
        let delta_time = delta_time.min(1.0 / 30.0);
        let walk_speed = 3.0;
        self.geng
            .audio()
            .set_listener_position(self.camera.pos.map(|x| x as f64));
        self.geng.audio().set_listener_orientation(
            { Mat4::rotate_z(self.camera.rot_h) * vec4(0.0, 1.0, 0.0, 1.0) }
                .xyz()
                .map(|x| x as f64),
            vec3(0.0, 0.0, 1.0),
        );
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

        for state in &mut self.interactables {
            let inter_time = 0.3;
            if state.open {
                state.progress += delta_time / inter_time;
            } else {
                state.progress -= delta_time / inter_time;
            }
            state.progress = state.progress.clamp(0.0, 1.0);
        }

        self.player.flashdark_strength = (self.player.flashdark_strength
            + if self.player.flashdark_on { 1.0 } else { -1.0 } * delta_time / 0.3)
            .clamp(0.0, 1.0);

        self.camera.pos = self.player.pos + vec3(0.0, 0.0, 1.0);
        self.camera.rot_h = self.player.rot_h;
        self.camera.rot_v = self.player.rot_v;

        let mut ray = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
        ray.dir = ray.dir.normalize_or_zero();

        let mut look_at_t =
            intersect_ray_with_obj(&self.assets.level.obj, Mat4::identity(), ray).unwrap_or(1e9);
        for (data, state) in izip![&self.assets.level.interactables, &self.interactables] {
            let mut highlight = false;
            if let Some(t) = intersect_ray_with_obj(&data.obj, data.typ.matrix(state.progress), ray)
            {
                if t < look_at_t {
                    look_at_t = t;
                }
            }
        }

        fn nlerp(a: Vec3<f32>, b: Vec3<f32>, t: f32) -> Vec3<f32> {
            (a * (1.0 - t) + b * t).normalize_or_zero()
        }
        let new_dir =
            (ray.from + ray.dir * look_at_t - self.player.flashdark_pos).normalize_or_zero();
        if Vec3::dot(new_dir, self.player.flashdark_dir) < 0.0 {
            self.player.flashdark_dir = new_dir;
        } else {
            self.player.flashdark_dir = nlerp(
                self.player.flashdark_dir,
                new_dir,
                (delta_time / 0.1).min(1.0),
            );
        }
    }
}
