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
    pub sfx: Option<String>,
}

#[derive(geng::Assets, Deserialize, Serialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    pub sky_color: Rgba<f32>,
    pub footstep_dist: f32,
    pub max_sound_distance: f64,
    pub arms_horizontal_length: f32,
    pub arms_vertical_length: f32,
    pub parents: HashMap<String, String>,
    pub open_interactables: HashSet<String>,
    pub interactables: HashMap<String, Rc<InteractableConfig>>,
}
