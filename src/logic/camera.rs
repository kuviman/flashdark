use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Object {
    StaticLevel,
    Interactable(Id),
    Item(Id),
}

pub struct LookAtTarget {
    pub object: Object,
    pub t: f32,
}

pub struct LookAt {
    pub ray: geng::CameraRay,
    pub target: Option<LookAtTarget>,
}

impl LookAt {
    pub fn position(&self) -> Option<Vec3<f32>> {
        self.target
            .as_ref()
            .map(|target| self.ray.from + self.ray.dir * target.t)
    }
    pub fn position_or_inf(&self) -> Vec3<f32> {
        const SHOULD_BE_FAR_ENOUGH: f32 = 100.0;
        let t = self
            .target
            .as_ref()
            .map_or(SHOULD_BE_FAR_ENOUGH, |target| target.t);
        self.ray.from + self.ray.dir * t
    }
}

impl Game {
    pub fn update_camera(&mut self, delta_time: f32) {
        self.camera.pos = self.player.pos + vec3(0.0, 0.0, 1.0);
        self.camera.rot_h = self.player.rot_h;
        self.camera.rot_v = self.player.rot_v;

        // Update audio listener
        self.geng
            .audio()
            .set_listener_position(self.camera.pos.map(|x| x as f64));
        self.geng.audio().set_listener_orientation(
            { Mat4::rotate_z(self.camera.rot_h) * vec4(0.0, 1.0, 0.0, 1.0) }
                .xyz()
                .map(|x| x as f64),
            vec3(0.0, 0.0, 1.0),
        );
    }
    pub fn look(&self) -> LookAt {
        let mut ray = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
        ray.dir = ray.dir.normalize_or_zero();

        let mut target: Option<LookAtTarget> = None;
        let mut update_target = |t: Option<f32>, object: Object| {
            if let Some(t) = t {
                if t < target.as_ref().map_or(f32::INFINITY, |target| target.t) {
                    target = Some(LookAtTarget { object, t });
                }
            }
        };
        update_target(
            intersect_ray_with_obj(&self.assets.level.obj, Mat4::identity(), ray),
            Object::StaticLevel,
        );
        for (id, interactable) in self.interactables.iter().enumerate() {
            update_target(
                intersect_ray_with_obj(
                    &interactable.data.obj,
                    interactable.data.typ.matrix(interactable.progress),
                    ray,
                ),
                Object::Interactable(id),
            );
        }
        for (id, item) in self.items.iter().enumerate() {
            update_target(
                intersect_ray_with_mesh(
                    &self.assets.level.items[&item.name].spawns[item.mesh_index].mesh,
                    self.item_matrix(item),
                    ray,
                ),
                Object::Item(id),
            );
        }
        LookAt { ray, target }
    }

    pub fn handle_event_camera(&mut self, event: &geng::Event) {
        if !self.geng.window().cursor_locked() {
            return;
        }
        if let geng::Event::MouseMove { delta, .. } = *event {
            let delta = delta.map(|x| x as f32);
            self.player.rot_h -= delta.x * self.sens;
            self.player.rot_v = (self.player.rot_v + delta.y * self.sens)
                .clamp(Camera::MIN_ROT_V, Camera::MAX_ROT_V);
        }
    }
}