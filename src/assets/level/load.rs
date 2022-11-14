use super::*;

pub struct TriggerCube {
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}
impl TriggerCube {
    pub fn center(&self) -> Vec3<f32> {
        vec3(
            self.min_x + self.max_x,
            self.min_y + self.max_y,
            self.min_z + self.max_z,
        ) / 2.0
    }
    pub fn horizontal_aabb(&self) -> AABB<f32> {
        AABB {
            x_min: self.min_x,
            x_max: self.max_x,
            y_min: self.min_y,
            y_max: self.max_y,
        }
    }
}

fn update_key_uvs(mesh: &mut ObjMesh, config: KeyConfiguration) {
    let update = |vertices: &mut [geng::obj::Vertex], (x, y)| {
        for v in vertices {
            v.a_vt.x += 0.25 * x as f32;
            v.a_vt.y += 0.25 * y as f32;
        }
    };
    update(
        &mut mesh.geometry[0..3],
        (config.top_color, config.top_shape),
    );
    update(
        &mut mesh.geometry[6..9],
        (config.top_color, config.top_shape),
    );
    update(
        &mut mesh.geometry[3..6],
        (config.bottom_color, config.bottom_shape),
    );
    update(
        &mut mesh.geometry[9..12],
        (config.bottom_color, config.bottom_shape),
    );
}

fn update_storage_lock_uvs(mesh: &mut ObjMesh, num: u8) {
    let mid = mesh.geometry.iter().map(|v| v.a_vt.x).sum::<f32>() / mesh.geometry.len() as f32;
    for v in mesh.geometry.iter_mut() {
        v.a_vt.x = 0.25 * num as f32 + if v.a_vt.x < mid { 0.0 } else { 0.25 };
    }
}

impl geng::LoadAsset for LevelData {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let mut obj = <Obj as geng::LoadAsset>::load(&geng, &path.join("roomMVP.obj")).await?;

            obj.meshes.retain(|mesh| {
                if mesh.name.starts_with("RDC_") {
                    // TODO rooms
                    return false;
                }
                if mesh.name.starts_with("H_") {
                    // TODO window holes?
                    return false;
                }
                true
            });

            let storage_lock_combination = std::array::from_fn(|_| global_rng().gen_range(0..4));

            for mesh in &mut obj.meshes {
                if mesh.name.starts_with("S_Grass") || mesh.name.starts_with("S_Ceiling") {
                    for v in mesh.geometry.iter_mut() {
                        v.a_vt = v.a_v.xy() / 2.0;
                    }
                }
                if mesh.name.starts_with("S_Walls") {
                    for v in mesh.geometry.iter_mut() {
                        v.a_vt = vec2((v.a_v.x + v.a_v.y) / 2.0, v.a_v.z / 2.0);
                    }
                }

                if mesh.name == "S_StudySymbol2" {
                    update_storage_lock_uvs(mesh, storage_lock_combination[0]);
                }
                if mesh.name == "S_LivingSymbol1" {
                    update_storage_lock_uvs(mesh, storage_lock_combination[1]);
                }
                if mesh.name == "S_LivingSymbol2" {
                    update_storage_lock_uvs(mesh, storage_lock_combination[2]);
                }
                if mesh.name == "S_StudySymbol1" {
                    update_storage_lock_uvs(mesh, storage_lock_combination[3]);
                }
            }

            let mut trigger_cubes = HashMap::new();
            for i in (0..obj.meshes.len()).rev() {
                // info!("{:?}", obj.meshes[i].name);
                let name = &obj.meshes[i].name;
                let trigger_name = if let Some(name) = name.strip_prefix("TC_") {
                    Some(name)
                } else if name == "GhostSpawn" {
                    Some(name.as_str())
                } else {
                    None
                };
                if let Some(name) = trigger_name {
                    let name = name.to_owned();
                    info!("Found trigger cube: {:?}", name);
                    let mesh = obj.meshes.remove(i);
                    let mut min = mesh.geometry[0].a_v;
                    let mut max = mesh.geometry[0].a_v;
                    for v in mesh.geometry.iter() {
                        min.x = min.x.min(v.a_v.x);
                        min.y = min.y.min(v.a_v.y);
                        min.z = min.z.min(v.a_v.z);
                        max.x = max.x.max(v.a_v.x);
                        max.y = max.y.max(v.a_v.y);
                        max.z = max.z.max(v.a_v.z);
                    }
                    trigger_cubes.insert(
                        name,
                        TriggerCube {
                            min_x: min.x,
                            min_y: min.y,
                            min_z: min.z,
                            max_x: max.x,
                            max_y: max.y,
                            max_z: max.z,
                        },
                    );
                }
            }

            let mut interactables = Vec::new();
            let mut planks = Vec::new();
            for i in (0..obj.meshes.len()).rev() {
                if obj.meshes[i].name.starts_with("D_") || obj.meshes[i].name.starts_with("DR_") {
                    let mesh = obj.meshes.remove(i);
                    let pivot = mesh
                        .geometry
                        .iter()
                        .max_by_key(|v| r32(v.a_vt.x))
                        .unwrap()
                        .a_v;
                    interactables.push(InteractableData {
                        obj: Obj { meshes: vec![mesh] },
                        typ: InteractableType::RDoor { pivot },
                    });
                }
                if obj.meshes[i].name.starts_with("DL_") {
                    let mesh = obj.meshes.remove(i);
                    let pivot = mesh
                        .geometry
                        .iter()
                        .min_by_key(|v| r32(v.a_vt.x))
                        .unwrap()
                        .a_v;
                    interactables.push(InteractableData {
                        obj: Obj { meshes: vec![mesh] },
                        typ: InteractableType::LDoor { pivot },
                    });
                }
                if obj.meshes[i].name.starts_with("I_")
                    || obj.meshes[i].name.starts_with("B_SingingGirl")
                {
                    let mesh = obj.meshes.remove(i);
                    let front_face = mesh
                        .geometry
                        .chunks(3)
                        .max_by_key(|face| face.iter().map(|v| r32(v.a_vt.x)).max().unwrap())
                        .unwrap();
                    let shift = Vec3::cross(
                        front_face[1].a_v - front_face[0].a_v,
                        front_face[2].a_v - front_face[0].a_v,
                    )
                    .normalize_or_zero()
                        * 0.3;
                    if mesh.name.starts_with("I_StorageButtonIcon") {
                        for n in 0..4 {
                            let mut mesh = ObjMesh {
                                name: format!("{}{}", mesh.name, n),
                                geometry: ugli::VertexBuffer::new_static(
                                    geng.ugli(),
                                    mesh.geometry.clone(),
                                ),
                                material: mesh.material.clone(),
                            };
                            update_storage_lock_uvs(&mut mesh, n);
                            interactables.push(InteractableData {
                                obj: Obj { meshes: vec![mesh] },
                                typ: InteractableType::Drawer { shift },
                            });
                        }
                    } else if mesh.name.starts_with("I_LoosePlank") {
                        planks.push(InteractableData {
                            obj: Obj { meshes: vec![mesh] },
                            typ: InteractableType::Drawer { shift },
                        });
                    } else if mesh.name.contains("BookshelfLibrary") {
                        interactables.push(InteractableData {
                            obj: Obj { meshes: vec![mesh] },
                            typ: InteractableType::Static,
                        });
                    } else {
                        interactables.push(InteractableData {
                            obj: Obj { meshes: vec![mesh] },
                            typ: InteractableType::Drawer { shift },
                        });
                    }
                }
            }

            const PLANKS_N: usize = 4;
            for _ in 0..PLANKS_N {
                let i = global_rng().gen_range(0..planks.len());
                interactables.push(planks.remove(i));
            }

            let mut items = HashMap::<String, ItemData>::new();
            let mut key_configs = HashMap::new();
            for i in (0..obj.meshes.len()).rev() {
                if let Some(name) = obj.meshes[i].name.strip_prefix("Spawn_") {
                    let mut name = name.to_owned();
                    let mut mesh = obj.meshes.remove(i);
                    if name.contains("StudyKey") {
                        let config = KeyConfiguration::random();
                        update_key_uvs(&mut mesh, config);
                        key_configs.insert(name.clone(), config);
                    } else {
                        if let Some(index) = name.find('.') {
                            name = name[..index].to_owned();
                        }
                    }
                    let mut sum = Vec3::ZERO;
                    for v in mesh.geometry.iter() {
                        sum += v.a_v;
                    }
                    let center = sum / mesh.geometry.len() as f32;
                    for v in mesh.geometry.iter_mut() {
                        v.a_v -= center;
                    }
                    items
                        .entry(name)
                        .or_insert(ItemData {
                            spawns: vec![],
                            texture_aabb: AABB::points_bounding_box(
                                mesh.geometry.iter().map(|v| v.a_vt),
                            ),
                        })
                        .spawns
                        .push(ItemSpawnData {
                            mesh,
                            pos: center,
                            parent_interactable: interactables.iter().enumerate().find_map(
                                |(index, inter)| {
                                    let mesh = &inter.obj.meshes[0];
                                    // if mesh.name.starts_with("I_LoosePlank") {
                                    //     return None;
                                    // }
                                    let mut min = mesh.geometry[0].a_v;
                                    let mut max = mesh.geometry[0].a_v;
                                    for v in mesh.geometry.iter() {
                                        min.x = min.x.min(v.a_v.x);
                                        min.y = min.y.min(v.a_v.y);
                                        min.z = min.z.min(v.a_v.z);
                                        max.x = max.x.max(v.a_v.x);
                                        max.y = max.y.max(v.a_v.y);
                                        max.z = max.z.max(v.a_v.z);
                                    }
                                    let off = 0.1;
                                    min -= vec3(off, off, off);
                                    max += vec3(off, off, off);
                                    if min.x < center.x
                                        && center.x < max.x
                                        && min.y < center.y
                                        && center.y < max.y
                                        && min.z < center.z
                                        && center.z < max.z
                                    {
                                        return Some(inter.obj.meshes[0].name.clone());
                                    }
                                    None
                                },
                            ),
                        });
                }
            }

            {
                let doll_spawns = &mut items.get_mut("Doll").unwrap().spawns;
                doll_spawns.retain(|spawn| spawn.parent_interactable.is_some());
                for spawn in doll_spawns.iter_mut() {
                    spawn.parent_interactable = None;
                }
                assert!(doll_spawns.len() == PLANKS_N);
            }

            let hint_key_config = **key_configs
                .values()
                .collect::<Vec<_>>()
                .choose(&mut global_rng())
                .unwrap();
            for interactable in &mut interactables {
                if interactable.obj.meshes[0].name == "I_HintKey" {
                    update_key_uvs(&mut interactable.obj.meshes[0], hint_key_config);
                }
            }
            let mut key_solution_item_data = ItemData {
                spawns: vec![],
                texture_aabb: AABB::point(Vec2::ZERO),
            };
            for (item_name, config) in &key_configs {
                if *config == hint_key_config {
                    let data = items.remove(item_name).unwrap();
                    key_solution_item_data.spawns.extend(data.spawns);
                }
            }
            items.insert(
                "TheStudyKeyPuzzleSolution".to_owned(),
                key_solution_item_data,
            );
            key_configs.insert("TheStudyKeyPuzzleSolution".to_owned(), hint_key_config);

            Ok(LevelData {
                key_configs,
                storage_lock_combination,
                hint_key_config,
                spawn_point: {
                    let index = obj
                        .meshes
                        .iter()
                        .position(|mesh| mesh.name == "PlayerSpawn")
                        .unwrap();
                    let mesh = obj.meshes.remove(index);
                    let mut sum = Vec3::ZERO;
                    for v in mesh.geometry.iter() {
                        sum += v.a_v;
                    }
                    sum / mesh.geometry.len() as f32
                },
                interactables: interactables.into_iter().map(Rc::new).collect(),
                items,
                obj,
                trigger_cubes,
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
