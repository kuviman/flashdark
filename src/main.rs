use geng::prelude::*;

mod assets;
mod camera;
mod draw;
mod id;
mod logic;
mod util;

pub use assets::*;
pub use camera::*;
pub use draw::*;
pub use id::*;
pub use logic::*;
pub use util::*;

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
            items: Self::initialize_items(assets),
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
            interactables: Self::initialize_interactables(assets),
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
        self.handle_event_camera(&event);
        self.handle_clicks(&event);

        // TODO: remove
        match event {
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
