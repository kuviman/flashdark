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
}

#[derive(geng::Assets)]
pub struct Shaders {
    pub wall: ugli::Program,
    pub billboard: ugli::Program,
    pub sprite: ugli::Program,
    pub horizontal_sprite: ugli::Program,
    pub vertical_sprite: ugli::Program,
    pub obj: ugli::Program,
    pub shadow: ugli::Program,
}
