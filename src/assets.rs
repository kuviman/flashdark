use super::*;

#[derive(geng::Assets)]
pub struct Shaders {
    pub wall: ugli::Program,
    pub billboard: ugli::Program,
    pub sprite: ugli::Program,
    pub horizontal_sprite: ugli::Program,
    pub vertical_sprite: ugli::Program,
    pub obj: ugli::Program,
}

pub fn make_repeated(texture: &mut ugli::Texture) {
    texture.set_wrap_mode(ugli::WrapMode::Repeat);
}

pub fn loop_sound(sound: &mut geng::Sound) {
    sound.looped = true;
}

#[derive(geng::Assets, Deserialize, Serialize, Clone, Debug)]
#[asset(json)]
pub struct Config {
    pub parents: HashMap<String, String>,
    pub open_interactables: HashSet<String>,
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
    #[asset(postprocess = "make_repeated")]
    pub wall: ugli::Texture,
    #[asset(postprocess = "make_repeated")]
    pub floor: ugli::Texture,
    #[asset(postprocess = "make_repeated")]
    pub ceiling: ugli::Texture,
    pub ghost: ugli::Texture,
    pub key: ugli::Texture,
    pub table_top: ugli::Texture,
    pub table_leg: ugli::Texture,
    pub bed_bottom: ugli::Texture,
    pub bed_back: ugli::Texture,
    pub hand: ugli::Texture,
    pub flashdark: ugli::Texture,
    #[asset(path = "box.png")]
    pub box_texture: ugli::Texture,
    #[asset(path = "table.obj")]
    pub obj: Obj,
    #[asset(path = "JumpScare1.wav")]
    pub jumpscare: geng::Sound,
    #[asset(path = "MainCreepyToneAmbient.wav", postprocess = "loop_sound")]
    pub music: geng::Sound,
    pub level: LevelData,
    pub config: Config,
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

pub struct Interactable {
    pub obj: Obj,
    pub typ: InteractableType,
}

pub struct ItemSpawnData {
    pub mesh: ObjMesh,
    pub pos: Vec3<f32>,
    pub parent_interactable: Option<usize>,
}

pub struct ItemData {
    pub spawns: Vec<ItemSpawnData>,
    pub texture_aabb: AABB<f32>,
}

pub struct LevelData {
    pub obj: Obj,
    pub items: HashMap<String, ItemData>,
    pub interactables: Vec<Interactable>,
    pub spawn_point: Vec3<f32>,
}

impl geng::LoadAsset for LevelData {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let mut obj = <Obj as geng::LoadAsset>::load(&geng, &path.join("roomMVP.obj")).await?;

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
                    interactables.push(Interactable {
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
                    interactables.push(Interactable {
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
                    interactables.push(Interactable {
                        obj: Obj { meshes: vec![mesh] },
                        typ: InteractableType::Drawer { shift },
                    });
                }
            }

            let mut items = HashMap::<String, ItemData>::new();
            for i in (0..obj.meshes.len()).rev() {
                if obj.meshes[i].name.starts_with("Spawn_") {
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
                        .entry(mesh.material.name.clone())
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
                                        return Some(index);
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
                interactables,
                items,
                obj,
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}
