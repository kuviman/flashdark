#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]
use geng::prelude::*;

mod assets;
mod camera;
mod draw;
mod id;
mod loading_screen;
mod logic;
mod particles;
mod util;

pub use assets::*;
pub use camera::*;
pub use draw::*;
pub use id::*;
pub use loading_screen::*;
pub use logic::*;
pub use particles::*;
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

static mut BEEN_INSIDE_HOUSE: bool = false;
static mut INTRO_SEEN: bool = false;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UiAction {
    None,
    Settings,
    Exit,
    Play,
    Back,
    ChangeVolume,
    ChangeMouseSens,
    IncDifficulty,
    DecDifficulty,
    Home,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Settings {
    pub mouse_sens: f32,
    pub volume: f32,
    pub difficulty: usize,
}

const DEFAULT_DIFF: usize = 0;

impl Default for Settings {
    fn default() -> Self {
        Self {
            mouse_sens: 0.5,
            volume: 0.5,
            difficulty: 0,
        }
    }
}

pub struct Game {
    player_inside_house: bool,
    show_crouch_tutorial: bool,
    draw_calls: Cell<usize>,
    show_flashlight_tutorial: bool,
    main_menu: bool,
    in_settings: bool,
    settings: Settings,
    difficulty: Difficulty,
    main_menu_next_camera: f32,
    main_menu_next_camera_index: usize,
    hover_ui_action: Option<UiAction>,
    gf_clock_timer: f32,
    light_flicker_time: f32,
    start_drag: Vec2<f32>,
    ui_mouse_pos: Vec2<f32>,
    rng: RngState,
    game_over: bool,
    game_over_sfx: Option<geng::SoundEffect>,
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
    ending: bool,
    ending_t: f32,
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
    particles: Particles,
    level: LevelData,
}

impl Drop for Game {
    fn drop(&mut self) {
        self.stop_sounds();
        batbox::preferences::save("flashdark.json", &self.settings);
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
        if let Some(sfx) = &mut self.swing_sfx {
            sfx.stop();
        }
        if let Some(sfx) = &mut self.intro_sfx {
            sfx.stop();
        }
        // if you're reading this, there's one very important thing you should know:
        if let Some(sfx) = &mut self.game_over_sfx {
            sfx.stop();
        }
        self.piano_music.stop();
        self.monster.stop_sounds();
    }
    pub fn new(geng: &Geng, assets: &Rc<Assets>, main_menu: bool) -> Self {
        let level = LevelData::generate(geng, &assets.level_obj);
        if main_menu {
            geng.window().unlock_cursor();
            unsafe {
                BEEN_INSIDE_HOUSE = false;
                INTRO_SEEN = false;
            }
        } else {
            geng.window().lock_cursor();
        }
        geng.window().set_cursor_type(geng::CursorType::None);

        let mut navmesh = if assets.config.create_navmesh {
            Self::init_navmesh(geng, &level)
        } else {
            assets.navmesh.clone()
        };
        navmesh.remove_unreachable_from(level.trigger_cubes["GhostSpawn"].center());

        let mut res = Self {
            ending_t: 0.0,
            ending: false,
            player_inside_house: false,
            show_flashlight_tutorial: true,
            show_crouch_tutorial: true,
            difficulty: assets.difficulties[DEFAULT_DIFF].clone(),
            start_drag: Vec2::ZERO,
            ui_mouse_pos: Vec2::ZERO,
            settings: batbox::preferences::load("flashdark.json").unwrap_or_default(),
            main_menu,
            main_menu_next_camera: 0.0,
            in_settings: false,
            hover_ui_action: None,
            main_menu_next_camera_index: 0,
            gf_clock_timer: 0.0,
            light_flicker_time: 0.0,
            rng: RngState::new(),
            game_over: false,
            game_over_sfx: None,
            game_over_t: 0.0,
            piano_music: {
                let mut sfx = assets.music.piano.effect();
                sfx.set_position(
                    find_center(
                        &level
                            .obj
                            .meshes
                            .iter()
                            .find(|mesh| mesh.name == "S_Piano")
                            .unwrap()
                            .geometry,
                    )
                    .map(|x| x as f64),
                );
                sfx.set_max_distance(4.0);
                sfx.play();
                sfx
            },
            chase_music: None,
            intro_t: if unsafe { INTRO_SEEN } { 0.1 } else { 21.0 },
            intro_skip_t: 0.0,
            music: main_menu.then(|| assets.music.anxiety.play()),
            storage_unlocked: false,
            key_puzzle_state: KeyPuzzleState::Begin,
            monster_spawned: unsafe { BEEN_INSIDE_HOUSE },
            cutscene_t: if unsafe { BEEN_INSIDE_HOUSE } {
                2.9
            } else {
                0.0
            },
            draw_calls: Cell::new(0),
            lock_controls: false,
            ambient_light: assets.config.ambient_light,
            tv_noise: main_menu.then(|| {
                let mut tv_noise = assets.sfx.tv_static.effect();
                let pos = level.trigger_cubes["GhostSpawn"].center();
                tv_noise.set_position(pos.map(|x| x as f64));
                // tv_noise.set_ref_distance((pos - self.camera.pos).len() as f64);
                tv_noise.set_max_distance(2.0);
                tv_noise.play();
                tv_noise
            }),
            swing_sfx: (main_menu || unsafe { BEEN_INSIDE_HOUSE }).then(|| {
                let mut swing_sfx = assets.sfx.swing_loop.effect();
                swing_sfx.set_position(
                    level.trigger_cubes["SwingingSwing"]
                        .center()
                        .map(|x| x as f64),
                );
                swing_sfx.set_max_distance(2.0);
                swing_sfx.play();
                swing_sfx
            }),
            current_swing_ref_distance: 10000.0,
            fuse_placed: unsafe { BEEN_INSIDE_HOUSE },
            time: 0.0,
            items: Self::initialize_items(assets, &level),
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
            fuse_spawned: main_menu || unsafe { BEEN_INSIDE_HOUSE }, // LOL
            interactables: Self::initialize_interactables(assets, &level),
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            player: Player {
                crouch: false,
                pos: level.spawn_point,
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
                pos: level.spawn_point,
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
            monster: Monster::new(assets, &level),
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
            intro_sfx: unsafe { !INTRO_SEEN && !main_menu }
                .then(|| assets.sfx.intro_sequence.play()),
            particles: Particles::new(geng),
            level,
        };
        if unsafe { BEEN_INSIDE_HOUSE } {
            res.player.item = Some("Fuse".to_owned());
            res.click_interactable(
                res.interactables
                    .iter()
                    .position(|i| i.data.obj.meshes[0].name == "I_FusePlaceholder")
                    .unwrap(),
                true,
                Vec3::ZERO,
            );
        }
        res
    }

    pub fn reset(&mut self) {
        self.transition = Some(geng::Transition::Switch(Box::new(Game::new(
            &self.geng,
            &self.assets,
            false,
        ))));
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        self.sens = 0.0002 + self.settings.mouse_sens * 0.01;
        self.geng.audio().set_volume(self.settings.volume as f64);
        self.difficulty = self.assets.difficulties[self.settings.difficulty].clone();
        let delta_time = delta_time as f32;
        self.update_particles(delta_time);
        self.rng.update(delta_time);
        self.update_impl(delta_time);
        if self.main_menu {
            self.main_menu_next_camera -= delta_time;
            if self.main_menu_next_camera < 0.0 {
                self.main_menu_next_camera += 6.0;
                self.camera =
                    self.assets.config.main_menu_cameras[self.main_menu_next_camera_index].clone();
                self.main_menu_next_camera_index += 1;
                self.main_menu_next_camera_index %= self.assets.config.main_menu_cameras.len();
            }
        }
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
        if self.ending {
            self.ending_t += delta_time;
            self.player.height = 1.0;
            self.lock_controls = true;
            let target_rot_h = 0.0;
            let target_rot_v = 0.0;
            let t = (delta_time / 0.3).min(1.0);
            self.player.rot_h += (target_rot_h - self.player.rot_h) * t;
            self.player.rot_v += (target_rot_v - self.player.rot_v) * t;
            let pentagram_pos = find_center(
                &self
                    .assets
                    .level_obj
                    .meshes
                    .iter()
                    .find(|m| m.name == "S_Pentagram")
                    .unwrap()
                    .geometry,
            )
            .xy();
            self.player.pos +=
                ((pentagram_pos + vec2(0.0, -2.0)).extend(self.player.pos.z) - self.player.pos) * t;
            self.monster.dir = vec3(0.0, -1.0, 0.0);
            self.player.flashdark.on = false;
            if self.ending_t < 1.0 {
                let t = self.ending_t;
                self.monster.pos = (pentagram_pos + vec2(0.5, 2.0) + (1.0 - t) * vec2(2.5, 0.0))
                    .extend(self.monster.pos.z);
                self.monster.target_type = TargetType::Rng;
                self.monster.speed = 1.0;
            } else if self.ending_t < 2.0 {
                self.monster.target_type = TargetType::Player;
                self.monster.speed = 10.0;
            } else if self.ending_t < 3.0 {
                let t = self.ending_t - 2.0;
                self.monster.pos =
                    (pentagram_pos + (1.0 - t) * vec2(0.5, 2.0)).extend(self.monster.pos.z);
            } else if self.ending_t < 12.0 {
                let t = (self.ending_t - 3.0) / 9.0;
                self.monster.pos = pentagram_pos.extend(0.0)
                    + vec3(self.rng.get(20.0), 0.0, self.rng.get(20.0)) * 0.1
                    + vec3(0.0, 0.0, -(t.sqr() * 2.0));
            }

            if self.ending_t > 15.0 {
                for i in &mut self.interactables {
                    if i.data.obj.meshes[0].name.starts_with("B_Candle") {
                        i.open = false;
                    }
                }
                // let t = self.ending_t - 10.0;
                // self.monster.pos = pentagram_pos.extend(0.0) + vec3(0.0, -t, t.powf(0.5) - 1.5);
            }
            if self.ending_t > 20.0 {
                self.transition = Some(geng::Transition::Switch(Box::new(Game::new(
                    &self.geng,
                    &self.assets,
                    true,
                ))));
            }
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.draw_calls.set(0);
        self.draw_impl(framebuffer);
        // info!("Draw calls: {}", self.draw_calls.get());
    }

    fn handle_event(&mut self, event: geng::Event) {
        if !self.lock_controls && self.intro_t < 0.0 && !self.main_menu && !self.in_settings {
            self.handle_event_camera(&event);
            self.handle_clicks(&event);
            if self
                .assets
                .config
                .controls
                .crouch
                .iter()
                .any(|button| button.matches(&event))
            {
                self.player.crouch = !self.player.crouch;
            }
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
        if !self.main_menu && self.intro_t < 0.0 && !self.ending {
            for button in &self.assets.config.controls.pause {
                if button.matches(&event) {
                    self.in_settings = !self.in_settings;
                    if !self.in_settings {
                        batbox::preferences::save("flashdark.json", &self.settings);
                    }
                    if !self.main_menu && !self.in_settings {
                        self.geng.window().lock_cursor();
                    } else {
                        self.geng.window().unlock_cursor();
                    }
                }
            }
        }
        // TODO: remove
        match event {
            geng::Event::KeyDown { key: geng::Key::R }
                if self.geng.window().is_key_pressed(geng::Key::LCtrl) =>
            {
                self.reset();
            }
            #[cfg(not(target_arch = "wasm32"))]
            geng::Event::KeyDown { key: geng::Key::I } => {
                serde_json::to_writer_pretty(
                    std::fs::File::create("saved_pos.json").unwrap(),
                    &self.camera,
                )
                .unwrap();
            }
            geng::Event::MouseDown {
                button: geng::MouseButton::Left,
                ..
            } => {
                if let Some(action) = self.hover_ui_action {
                    match action {
                        UiAction::Settings => self.in_settings = true,
                        UiAction::Exit => self.transition = Some(geng::Transition::Pop),
                        UiAction::Play => self.reset(),
                        UiAction::Back => {
                            self.in_settings = false;
                            batbox::preferences::save("flashdark.json", &self.settings);
                            if !self.main_menu {
                                self.geng.window().lock_cursor();
                            } else {
                                self.geng.window().unlock_cursor();
                            }
                        }
                        UiAction::ChangeVolume => {
                            self.start_drag = self.ui_mouse_pos;
                        }
                        UiAction::ChangeMouseSens => {
                            self.start_drag = self.ui_mouse_pos;
                        }
                        UiAction::None => {}
                        UiAction::Home => {
                            self.transition = Some(geng::Transition::Switch(Box::new(Game::new(
                                &self.geng,
                                &self.assets,
                                true,
                            ))));
                        }
                        UiAction::IncDifficulty => {
                            self.settings.difficulty =
                                (self.settings.difficulty + 1) % self.assets.difficulties.len()
                        }
                        UiAction::DecDifficulty => {
                            self.settings.difficulty =
                                (self.settings.difficulty + self.assets.difficulties.len() - 1)
                                    % self.assets.difficulties.len()
                        }
                    }
                }
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
                move |assets| Game::new(&geng, &Rc::new(assets.unwrap()), true)
            },
        ),
    );
}
