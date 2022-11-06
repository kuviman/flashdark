use super::*;

pub struct Player {
    pub pos: Vec3<f32>,
    pub vel: Vec3<f32>,
    pub rot_h: f32,
    pub rot_v: f32,
    pub flashdark_strength: f32,
    pub flashdark_on: bool,
    pub flashdark_dir: Vec3<f32>,
    pub flashdark_pos: Vec3<f32>,
    pub item: Option<String>,
}
