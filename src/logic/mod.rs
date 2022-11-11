use super::*;

mod camera;
mod flashdark;
mod interactables;
mod items;
mod monster;
mod movement;
mod navmesh;
mod player;

pub use camera::*;
pub use flashdark::*;
pub use interactables::*;
pub use items::*;
pub use monster::*;
pub use movement::*;
pub use navmesh::*;
pub use player::*;

impl Game {
    pub fn update_impl(&mut self, delta_time: f32) {
        let delta_time = delta_time.min(1.0 / 30.0);
        self.time += delta_time;

        self.update_movement(delta_time);
        self.update_camera(delta_time);
        self.update_flashdark(delta_time);
        self.update_interactables(delta_time);
        self.update_monster(delta_time);

        if let Some(target) = self.look().target {
            if let Object::Interactable(id) = target.object {
                let interactable = &self.interactables[id];
                if interactable.data.obj.meshes[0].name == "I_FusePlaceholder" {
                    if !self.fuse_spawned {
                        self.fuse_spawned = true;
                        let name = "Fuse";
                        let data = &self.assets.level.items[name];
                        let spawn_index = global_rng().gen_range(0..data.spawns.len());
                        let spawn = &data.spawns[spawn_index];
                        self.assets.sfx.drawerOpen.play(); // TODO: swing
                        self.items.push(Item {
                            name: name.to_owned(),
                            mesh_index: spawn_index,
                            parent_interactable: None,
                            pos: spawn.pos,
                        });
                    }
                }
            }
        }
    }

    pub fn handle_clicks(&mut self, event: &geng::Event) {
        if let geng::Event::MouseDown { button, .. } = *event {
            self.geng.window().lock_cursor();

            match button {
                geng::MouseButton::Left => {
                    if let Some(target) = self.look().target {
                        match target.object {
                            Object::StaticLevel => {}
                            Object::Interactable(id) => {
                                self.click_interactable(id, true);
                            }
                            Object::Item(id) => {
                                self.click_item(id);
                            }
                        }
                    }
                }
                geng::MouseButton::Right => {
                    self.toggle_flashdark();
                }
                _ => {}
            }
        }
    }
}
