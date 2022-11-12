use super::*;

impl Game {
    pub fn update_flashdark(&mut self, delta_time: f32) {
        self.player.flashdark_strength = (self.player.flashdark_strength
            + if self.player.flashdark_on { 1.0 } else { -1.0 } * delta_time / 0.3)
            .clamp(0.0, 1.0);

        self.player.flashdark_pos =
            self.player.pos + vec2(-0.2, 0.0).rotate(self.player.rot_h).extend(0.8);

        fn nlerp(a: Vec3<f32>, b: Vec3<f32>, t: f32) -> Vec3<f32> {
            (a * (1.0 - t) + b * t).normalize_or_zero()
        }
        let new_dir =
            (self.look().position_or_inf() - self.player.flashdark_pos).normalize_or_zero();
        if Vec3::dot(new_dir, self.player.flashdark_dir) < 0.0 {
            self.player.flashdark_dir = new_dir;
        } else {
            self.player.flashdark_dir = nlerp(
                self.player.flashdark_dir,
                new_dir,
                (delta_time / 0.1).min(1.0),
            );
        }

        let light = self.lights.get_mut(&LightId(0)).unwrap();
        light.pos = self.player.flashdark_pos;
        light.rot_h = self.camera.rot_h;
        light.rot_v = self.camera.rot_v;
    }

    pub fn toggle_flashdark(&mut self) {
        self.player.flashdark_on = !self.player.flashdark_on;
    }
}
