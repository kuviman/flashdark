use super::*;

mod load;

pub use load::*;

pub struct InteractableData {
    pub obj: Obj,
    pub typ: InteractableType,
}

pub struct ItemSpawnData {
    pub mesh: ObjMesh,
    pub pos: Vec3<f32>,
    pub parent_interactable: Option<String>,
}

pub struct ItemData {
    pub spawns: Vec<ItemSpawnData>,
    pub texture_aabb: AABB<f32>,
}

pub struct LevelData {
    pub obj: Obj,
    pub skybox: ObjMesh,
    pub items: HashMap<String, ItemData>,
    pub key_configs: HashMap<String, KeyConfiguration>,
    pub hint_key_config: KeyConfiguration,
    pub storage_lock_combination: [u8; 4],
    pub interactables: Vec<Rc<InteractableData>>,
    pub spawn_point: Vec3<f32>,
    pub trigger_cubes: HashMap<String, TriggerCube>,
}
