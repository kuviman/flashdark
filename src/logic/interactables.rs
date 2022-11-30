use super::*;

pub struct InteractableState {
    pub open: bool,
    pub open_other_way: bool,
    pub progress: f32,
    pub extra_hacky_library_moving_closet_progress: f32,
    pub data: Rc<InteractableData>,
    pub config: Rc<InteractableConfig>,
}

pub enum InteractableType {
    LDoor { pivot: Vec3<f32> },
    RDoor { pivot: Vec3<f32> },
    Drawer { shift: Vec3<f32> },
    Static,
}

impl InteractableState {
    pub fn matrix(&self) -> Mat4<f32> {
        let mut matrix = match self.data.typ {
            InteractableType::LDoor { pivot } => {
                Mat4::translate(pivot)
                    * Mat4::rotate_z(
                        -self.progress * f32::PI / 2.0
                            * if self.open_other_way { -1.0 } else { 1.0 },
                    )
                    * Mat4::translate(-pivot)
            }
            InteractableType::RDoor { pivot } => {
                Mat4::translate(pivot)
                    * Mat4::rotate_z(
                        self.progress * f32::PI / 2.0
                            * if self.open_other_way { -1.0 } else { 1.0 },
                    )
                    * Mat4::translate(-pivot)
            }
            InteractableType::Drawer { shift } => Mat4::translate(self.progress * shift),
            InteractableType::Static => Mat4::identity(),
        };
        matrix = Mat4::translate(vec3(
            0.0,
            self.extra_hacky_library_moving_closet_progress,
            0.0,
        )) * matrix;
        matrix
    }
}

impl Game {
    pub fn initialize_interactables(assets: &Assets, level: &LevelData) -> Vec<InteractableState> {
        let initial_storage_lock_config: [u8; 4] = loop {
            let config = std::array::from_fn(|_| global_rng().gen_range(0..4));
            if config != level.storage_lock_combination {
                break config;
            }
        };
        level
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
                let open = assets.config.open_interactables.contains(name)
                    || config.map_or(false, |config| config.open);
                Some(InteractableState {
                    open_other_way: config.map_or(false, |config| config.open_inverse),
                    open,
                    extra_hacky_library_moving_closet_progress: 0.0,
                    progress: if open { 1.0 } else { 0.0 },
                    data: data.clone(),
                    config: config.cloned().unwrap_or_default(),
                })
            })
            .collect()
    }

    pub fn light_out(&mut self) {
        self.key_puzzle_state = KeyPuzzleState::LightOut;
        self.assets.sfx.study_lights.play();
        self.ambient_light = Rgba::BLACK;
        self.player.flashdark.on = false;
        self.monster.pos = self.level.room_data["Kitchen"]
            .center()
            .xy()
            .extend(self.monster.pos.z);
        self.monster.scream_time = 0.0;
        self.monster.scan_timer_going = true;
        self.monster.next_pathfind_pos = self.monster.pos;
        self.monster.next_target_pos = self.monster.pos;
        unsafe {
            SEEN_LIGHT_OUT = true;
        }
    }

    pub fn update_interactables(&mut self, delta_time: f32) {
        for interactable in &mut self.interactables {
            let inter_time = if interactable.data.obj.meshes[0]
                .name
                .starts_with("DL_FenceDoor")
            {
                2.0
            } else if interactable.data.obj.meshes[0].name.starts_with("D") {
                0.6
            } else {
                0.3
            };
            let was_zero = interactable.progress == 0.0;
            if interactable.open {
                interactable.progress += delta_time / inter_time;
            } else {
                interactable.progress -= delta_time / inter_time;
            }
            interactable.progress = interactable.progress.clamp(0.0, 1.0);
            if !was_zero && interactable.progress == 0.0 {
                if interactable.data.obj.meshes[0].name == "D_DoorMain" {
                    if let Some(mut music) = self.music.take() {
                        music.stop();
                    }
                    self.music = Some({
                        let mut music = self.assets.sfx.ambient.effect();
                        music.set_volume(0.8);
                        music.play();
                        music
                    });
                }
            }
        }
    }

    pub fn click_interactable(&mut self, id: Id, player: bool, from: Vec3<f32>) {
        let interactable = &mut self.interactables[id];

        if interactable.data.obj.meshes[0].name.starts_with("B_Candle") {
            interactable.open = true;

            // TODO: sfx
            let all_candles = self
                .interactables
                .iter()
                .filter(|i| i.data.obj.meshes[0].name.starts_with("B_Candle"))
                .count();
            let lit_candles = self
                .interactables
                .iter()
                .filter(|i| i.data.obj.meshes[0].name.starts_with("B_Candle"))
                .filter(|i| !i.open)
                .count();
            self.ambient_light = Rgba::lerp(
                Rgba::BLACK,
                self.assets.config.ambient_light_inside_house,
                lit_candles as f32 / all_candles as f32,
            );
            if lit_candles == 0 {
                self.lock_controls = true;
                self.stop_sounds();
                self.ending = true;
                self.assets.sfx.ending.play();
            }
            self.assets.sfx.blow_candle.play();
            return;
        }

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
            let mut tv_noise = self.assets.sfx.tv_static.effect();
            let pos = self.level.trigger_cubes["GhostSpawn"].center();
            tv_noise.set_position(pos.map(|x| x as f64));
            // tv_noise.set_ref_distance((pos - self.camera.pos).len() as f64);
            tv_noise.set_max_distance(2.0);
            tv_noise.play();
            // self.swing_sfx.take().unwrap().stop();
            self.tv_noise = Some(tv_noise);
            self.ambient_light = self.assets.config.ambient_light_after_fuse;
        }

        if interactable.data.obj.meshes[0].name == "B_SingingGirl" {
            if let Some(mut sfx) = self.creepy_sing_sfx.take() {
                sfx.stop();
            }
        }

        // Key puzzle
        if interactable.data.obj.meshes[0].name == "D_DoorStudy" {
            if self.key_puzzle_state == KeyPuzzleState::Entered {
                self.light_out();
                return;
            }
        }
        let mut clear_keys = false;
        if interactable.data.obj.meshes[0].name == "I_StudyClosetLock" {
            self.key_puzzle_state = KeyPuzzleState::Finish;
            if let Some(mut sfx) = self.tv_noise.take() {
                sfx.stop();
            }
            clear_keys = true;
        }

        let sfx_position = find_center(&interactable.data.obj.meshes[0].geometry);

        if self.time != 0.0 {
            let sfx = if let Some(sfx) = interactable.config.sfx.as_deref() {
                self.assets.sfx.get_by_name(sfx)
            } else if interactable.data.obj.meshes[0]
                .name
                .starts_with("I_LoosePlank")
            {
                &self.assets.sfx.plank_removal
            } else if interactable.data.obj.meshes[0]
                .name
                .starts_with("I_StorageButton")
            {
                &self.assets.sfx.symbols_puzzle_button
            } else if interactable.data.obj.meshes[0].name.starts_with("D") {
                if interactable.open {
                    &self.assets.sfx.door_close
                } else {
                    &self.assets.sfx.door_open
                }
            } else if interactable.data.obj.meshes[0].name.starts_with("I_") {
                if interactable.open {
                    &self.assets.sfx.drawer_close
                } else {
                    &self.assets.sfx.drawer_open
                }
            } else {
                &self.assets.sfx.drawer_open // Girl sound?
            };
            let mut effect = sfx.effect();
            if let Some(volume) = interactable.config.sfx_volume {
                effect.set_volume(volume);
            }
            if interactable.data.obj.meshes[0].name != "I_FusePlaceholder" {
                effect.set_position(sfx_position.map(|x| x as f64));
                effect.set_max_distance(self.assets.config.max_sound_distance);
            }
            effect.play();
        }

        if !interactable.open
            && interactable.progress == 0.0
            && !interactable.data.obj.meshes[0].name.contains("Closet")
        {
            if let InteractableType::LDoor { pivot } = interactable.data.typ {
                interactable.open_other_way =
                    Vec2::skew((sfx_position - from).xy(), (pivot - from).xy()) > 0.0;
            }
            if let InteractableType::RDoor { pivot } = interactable.data.typ {
                interactable.open_other_way =
                    Vec2::skew((sfx_position - from).xy(), (pivot - from).xy()) < 0.0;
            }
        }
        interactable.open = !interactable.open;
        if interactable.config.use_item {
            self.player.item = None;
        }
        if let Some(give) = &interactable.config.give_item {
            self.player.item = Some(give.clone());
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
                open_other_way: false,
                open: false,
                progress: 0.0,
                extra_hacky_library_moving_closet_progress: 0.0,
                data: self
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

        if clear_keys {
            self.items.retain(|item| !item.name.contains("StudyKey"));
            self.interactables
                .retain(|i| !i.data.obj.meshes[0].name.contains("I_HintKey"));

            self.click_interactable(
                self.interactables
                    .iter()
                    .position(|i| i.data.obj.meshes[0].name == "D_DoorStudy")
                    .unwrap(),
                false,
                Vec3::ZERO,
            );
        }

        // Library puzzle
        {
            let current_library_puzzle_progress = self
                .interactables
                .iter()
                .filter(|i| i.data.obj.meshes[0].name.starts_with("I_BookshelfLibrary"))
                .filter(|i| i.open)
                .count();
            let mut p = current_library_puzzle_progress as f32 / 10.0;
            if current_library_puzzle_progress == 5 {
                p = 1.0;
            }
            for i in self
                .interactables
                .iter_mut()
                .filter(|i| i.data.obj.meshes[0].name.contains("LibraryMovingCloset"))
            {
                i.extra_hacky_library_moving_closet_progress = p;
            }
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
            if current_lock_combination == self.level.storage_lock_combination {
                self.interactables
                    .retain(|i| !i.data.obj.meshes[0].name.contains("StorageButton"));
                self.assets.sfx.symbols_puzzle_solved.play();
                self.storage_unlocked = true;
            }
        }
    }
}
