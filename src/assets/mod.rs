use super::*;

mod config;
mod level;
mod obj;

pub use config::*;
pub use level::*;
pub use obj::*;

pub fn make_repeated(texture: &mut ugli::Texture) {
    if texture.is_pot() {
        texture.set_wrap_mode(ugli::WrapMode::Repeat);
    }
}

pub fn loop_sound(sound: &mut geng::Sound) {
    sound.looped = true;
}

pub fn loop_sound_all(sounds: &mut [geng::Sound]) {
    for sound in sounds {
        loop_sound(sound);
    }
}

#[derive(geng::Assets)]
pub struct GhostDirections {
    pub front: ugli::Texture,
    pub back: ugli::Texture,
    pub left: ugli::Texture,
    pub right: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct GhostAssets {
    pub normal: GhostDirections,
    pub chasing: GhostDirections,
    pub dying: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct TutorialAssets {
    pub flashlight: ugli::Texture,
    pub crouch: ugli::Texture,
    pub intro: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    pub ghost: GhostAssets,
    pub hand: ugli::Texture,
    pub reticle: ugli::Texture,
    pub require_item: ugli::Texture,
    pub flashdark: ugli::Texture,
    #[asset(path = "Music")]
    pub music: Music,
    #[asset(path = "level/roomMVP.obj")]
    pub level_obj: Obj,
    pub config: Config,
    pub navmesh: NavMesh,
    #[asset(path = "SFX")]
    pub sfx: SfxAssets,
    pub ui: UiAssets,
    #[asset(path = "VFX/dustParticle.png")]
    pub dust_particle: ugli::Texture,
    #[asset(path = "VFX/glowParticle.png")]
    pub glow_particle: ugli::Texture,
    #[asset(path = "VFX/pentagramFire.png")]
    pub pentagram_fire: ugli::Texture,
    #[asset(path = "difficulty/*.json", range = "1..=3")]
    pub difficulties: Vec<Difficulty>,
    pub tutorial: TutorialAssets,
    pub tobecontinued: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct UiAssets {
    pub icon_flashlight: ugli::Texture,
    pub flashlight: ugli::Texture,
    pub title: ugli::Texture,
    pub label_controls: ugli::Texture,
    pub play: ugli::Texture,
    pub icon_settings: ugli::Texture,
    pub icon_door: ugli::Texture,
    pub icon_controls: ugli::Texture,
    pub icon_back: ugli::Texture,
    pub label_mouse_sensitivity: ugli::Texture,
    pub label_soundvolume: ugli::Texture,
    pub slider_handle1: ugli::Texture,
    pub slider_handle2: ugli::Texture,
    pub slider_line: ugli::Texture,
    pub icon_home: ugli::Texture,
    pub icon_arrow_left: ugli::Texture,
    pub icon_arrow_right: ugli::Texture,
    pub label_difficulty: ugli::Texture,
    pub label_easy: ugli::Texture,
    pub label_hard: ugli::Texture,
    // Donde esta la leche
    pub label_normal: ugli::Texture,
}

#[derive(geng::Assets)]
pub struct Music {
    #[asset(path = "OutsideMusic.mp3", postprocess = "loop_sound")]
    pub outside: geng::Sound,
    #[asset(path = "AnxietyMusic.mp3", postprocess = "loop_sound")]
    pub anxiety: geng::Sound,
    #[asset(
        path = "ChaseMusic*.mp3",
        range = "1..=2",
        postprocess = "loop_sound_all"
    )]
    pub chase: Vec<geng::Sound>,
    #[asset(path = "ThePiano.mp3", postprocess = "loop_sound")]
    pub piano: geng::Sound,
    #[asset(path = "MusicBox.mp3")]
    pub music_box: geng::Sound,
    #[asset(path = "CreepySinging.mp3")]
    pub creepy_singing: geng::Sound,
}

#[derive(geng::Assets)]
pub struct SfxAssets {
    #[asset(path = "HouseAmbient.mp3", postprocess = "loop_sound")]
    pub ambient: geng::Sound,
    #[asset(path = "clockChime.mp3")]
    pub grand_clock: geng::Sound,
    #[asset(path = "blowCandle.mp3")]
    pub blow_candle: geng::Sound,
    #[asset(path = "brokenFlashlight.mp3")]
    pub broken_flashlight: geng::Sound,
    #[asset(path = "FlashdarkEndingSequence.mp3")]
    pub ending: geng::Sound,
    #[asset(path = "doorClose.mp3")]
    pub door_close: geng::Sound,
    #[asset(path = "doorLocked.mp3")]
    pub door_locked: geng::Sound,
    #[asset(path = "doorOpen.mp3")]
    pub door_open: geng::Sound,
    #[asset(path = "doorUnlocked.mp3")]
    pub door_unlocked: geng::Sound,
    #[asset(path = "drawerClose.mp3")]
    pub drawer_close: geng::Sound,
    #[asset(path = "drawerOpen.mp3")]
    pub drawer_open: geng::Sound,
    #[asset(path = "flashOff.mp3")]
    pub flash_off: geng::Sound,
    #[asset(path = "flashOn.mp3")]
    pub flash_on: geng::Sound,
    #[asset(path = "swingLoop.mp3", postprocess = "loop_sound")]
    pub swing_loop: geng::Sound,
    #[asset(path = "fusePlacedSequence.mp3")]
    pub fuse_placed: geng::Sound,
    #[asset(path = "introSequence.mp3")]
    pub intro_sequence: geng::Sound,
    #[asset(path = "genericPickup.mp3")]
    pub generic_pickup: geng::Sound,
    #[asset(path = "ghostLoop.mp3", postprocess = "loop_sound")]
    pub ghost_loop: geng::Sound,
    #[asset(path = "ghostScream.mp3")]
    pub ghost_scream: geng::Sound,
    #[asset(path = "studyLightsOutScare.mp3")]
    pub study_lights: geng::Sound,
    #[asset(path = "placeObject.mp3")]
    pub place_object: geng::Sound,
    #[asset(path = "gameOverScare.mp3")]
    pub jumpscare: geng::Sound,
    #[asset(path = "curtainsOpen.mp3")]
    pub curtains_open: geng::Sound,
    #[asset(path = "curtainsClose.mp3")]
    pub curtains_close: geng::Sound,
    #[asset(path = "lightFlicker.mp3")]
    pub light_flicker: geng::Sound,
    #[asset(path = "tvStatic.mp3", postprocess = "loop_sound")]
    pub tv_static: geng::Sound,
    #[asset(path = "ghostAlarmed*.mp3", range = "1..=3")]
    pub ghost_alarmed: Vec<geng::Sound>,
    #[asset(path = "footstep*.mp3", range = "1..=4")]
    pub footsteps: Vec<geng::Sound>,
    #[asset(path = "footstepCreak*.mp3", range = "1..=3")]
    pub footstep_creaks: Vec<geng::Sound>,
}

impl SfxAssets {
    pub fn get_by_name(&self, name: &str) -> &geng::Sound {
        match name {
            "placeObject.mp3" => &self.place_object,
            "doorLocked.mp3" => &self.door_locked,
            "doorUnlocked.mp3" => &self.door_unlocked,
            "fusePlaced.mp3" => &self.fuse_placed,
            "curtainsOpen.mp3" => &self.curtains_open,
            "curtainsClose.mp3" => &self.curtains_close,
            _ => unreachable!(),
        }
    }
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub skybox: ugli::Program,
    pub billboard: ugli::Program,
    pub obj: ugli::Program,
    pub shadow: ugli::Program,
    pub particle: ugli::Program,
    pub shine: ugli::Program,
}
