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
    navmesh: NavMesh,
    interactables: Vec<InteractableState>,
    items: Vec<Item>,
    monster: Monster,
    transision: Option<geng::Transition>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        // geng.window().lock_cursor();
        let mut music = assets.music.effect();
        music.set_volume(0.5);
        music.play();
        let navmesh = if false {
            Self::init_navmesh(geng, &assets.level)
        } else {
            assets.navmesh.clone()
        };
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
            monster: Monster::new(assets, &navmesh),
            navmesh,
            transision: None,
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
                let mut effect = self.assets.jumpscare.effect();
                effect.set_position(Vec3::ZERO);
                effect.set_max_distance(5.0);
                effect.play();
            }
            geng::Event::KeyDown { key: geng::Key::P } => {
                self.monster.next_target_pos = self.player.pos;
            }
            _ => {}
        }
    }
    fn transition(&mut self) -> Option<geng::Transition> {
        self.transision.take()
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
