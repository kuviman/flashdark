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
    pub items: HashMap<String, ItemData>,
    pub interactables: Vec<Rc<InteractableData>>,
    pub spawn_point: Vec3<f32>,
}