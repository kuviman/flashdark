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
        let initial_storage_lock_config: [u8; 4] = loop {
            let config = std::array::from_fn(|_| global_rng().gen_range(0..4));
            if config != assets.level.storage_lock_combination {
                break config;
            }
        };
        assets
            .level
            .interactables
            .iter()
            .filter_map(|data| {
                let name = &data.obj.meshes[0].name;
                if name == "I_HintKey" {
                    return None;
                }
                if let Some(numbers) = name.strip_prefix("I_StorageButtonIcon") {
                    let mut numbers = numbers.chars().map(|c| c.to_digit(10).unwrap());
                    let index = numbers.next().unwrap() as usize - 1;
                    let value = numbers.next().unwrap() as u8;
                    if initial_storage_lock_config[index] != value {
                        return None;
                    }
                }
                let config = assets.config.interactables.get(name);
                if config.map_or(false, |config| config.hidden) {
                    return None;
                }
                Some(InteractableState {
                    open: assets.config.open_interactables.contains(name),
                    progress: 0.0,
                    data: data.clone(),
                    config: config.cloned().unwrap_or_default(),
                })
            })
            .collect()
    }

    pub fn update_interactables(&mut self, delta_time: f32) {
        for interactable in &mut self.interactables {
            let inter_time = if interactable.data.obj.meshes[0].name.starts_with("D") {
                0.6
            } else {
                0.3
            };
            if interactable.open {
                interactable.progress += delta_time / inter_time;
            } else {
                interactable.progress -= delta_time / inter_time;
            }
            interactable.progress = interactable.progress.clamp(0.0, 1.0);
        }
    }

    pub fn click_interactable(&mut self, id: Id, player: bool) {
        let interactable = &mut self.interactables[id];

        if self.key_puzzle_state == KeyPuzzleState::LightOut {
            return;
        }

        let mut requirement = interactable.config.require_item.as_deref();
        if interactable.data.obj.meshes[0]
            .name
            .starts_with("I_LoosePlank")
        {
            requirement = Some("Crowbar");
        }
        if let Some(requirement) = requirement {
            if !player {
                return;
            }
            if self.player.item.as_deref() != Some(requirement) {
                return;
            }
        }

        // Fix the fuse
        if interactable.data.obj.meshes[0].name == "I_FusePlaceholder" {
            self.fuse_placed = true;
            let mut tv_noise = self.assets.sfx.tvStatic.effect();
            let pos = self.assets.level.trigger_cubes["GhostSpawn"].center();
            tv_noise.set_position(pos.map(|x| x as f64));
            tv_noise.set_ref_distance((pos - self.camera.pos).len() as f64);
            tv_noise.set_max_distance(self.assets.config.max_sound_distance);
            tv_noise.play();
            self.swing_sfx.take().unwrap().stop();
            self.tv_noise = Some(tv_noise);
            self.ambient_light = self.assets.config.ambient_light_after_fuse;
        }

        // Key puzzle
        if interactable.data.obj.meshes[0].name == "D_DoorStudy" {
            if self.key_puzzle_state == KeyPuzzleState::Entered {
                self.key_puzzle_state = KeyPuzzleState::LightOut;
                self.ambient_light = Rgba::BLACK;
                self.player.flashdark_on = false;
                return;
            }
        }

        let sfx_position = find_center(&interactable.data.obj.meshes[0].geometry);

        let sfx = if let Some(sfx) = interactable.config.sfx.as_deref() {
            self.assets.sfx.get_by_name(sfx)
        } else if interactable.data.obj.meshes[0].name.starts_with("D") {
            if interactable.open {
                &self.assets.sfx.doorClose
            } else {
                &self.assets.sfx.doorOpen
            }
        } else if interactable.data.obj.meshes[0].name.starts_with("I_") {
            if interactable.open {
                &self.assets.sfx.drawerClose
            } else {
                &self.assets.sfx.drawerOpen
            }
        } else {
            unreachable!()
        };
        let mut effect = sfx.effect();
        effect.set_position(sfx_position.map(|x| x as f64));
        effect.set_max_distance(self.assets.config.max_sound_distance);
        effect.play();

        interactable.open = !interactable.open;
        if interactable.config.use_item {
            self.player.item = None;
        }
        // TODO: clone not needed
        let mut transform = interactable.config.transform_on_use.clone();
        // Storage lock puzzle
        let mut check_storage_lock = false;
        if let Some(numbers) = interactable.data.obj.meshes[0]
            .name
            .strip_prefix("I_StorageButtonIcon")
        {
            let mut numbers = numbers.chars().map(|c| c.to_digit(10).unwrap());
            let _index = numbers.next().unwrap() as usize;
            let value = numbers.next().unwrap() as u8;
            let value = (value + 1) % 4;
            let mut new_name = interactable.data.obj.meshes[0].name.clone();
            new_name.pop();
            new_name += &value.to_string();
            transform = Some(new_name);
            check_storage_lock = true;
        }
        if let Some(transform) = transform {
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
        } else if interactable.config.dissapear_on_use
            || interactable.data.obj.meshes[0]
                .name
                .starts_with("I_LoosePlank")
        {
            self.interactables.remove(id);
        }

        if player {
            self.check_monster_sfx(sfx_position);
        }

        if check_storage_lock {
            let current_lock_combination = std::array::from_fn(|i| {
                self.interactables
                    .iter()
                    .filter_map(|interactable| {
                        interactable.data.obj.meshes[0]
                            .name
                            .strip_prefix(&format!("I_StorageButtonIcon{}", i + 1))
                    })
                    .next()
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap()
                    .to_digit(10)
                    .unwrap() as u8
            });
            if current_lock_combination == self.assets.level.storage_lock_combination {
                self.interactables
                    .retain(|i| !i.data.obj.meshes[0].name.contains("StorageButton"));
                self.storage_unlocked = true;
            }
        }
    }
}
