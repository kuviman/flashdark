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
pub struct Assets {
    pub shaders: Shaders,
    #[asset(postprocess = "make_repeated")]
    pub wall: ugli::Texture,
    #[asset(postprocess = "make_repeated")]
    pub floor: ugli::Texture,
    #[asset(postprocess = "make_repeated")]
    pub ceiling: ugli::Texture,
    pub ghost: ugli::Texture,
    pub ghost_front: ugli::Texture,
    pub key: ugli::Texture,
    pub table_top: ugli::Texture,
    pub table_leg: ugli::Texture,
    pub bed_bottom: ugli::Texture,
    pub bed_back: ugli::Texture,
    pub hand: ugli::Texture,
    pub flashdark: ugli::Texture,
    #[asset(path = "box.png")]
    pub box_texture: ugli::Texture,
    #[asset(path = "table.obj")]
    pub obj: Obj,
    #[asset(path = "JumpScare1.wav")]
    pub jumpscare: geng::Sound,
    #[asset(path = "MainCreepyToneAmbient.wav", postprocess = "loop_sound")]
    pub music: geng::Sound,
    pub level: LevelData,
    pub config: Config,
    pub navmesh: NavMesh,
    #[asset(path = "SFX")]
    pub sfx: SfxAssets,
}
#[derive(geng::Assets)]
pub struct SfxAssets {
    pub doorClose: geng::Sound,
    pub doorLocked: geng::Sound,
    pub doorOpen: geng::Sound,
    pub doorUnlocked: geng::Sound,
    pub drawerClose: geng::Sound,
    pub drawerOpen: geng::Sound,
    pub flashOff: geng::Sound,
    pub flashOn: geng::Sound,
    pub fusePlaced: geng::Sound,
    pub genericPickup: geng::Sound,
    #[asset(postprocess = "loop_sound")]
    pub ghostLoop: geng::Sound,
    pub ghostScream: geng::Sound,
    pub placeObject: geng::Sound,
    pub tvStatic: geng::Sound,
    #[asset(path = "ghostAlarmed*.wav", range = "1..=3")]
    pub ghostAlarmed: Vec<geng::Sound>,
    #[asset(path = "footstep*.wav", range = "1..=4")]
    pub footsteps: Vec<geng::Sound>,
    #[asset(path = "footstepCreak*.wav", range = "1..=4")]
    pub footstepCreaks: Vec<geng::Sound>,
}

impl SfxAssets {
    pub fn get_by_name(&self, name: &str) -> &geng::Sound {
        match name {
            "placeObject.wav" => &self.placeObject,
            "doorLocked.wav" => &self.doorLocked,
            "doorUnlocked.wav" => &self.doorUnlocked,
            _ => unreachable!(),
        }
    }
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub wall: ugli::Program,
    pub billboard: ugli::Program,
    pub sprite: ugli::Program,
    pub horizontal_sprite: ugli::Program,
    pub vertical_sprite: ugli::Program,
    pub obj: ugli::Program,
}
