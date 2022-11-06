use super::*;

pub struct InteractableState {
    pub open: bool,
    pub progress: f32,
    pub data: Rc<InteractableData>,
    pub config: Rc<InteractableConfig>,
}

pub enum InteractableType {
    LDoor { pivot: Vec3<f32> },
    RDoor { pivot: Vec3<f32> },
    Drawer { shift: Vec3<f32> },
}

impl InteractableType {
    pub fn matrix(&self, progress: f32) -> Mat4<f32> {
        match *self {
            Self::LDoor { pivot } => {
                Mat4::translate(pivot)
                    * Mat4::rotate_z(-progress * f32::PI / 2.0)
                    * Mat4::translate(-pivot)
            }
            Self::RDoor { pivot } => {
                Mat4::translate(pivot)
                    * Mat4::rotate_z(progress * f32::PI / 2.0)
                    * Mat4::translate(-pivot)
            }
            Self::Drawer { shift } => Mat4::translate(progress * shift),
        }
    }
}

impl Game {
    pub fn initialize_interactables(assets: &Assets) -> Vec<InteractableState> {
        assets
            .level
            .interactables
            .iter()
            .filter_map(|data| {
                let config = assets.config.interactables.get(&data.obj.meshes[0].name);
                if config.map_or(false, |config| config.hidden) {
                    return None;
                }
                Some(InteractableState {
                    open: assets
                        .config
                        .open_interactables
                        .contains(&data.obj.meshes[0].name),
                    progress: 0.0,
                    data: data.clone(),
                    config: config.cloned().unwrap_or_default(),
                })
            })
            .collect()
    }

    pub fn update_interactables(&mut self, delta_time: f32) {
        for state in &mut self.interactables {
            let inter_time = 0.3;
            if state.open {
                state.progress += delta_time / inter_time;
            } else {
                state.progress -= delta_time / inter_time;
            }
            state.progress = state.progress.clamp(0.0, 1.0);
        }
    }

    pub fn click_interactable(&mut self, id: Id) {
        let interactable = &mut self.interactables[id];

        if let Some(requirement) = &interactable.config.require_item {
            if self.player.item.as_ref() != Some(requirement) {
                return;
            }
        }

        interactable.open = !interactable.open;
        if interactable.config.use_item {
            self.player.item = None;
        }
        // TODO: clone not needed
        if let Some(transform) = interactable.config.transform_on_use.clone() {
            self.interactables.remove(id);
            self.interactables.push(InteractableState {
                open: false,
                progress: 0.0,
                data: self
                    .assets
                    .level
                    .interactables
                    .iter()
                    .find(|data| data.obj.meshes[0].name == transform)
                    .unwrap()
                    .clone(),
                config: self
                    .assets
                    .config
                    .interactables
                    .get(&transform)
                    .cloned()
                    .unwrap_or_default(),
            });
        }
    }
}