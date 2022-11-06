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
}

#[derive(geng::Assets, Deserialize, Serialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    pub parents: HashMap<String, String>,
    pub open_interactables: HashSet<String>,
    pub interactables: HashMap<String, Rc<InteractableConfig>>,
}
