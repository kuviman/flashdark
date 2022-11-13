use super::*;

pub struct Item {
    pub name: String,
    pub mesh_index: usize,
    pub parent_interactable: Option<String>,
    pub pos: Vec3<f32>,
}

impl Game {
    pub fn initialize_items(assets: &Assets) -> Vec<Item> {
        let mut items = Vec::new();
        for (name, data) in assets.level.items.iter().filter(|(name, _data)| {
            if name.contains("Fuse") {
                return false;
            }
            if name.contains("StudyKey") {
                return false;
            }
            if *name == "Book5" {
                return false;
            }
            true
        }) {
            let index = global_rng().gen_range(0..data.spawns.len());
            let spawn = &data.spawns[index];
            items.push(Item {
                name: name.clone(),
                parent_interactable: spawn.parent_interactable.clone(),
                mesh_index: index,
                pos: spawn.pos,
            });
        }
        items
    }

    pub fn item_matrix(&self, item: &Item) -> Mat4<f32> {
        let mut matrix = Mat4::translate(item.pos);
        if let Some(parent) = &item.parent_interactable {
            let parent = self
                .interactables
                .iter()
                .find(|inter| inter.data.obj.meshes[0].name == *parent) // TODO: this is slow
                .unwrap();
            matrix = parent.matrix() * matrix;
        }
        matrix
    }

    pub fn click_item(&mut self, id: Id) {
        let item = self.items.remove(id);
        self.assets.sfx.genericPickup.play();
        if let Some(prev) = self.player.item.replace(item.name) {
            self.items.push(Item {
                name: prev,
                parent_interactable: None,
                mesh_index: 0,
                pos: self.player.pos,
            })
        }
    }
}
