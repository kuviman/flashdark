use super::*;

pub struct Flashdark {
    pub strength: f32,
    pub on: bool,
    pub rot_h: f32,
    pub rot_v: f32,
    pub dir: Vec3<f32>,
    pub pos: Vec3<f32>,
    pub dark: f32,
}

pub struct Player {
    pub pos: Vec3<f32>,
    pub vel: Vec3<f32>,
    pub height: f32,
    pub rot_h: f32,
    pub rot_v: f32,
    pub flashdark: Flashdark,
    pub item: Option<String>,
    pub next_footstep: f32,
    pub god_mode: bool,
}
