use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LightId(pub u64);

/// A directional point source of light.
#[derive(Debug, Clone, HasId)]
pub struct Light {
    pub id: LightId,
    pub flicker_time: f32,
    pub fov: f32,
    pub pos: Vec3<f32>,
    pub rot_h: f32,
    pub rot_v: f32,
    pub intensity: f32,
}

impl Light {
    pub fn matrix(&self, framebuffer_size: Vec2<f32>) -> Mat4<f32> {
        use geng::AbstractCamera3d;
        self.projection_matrix(framebuffer_size) * self.view_matrix()
    }
}

impl geng::AbstractCamera3d for Light {
    fn view_matrix(&self) -> Mat4<f32> {
        Mat4::rotate_x(-self.rot_v)
            * Mat4::rotate_y(-self.rot_h)
            * Mat4::rotate_x(-f32::PI / 2.0)
            * Mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: Vec2<f32>) -> Mat4<f32> {
        Mat4::perspective(self.fov, framebuffer_size.x / framebuffer_size.y, 0.1, 50.0)
    }
}

impl Game {
    pub fn initialize_lights(assets: &Rc<Assets>) -> Collection<Light> {
        let mut id = 0;
        let mut id = || {
            let i = LightId(id);
            id += 1;
            i
        };
        let mut lights = Collection::new();
        lights.insert(Light {
            id: id(),
            flicker_time: 0.0,
            fov: 1.3,
            pos: Vec3::ZERO,
            rot_h: 0.0,
            rot_v: 0.0,
            intensity: 1.0,
        });
        lights.extend(assets.level_obj.meshes.iter().filter_map(|mesh| {
            if mesh.name == "B_CeilingLight.008_B_CeilingLight.011" {
                return None;
            }
            mesh.name.contains("CeilingLight").then(|| Light {
                id: id(),
                flicker_time: 0.0,
                fov: 2.0,
                pos: {
                    let mut sum = Vec3::ZERO;
                    for v in mesh.geometry.iter() {
                        sum += v.a_v;
                    }
                    sum / mesh.geometry.len() as f32
                },
                rot_h: 0.0,
                rot_v: -f32::PI / 2.0,
                intensity: 1.0,
            })
        }));
        lights
    }

    pub fn update_lights(&mut self, delta_time: f32) {
        self.light_flicker_time -= delta_time;
        if self.light_flicker_time < 0.0 {
            self.light_flicker_time = 5.0;
            let light = self
                .lights
                .iter_mut()
                .filter(|l| l.id.0 != 0)
                .choose(&mut global_rng())
                .unwrap();
            light.flicker_time = 0.5;
            // let mut sfx = self.assets.sfx.light_flicker.effect();
            // sfx.set_position(light.pos.map(|x| x as f64));
            // sfx.set_max_distance(self.assets.config.max_sound_distance);
            // sfx.play();
        }
        for light in &mut self.lights {
            light.flicker_time -= delta_time;
            // light.intensity += (target_intensity - light.intensity).clamp_abs(delta_time / 0.)
        }
    }
}
