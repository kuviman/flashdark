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
    pub crawling: ugli::Texture,
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
    pub level: LevelData,
    pub config: Config,
    pub navmesh: NavMesh,
    #[asset(path = "SFX")]
    pub sfx: SfxAssets,
}

#[derive(geng::Assets)]
pub struct Music {
    #[asset(path = "OutsideMusic.mp3", postprocess = "loop_sound")]
    pub outside: geng::Sound,
    #[asset(path = "ChaseMusic*.mp3", range = "1..=2", postprocess = "loop_sound")]
    pub chase: Vec<geng::Sound>,
    #[asset(path = "ThePiano.mp3", postprocess = "loop_sound")]
    pub piano: geng::Sound,
}

#[derive(geng::Assets)]
pub struct SfxAssets {
    #[asset(path = "HouseAmbient.mp3", postprocess = "loop_sound")]
    pub ambient: geng::Sound,
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
    #[asset(path = "fusePlaced.mp3")]
    pub fuse_placed: geng::Sound,
    #[asset(path = "introSequence.mp3")]
    pub intro_sequence: geng::Sound,
    #[asset(path = "genericPickup.mp3")]
    pub generic_pickup: geng::Sound,
    #[asset(path = "ghostLoop.mp3", postprocess = "loop_sound")]
    pub ghost_loop: geng::Sound,
    #[asset(path = "ghostScream.mp3")]
    pub ghost_scream: geng::Sound,
    #[asset(path = "placeObject.mp3")]
    pub place_object: geng::Sound,
    #[asset(path = "jumpScare1.mp3")]
    pub jumpscare: geng::Sound,
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
            _ => unreachable!(),
        }
    }
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub wall: ugli::Program,
    pub skybox: ugli::Program,
    pub billboard: ugli::Program,
    pub sprite: ugli::Program,
    pub horizontal_sprite: ugli::Program,
    pub vertical_sprite: ugli::Program,
    pub obj: ugli::Program,
    pub shadow: ugli::Program,
}
