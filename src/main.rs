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

struct Player {
    pos: Vec3<f32>,
    vel: Vec3<f32>,
}

struct InteractableState {
    open: bool,
    progress: f32,
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera,
    sens: f32,
    white_texture: ugli::Texture,
    player: Player,
    waypoints: Vec<Vec3<f32>>,
    interactables: Vec<InteractableState>,
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
            interactables: assets
                .level
                .interactables
                .iter()
                .map(|_| InteractableState {
                    open: false,
                    progress: 0.0,
                })
                .collect(),
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            player: Player {
                pos: assets.level.spawn_point,
                vel: Vec3::ZERO,
            },
            camera: Camera {
                pos: assets.level.spawn_point,
                fov: f32::PI / 2.0,
                rot_h: 0.0,
                rot_v: 0.0,
            },
            sens: 0.001,
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
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
            geng::Event::MouseDown { .. } => {
                self.geng.window().lock_cursor();

                let mut ray = self
                    .camera
                    .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
                ray.dir = ray.dir.normalize_or_zero();
                let max_t = intersect_ray_with_obj(&self.assets.level.obj, Mat4::identity(), ray)
                    .unwrap_or(1e9);
                for (door_data, door_state) in
                    izip![&self.assets.level.interactables, &mut self.interactables]
                {
                    if let Some(t) = intersect_ray_with_obj(
                        &door_data.obj,
                        door_data.typ.matrix(door_state.progress),
                        ray,
                    ) {
                        if t < max_t {
                            door_state.open = !door_state.open;
                        }
                    }
                }
            }
            geng::Event::MouseMove { position, delta } => {
                let delta = delta.map(|x| x as f32);
                self.camera.rot_h -= delta.x * self.sens;
                self.camera.rot_v = (self.camera.rot_v + delta.y * self.sens)
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
            <Assets as geng::LoadAsset>::load(&geng, &static_path()),
            {
                let geng = geng.clone();
                move |assets| Game::new(&geng, &Rc::new(assets.unwrap()))
            },
        ),
    );
}
