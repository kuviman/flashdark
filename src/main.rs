use geng::prelude::*;

mod assets;
mod camera;
mod draw;
mod logic;
mod obj;
mod util;

use assets::*;
use camera::*;
use draw::*;
use logic::*;
use obj::*;
use util::*;

pub struct Player {
    pub pos: Vec3<f32>,
    pub vel: Vec3<f32>,
    pub rot_h: f32,
    pub rot_v: f32,
    pub flashdark_strength: f32,
    pub flashdark_on: bool,
    pub flashdark_dir: Vec3<f32>,
    pub flashdark_pos: Vec3<f32>,
    pub item: Option<String>,
}

struct InteractableState {
    open: bool,
    progress: f32,
    data: Rc<InteractableData>,
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    quad_geometry: ugli::VertexBuffer<geng::obj::Vertex>,
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera,
    sens: f32,
    white_texture: ugli::Texture,
    black_texture: ugli::Texture,
    transparent_black_texture: ugli::Texture,
    player: Player,
    waypoints: Vec<Vec3<f32>>,
    interactables: Vec<InteractableState>,
    items: Vec<Item>,
}

pub struct Item {
    pub name: String,
    pub mesh_index: usize,
    pub parent_interactable: Option<String>,
    pub pos: Vec3<f32>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        // geng.window().lock_cursor();
        let mut music = assets.music.effect();
        music.set_volume(0.5);
        music.play();
        // let waypoints = {
        //     let obj = &assets.level.obj;
        //     let mut points = Vec::new();
        //     const HOR_GRID_SIZE: usize = 20;
        //     const VER_GRID_SIZE: usize = 5;
        //     let hor_range = -10.0..10.0;
        //     let ver_range = 0.0..1.0;
        //     for x in 0..=HOR_GRID_SIZE {
        //         let x = hor_range.start
        //             + (hor_range.end - hor_range.start) * x as f32 / HOR_GRID_SIZE as f32;
        //         for y in 0..=HOR_GRID_SIZE {
        //             let y = hor_range.start
        //                 + (hor_range.end - hor_range.start) * y as f32 / HOR_GRID_SIZE as f32;
        //             for z in 0..=VER_GRID_SIZE {
        //                 let z = ver_range.start
        //                     + (ver_range.end - ver_range.start) * z as f32 / VER_GRID_SIZE as f32;
        //                 let p = vec3(x, y, z);
        //                 if let Some(t) = intersect_ray_with_obj(
        //                     obj,
        //                     Mat4::identity(),
        //                     geng::CameraRay {
        //                         from: p,
        //                         dir: vec3(0.0, 0.0, -1.0),
        //                     },
        //                 ) {
        //                     if t > 0.1 {
        //                         continue;
        //                     }
        //                 }
        //                 points.push(p);
        //             }
        //         }
        //     }
        //     points
        // };
        let waypoints = vec![];
        Self {
            items: assets
                .level
                .items
                .iter()
                // .map(|(name, spawns)| Item {
                //     name: name.clone(),
                //     pos: spawns.choose(&mut global_rng()).unwrap().clone(),
                // })
                .flat_map(|(name, data)| {
                    data.spawns.iter().enumerate().map(|(index, data)| Item {
                        name: name.clone(),
                        parent_interactable: data.parent_interactable.clone(),
                        mesh_index: index,
                        pos: data.pos,
                    })
                })
                .collect(),
            quad_geometry: ugli::VertexBuffer::new_static(
                geng.ugli(),
                vec![
                    geng::obj::Vertex {
                        a_v: vec3(0.0, 0.0, 0.0),
                        a_vt: vec2(0.0, 0.0),
                        a_vn: vec3(0.0, 1.0, 0.0),
                    },
                    geng::obj::Vertex {
                        a_v: vec3(1.0, 0.0, 0.0),
                        a_vt: vec2(1.0, 0.0),
                        a_vn: vec3(0.0, 1.0, 0.0),
                    },
                    geng::obj::Vertex {
                        a_v: vec3(1.0, 1.0, 0.0),
                        a_vt: vec2(1.0, 1.0),
                        a_vn: vec3(0.0, 1.0, 0.0),
                    },
                    geng::obj::Vertex {
                        a_v: vec3(0.0, 1.0, 0.0),
                        a_vt: vec2(0.0, 1.0),
                        a_vn: vec3(0.0, 1.0, 0.0),
                    },
                ],
            ),
            interactables: assets
                .level
                .interactables
                .iter()
                .filter_map(|data| {
                    let config = assets.config.interactables.get(&data.obj.meshes[0].name);
                    if config.map_or(false, |config| config.hidden) {
                        return None;
                    }
                    Some(InteractableState {
                        open: assets
                            .config
                            .open_interactables
                            .contains(&data.obj.meshes[0].name),
                        progress: 0.0,
                        data: data.clone(),
                    })
                })
                .collect(),
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            player: Player {
                pos: assets.level.spawn_point,
                vel: Vec3::ZERO,
                rot_h: 0.0,
                rot_v: 0.0,
                flashdark_pos: Vec3::ZERO,
                flashdark_dir: vec3(0.0, 1.0, 0.0),
                flashdark_on: false,
                flashdark_strength: 0.0,
                item: None,
            },
            camera: Camera {
                pos: assets.level.spawn_point,
                fov: f32::PI / 2.0,
                rot_h: 0.0,
                rot_v: 0.0,
            },
            sens: 0.001,
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
            black_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::BLACK),
            transparent_black_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| {
                Rgba::TRANSPARENT_BLACK
            }),
            waypoints,
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.update_impl(delta_time as f32);
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_impl(framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { button, .. } => {
                self.geng.window().lock_cursor();

                match button {
                    geng::MouseButton::Left => {
                        let mut ray = self
                            .camera
                            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
                        ray.dir = ray.dir.normalize_or_zero();
                        let mut look_at_t =
                            intersect_ray_with_obj(&self.assets.level.obj, Mat4::identity(), ray)
                                .unwrap_or(1e9);
                        for interactable in &self.interactables {
                            if let Some(t) = intersect_ray_with_obj(
                                &interactable.data.obj,
                                interactable.data.typ.matrix(interactable.progress),
                                ray,
                            ) {
                                if t < look_at_t {
                                    look_at_t = t;
                                }
                            }
                        }

                        for (index, item) in self.items.iter().enumerate() {
                            // TODO: copypasta
                            let data = &self.assets.level.items[&item.name].spawns[item.mesh_index];
                            let mut matrix = Mat4::translate(item.pos);
                            if let Some(parent) = &item.parent_interactable {
                                let parent = self
                                    .interactables
                                    .iter()
                                    .find(|inter| inter.data.obj.meshes[0].name == *parent)
                                    .unwrap();
                                matrix = parent.data.typ.matrix(parent.progress) * matrix;
                            }
                            let see = || -> bool {
                                let pos = (matrix * vec4(0.0, 0.0, 0.0, 1.0)).xyz();
                                let t = Vec3::dot(pos - ray.from, ray.dir);
                                if t < 0.0 || t > look_at_t {
                                    return false;
                                }
                                Vec3::cross(pos - ray.from, ray.dir).len() < 0.1
                            }();
                            if see {
                                let item = self.items.remove(index);
                                if let Some(prev) = self.player.item.replace(item.name) {
                                    self.items.push(Item {
                                        name: prev,
                                        parent_interactable: None,
                                        mesh_index: 0,
                                        pos: self.player.pos,
                                    })
                                }
                                return;
                            }
                        }
                        for (index, interactable) in self.interactables.iter_mut().enumerate() {
                            let config = self
                                .assets
                                .config
                                .interactables
                                .get(&interactable.data.obj.meshes[0].name);
                            if let Some(config) = config {
                                if let Some(requirement) = &config.require_item {
                                    if self.player.item.as_ref() != Some(requirement) {
                                        continue;
                                    }
                                }
                            }
                            if let Some(t) = intersect_ray_with_obj(
                                &interactable.data.obj,
                                interactable.data.typ.matrix(interactable.progress),
                                ray,
                            ) {
                                if t <= look_at_t + EPS {
                                    interactable.open = !interactable.open;
                                    if config.map_or(false, |config| config.use_item) {
                                        self.player.item = None;
                                    }
                                    if let Some(transform) =
                                        config.and_then(|config| config.transform_on_use.as_deref())
                                    {
                                        self.interactables.remove(index);
                                        let new_object = self
                                            .assets
                                            .level
                                            .interactables
                                            .iter()
                                            .find(|data| data.obj.meshes[0].name == transform)
                                            .unwrap();
                                        self.interactables.push(InteractableState {
                                            open: false,
                                            progress: 0.0,
                                            data: new_object.clone(),
                                        });
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    geng::MouseButton::Right => {
                        self.player.flashdark_on = !self.player.flashdark_on;
                    }
                    _ => {}
                }
            }
            geng::Event::MouseMove { position, delta } => {
                let delta = delta.map(|x| x as f32);
                self.player.rot_h -= delta.x * self.sens;
                self.player.rot_v = (self.player.rot_v + delta.y * self.sens)
                    .clamp(Camera::MIN_ROT_V, Camera::MAX_ROT_V);
            }
            geng::Event::KeyDown { key: geng::Key::J } => {
                self.assets.jumpscare.play();
            }
            _ => {}
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();

    let geng = Geng::new("FlashDark");
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            <Assets as geng::LoadAsset>::load(&geng, &static_path().join("assets")),
            {
                let geng = geng.clone();
                move |assets| Game::new(&geng, &Rc::new(assets.unwrap()))
            },
        ),
    );
}
