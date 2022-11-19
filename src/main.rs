use geng::{prelude::*, Key};

mod assets;
mod camera;
mod draw;
mod id;
mod loading_screen;
mod logic;
mod util;

pub use assets::*;
pub use camera::*;
pub use draw::*;
pub use id::*;
pub use loading_screen::*;
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

static mut BOOLEAN: bool = false;

pub struct Game {
    gf_clock_timer: f32,
    light_flicker_time: f32,
    rng: RngState,
    game_over: bool,
    game_over_t: f32,
    chase_music: Option<(f64, geng::SoundEffect)>,
    piano_music: geng::SoundEffect,
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
    transition: Option<geng::Transition>,
    music: Option<geng::SoundEffect>,
    noise: ugli::Texture,
    intro_t: f32,
    intro_skip_t: f32,
    intro_sfx: Option<geng::SoundEffect>,
}

impl Drop for Game {
    fn drop(&mut self) {
        self.stop_sounds();
    }
}

impl Game {
    fn stop_sounds(&mut self) {
        if let Some(sfx) = &mut self.music {
            sfx.stop();
        }
        if let Some(sfx) = &mut self.tv_noise {
            sfx.stop();
        }
        if let Some((_, sfx)) = &mut self.chase_music {
            sfx.stop();
        }
        self.piano_music.stop();
    }
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        geng.window().lock_cursor();

        let mut navmesh = if assets.config.create_navmesh {
            Self::init_navmesh(geng, &assets.level)
        } else {
            assets.navmesh.clone()
        };
        navmesh.remove_unreachable_from(assets.level.trigger_cubes["GhostSpawn"].center());

        Self {
            gf_clock_timer: 0.0,
            light_flicker_time: 0.0,
            rng: RngState::new(),
            game_over: false,
            game_over_t: 0.0,
            piano_music: {
                let mut sfx = assets.music.piano.effect();
                sfx.set_position(
                    find_center(
                        &assets
                            .level
                            .obj
                            .meshes
                            .iter()
                            .find(|mesh| mesh.name == "S_Piano")
                            .unwrap()
                            .geometry,
                    )
                    .map(|x| x as f64),
                );
                sfx.set_max_distance(2.0);
                sfx.play();
                sfx
            },
            chase_music: None,
            intro_t: if unsafe { BOOLEAN } { 0.1 } else { 21.0 },
            intro_skip_t: 0.0,
            music: None,
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
                flashdark: Flashdark {
                    pos: Vec3::ZERO,
                    rot_h: 0.0,
                    rot_v: 0.0,
                    dir: vec3(0.0, 1.0, 0.0),
                    on: true,
                    strength: 1.0,
                    dark: 0.0,
                },
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
            transition: None,
            noise: {
                let mut texture = ugli::Texture::new_with(geng.ugli(), vec2(1024, 1024), |_| {
                    Rgba::new(
                        global_rng().gen(),
                        global_rng().gen(),
                        global_rng().gen(),
                        global_rng().gen(),
                    )
                });
                texture.set_wrap_mode(ugli::WrapMode::Repeat);
                texture
            },
            intro_sfx: unsafe { !BOOLEAN }.then(|| assets.sfx.intro_sequence.play()),
        }
    }

    pub fn reset(&mut self) {
        self.transition = Some(geng::Transition::Switch(Box::new(Game::new(
            &self.geng,
            &self.assets,
        ))));
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let delta_time = delta_time as f32;
        self.rng.update(delta_time);
        self.update_impl(delta_time);
        if self.game_over {
            self.game_over_t += delta_time;
            self.player.height += (1.0 - self.player.height) * (delta_time / 0.1).min(1.0);
            let dv = self.monster.pos + vec3(0.0, 0.0, 1.0)
                - (self.player.pos + vec3(0.0, 0.0, self.player.height));
            let target_rot_h = dv.xy().arg() - f32::PI / 2.0; // + self.shake.x * 0.05;
            let target_rot_v = vec2(dv.xy().len(), dv.z).arg(); // + self.shake.y * 0.05;
            self.player.rot_h +=
                normalize_angle(target_rot_h - self.player.rot_h) * (delta_time / 0.1).min(1.0);
            self.player.rot_v += (target_rot_v - self.player.rot_v) * (delta_time / 0.1).min(1.0);
            self.lock_controls = true;
            if self.game_over_t > 6.0 {
                self.reset();
            }
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_impl(framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        if !self.lock_controls && self.intro_t < 0.0 {
            self.handle_event_camera(&event);
            self.handle_clicks(&event);
        }

        for button in &self.assets.config.controls.god_mode {
            if button.matches(&event) {
                self.player.god_mode = !self.player.god_mode;
                self.ambient_light = self.assets.config.ambient_light_inside_house;
                self.player.flashdark.dark = 1.0;
                // self.cutscene_t = 2.9;
                self.fuse_placed = true;
                self.lights.get_mut(&LightId(0)).unwrap().flicker_time = 2.0;
            }
        }
        for button in &self.assets.config.controls.toggle_fullscreen {
            if button.matches(&event) {
                self.geng.window().toggle_fullscreen();
            }
        }
        // TODO: remove
        match event {
            geng::Event::KeyDown { key: geng::Key::R }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.reset();
            }
            _ => {}
        }
    }
    fn transition(&mut self) -> Option<geng::Transition> {
        self.transition.take()
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();

    let geng = Geng::new_with(geng::ContextOptions {
        title: "FlashDark".to_string(),
        vsync: false,
        ..default()
    });
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            LoadingScreen::new(&geng),
            <Assets as geng::LoadAsset>::load(&geng, &static_path().join("assets")),
            {
                let geng = geng.clone();
                move |assets| Game::new(&geng, &Rc::new(assets.unwrap()))
            },
        ),
    );
}
