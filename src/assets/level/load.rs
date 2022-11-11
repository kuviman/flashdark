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
                if obj.meshes[i].name.starts_with("I_") {
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
                    interactables.push(InteractableData {
                        obj: Obj { meshes: vec![mesh] },
                        typ: InteractableType::Drawer { shift },
                    });
                }
            }

            let mut items = HashMap::<String, ItemData>::new();
            for i in (0..obj.meshes.len()).rev() {
                if let Some(mut name) = obj.meshes[i].name.strip_prefix("Spawn_") {
                    if let Some(index) = name.find('.') {
                        name = &name[..index];
                    }
                    let name = name.to_owned();
                    let mut mesh = obj.meshes.remove(i);
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

            Ok(LevelData {
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
