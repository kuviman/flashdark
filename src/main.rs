use geng::{prelude::*, Key};

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KeyPuzzleState {
    Begin,
    Entered,
    LightOut,
    Ready,
    Finish,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct KeyConfiguration {
    top_color: usize,
    bottom_color: usize,
    top_shape: usize,
    bottom_shape: usize,
}

impl KeyConfiguration {
    pub fn random() -> Self {
        Self {
            top_color: global_rng().gen_range(0..4),
            bottom_color: global_rng().gen_range(0..4),
            top_shape: global_rng().gen_range(0..4),
            bottom_shape: global_rng().gen_range(0..4),
        }
    }
}

pub struct Game {
    storage_unlocked: bool,
    key_puzzle_state: KeyPuzzleState,
    monster_spawned: bool,
    framebuffer_size: Vec2<f32>,
    quad_geometry: ugli::VertexBuffer<geng::obj::Vertex>,
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera,
    sens: f32,
    white_texture: ugli::Texture,
    black_texture: ugli::Texture,
    transparent_black_texture: ugli::Texture,
    shadow_calc: Option<ShadowCalculation>,
    player: Player,
    ambient_light: Rgba<f32>,
    navmesh: NavMesh,
    interactables: Vec<InteractableState>,
    items: Vec<Item>,
    monster: Monster,
    lights: Collection<Light>,
    time: f32,
    fuse_spawned: bool,
    fuse_placed: bool,
    lock_controls: bool,
    cutscene_t: f32,
    tv_noise: Option<geng::SoundEffect>,
    swing_sfx: Option<geng::SoundEffect>,
    current_swing_ref_distance: f32,
    transision: Option<geng::Transition>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        // geng.window().lock_cursor();

        if false {
            let mut music = assets.music.effect();
            music.set_volume(0.5);
            music.play();
        }
        let mut navmesh = if false {
            Self::init_navmesh(geng, &assets.level)
        } else {
            assets.navmesh.clone()
        };
        navmesh.remove_unreachable_from(assets.level.spawn_point);
        Self {
            storage_unlocked: false,
            key_puzzle_state: KeyPuzzleState::Begin,
            monster_spawned: false,
            cutscene_t: 0.0,
            lock_controls: false,
            ambient_light: assets.config.ambient_light,
            tv_noise: None,
            swing_sfx: None,
            current_swing_ref_distance: 10000.0,
            fuse_placed: false,
            time: 0.0,
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
            fuse_spawned: false,
            interactables: Self::initialize_interactables(assets),
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            player: Player {
                pos: assets.level.spawn_point,
                height: 1.0,
                vel: Vec3::ZERO,
                rot_h: 0.0,
                rot_v: 0.0,
                flashdark_pos: Vec3::ZERO,
                flashdark_dir: vec3(0.0, 1.0, 0.0),
                flashdark_on: true,
                flashdark_strength: 0.0,
                flashdark_dark: 0.0,
                item: None,
                next_footstep: 0.0,
                god_mode: false,
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
            shadow_calc: Some(ShadowCalculation::new()),
            lights: Self::initialize_lights(assets),
            monster: Monster::new(assets),
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
        if !self.lock_controls {
            self.handle_event_camera(&event);
            self.handle_clicks(&event);
        }

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
            geng::Event::KeyDown { key: geng::Key::G } => {
                self.player.god_mode = !self.player.god_mode;
                self.ambient_light = self.assets.config.ambient_light_inside_house;
                self.player.flashdark_dark = 1.0;
                self.player.flashdark_dark = 1.0;
                self.fuse_placed = true;
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
