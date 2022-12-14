use super::*;

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct InteractableConfig {
    pub require_item: Option<String>,
    #[serde(default)]
    pub use_item: bool,
    pub transform_on_use: Option<String>,
    pub give_item: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub dissapear_on_use: bool,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub open_inverse: bool,
    pub sfx: Option<String>,
    pub sfx_volume: Option<f64>,
}

#[derive(geng::Assets, Deserialize, Serialize, Clone, Debug)]
#[asset(json)]
pub struct Difficulty {
    pub peek_distance: f32,
    pub monster_180_range: f32,
    pub monster_detect_time: f32,
    pub monster_scan_time: f32,
    pub monster_scan_radius: f32,
    pub ghost_stand_still_time: (f32, f32),
    pub monster_view_distance: f32,
    pub monster_fov: f32,
    pub crouch_detect_time_multiplier: f32,
    pub monster_chase_speed: ((f32, f32), (f32, f32)),
    pub max_ghost_sound_distance: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Controls {
    pub crouch: Vec<Button>,
    pub interact: Vec<Button>,
    pub god_mode: Vec<Button>,
    pub toggle_fullscreen: Vec<Button>,
    pub toggle_flashdark: Vec<Button>,
    pub move_forward: Vec<Button>,
    pub move_backward: Vec<Button>,
    pub move_left: Vec<Button>,
    pub move_right: Vec<Button>,
    pub drop_item: Vec<Button>,
    pub pause: Vec<Button>,
}

#[derive(geng::Assets, Deserialize, Serialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    pub main_menu_cameras: Vec<Camera>,
    pub controls: Controls,
    pub create_navmesh: bool,
    pub flashdark_flicker_interval: f32,
    pub flashdark_turn_off_probability: f32,
    pub tv_detection_angle: f32,
    pub sky_color: Rgba<f32>,
    pub ambient_light: Rgba<f32>,
    pub ambient_light_after_fuse: Rgba<f32>,
    pub ambient_light_inside_house: Rgba<f32>,
    pub footstep_dist: f32,
    pub max_sound_distance: f64,
    pub arms_horizontal_length: f32,
    pub arms_vertical_length: f32,
    pub parents: HashMap<String, String>,
    pub open_interactables: HashSet<String>,
    pub interactables: HashMap<String, Rc<InteractableConfig>>,
}
