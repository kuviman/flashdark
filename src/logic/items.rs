use super::*;

pub struct Item {
    pub name: String,
    pub matrix: Mat4<f32>,
    pub mesh_index: usize,
    pub parent_interactable: Option<String>,
}

impl Game {
    pub fn initialize_items(assets: &Assets, level: &LevelData) -> Vec<Item> {
        let mut items = Vec::new();
        for (name, data) in level.items.iter().filter(|(name, _data)| {
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
                matrix: Mat4::translate(spawn.pos),
                name: name.clone(),
                parent_interactable: spawn.parent_interactable.clone(),
                mesh_index: index,
            });
        }
        items
    }

    pub fn item_matrix(&self, item: &Item) -> Mat4<f32> {
        let mut matrix = item.matrix; // Mat4::translate(item.pos);
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

    pub fn create_dropped(&mut self, name: String) {
        let matrix = Mat4::translate(self.player.pos + vec3(0.0, 0.0, -0.24));
        self.items.push(Item {
            mesh_index: if name == "Doll" { 1000 } else { 0 },
            name,
            matrix,
            parent_interactable: None,
        })
    }

    pub fn click_item(&mut self, id: Id) {
        let item = self.items.remove(id);
        self.assets.sfx.generic_pickup.play();
        if let Some(prev) = self.player.item.replace(item.name) {
            self.create_dropped(prev);
        }
    }

    pub fn drop_item(&mut self) {
        if let Some(item) = self.player.item.take() {
            self.assets.sfx.generic_pickup.play();
            self.create_dropped(item);
        }
    }
}
