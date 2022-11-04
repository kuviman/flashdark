use super::*;

pub struct Material {
    pub texture: Option<Rc<ugli::Texture>>,
    pub ambient_color: Rgba<f32>,
    pub diffuse_color: Rgba<f32>,
}

pub struct ObjMesh {
    pub name: String,
    pub geometry: ugli::VertexBuffer<geng::obj::Vertex>,
    pub material: Rc<Material>,
}

pub struct Obj {
    pub meshes: Vec<ObjMesh>,
    // pub size: f32,
}

impl Obj {
    pub fn plane(geng: &Geng, size: Vec2<f32>, texture: &Rc<ugli::Texture>) -> Self {
        Self {
            meshes: vec![ObjMesh {
                name: "plane".to_owned(),
                geometry: ugli::VertexBuffer::new_static(
                    geng.ugli(),
                    vec![
                        geng::obj::Vertex {
                            a_v: vec3(-size.x, -size.y, 0.0),
                            a_vt: vec2(-1.0, -1.0),
                            a_vn: vec3(0.0, 0.0, 1.0),
                        },
                        geng::obj::Vertex {
                            a_v: vec3(size.x, -size.y, 0.0),
                            a_vt: vec2(1.0, -1.0),
                            a_vn: vec3(0.0, 0.0, 1.0),
                        },
                        geng::obj::Vertex {
                            a_v: vec3(size.x, size.y, 0.0),
                            a_vt: vec2(1.0, 1.0),
                            a_vn: vec3(0.0, 0.0, 1.0),
                        },
                        geng::obj::Vertex {
                            a_v: vec3(-size.x, -size.y, 0.0),
                            a_vt: vec2(-1.0, -1.0),
                            a_vn: vec3(0.0, 0.0, 1.0),
                        },
                        geng::obj::Vertex {
                            a_v: vec3(size.x, size.y, 0.0),
                            a_vt: vec2(1.0, 1.0),
                            a_vn: vec3(0.0, 0.0, 1.0),
                        },
                        geng::obj::Vertex {
                            a_v: vec3(-size.x, size.y, 0.0),
                            a_vt: vec2(-1.0, 1.0),
                            a_vn: vec3(0.0, 0.0, 1.0),
                        },
                    ],
                ),
                material: Rc::new(Material {
                    texture: Some(texture.clone()),
                    ambient_color: Rgba::WHITE,
                    diffuse_color: Rgba::WHITE,
                }),
            }],
            // size: 1.0,
        }
    }
}

#[derive(ugli::Vertex, Debug, Clone)]
pub struct ObjInstance {
    pub i_model_matrix: Mat4<f32>,
    pub i_color: Rgba<f32>,
}

impl geng::LoadAsset for Obj {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let dir = path.parent().unwrap();
            let mut meshes = Vec::new();
            let obj_source = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut current_name = String::from("__unnamed__");
            let mut v = Vec::new();
            let mut vn = Vec::new();
            let mut vt = Vec::new();
            let mut current_material: Option<Rc<Material>> = Some(Rc::new(Material {
                texture: None,
                ambient_color: Rgba::WHITE,
                diffuse_color: Rgba::WHITE,
            }));
            let mut current_geometry = Vec::new();
            let mut materials = HashMap::<String, Rc<Material>>::new();
            for line in obj_source.lines().chain(std::iter::once("o _")) {
                if line.starts_with("v ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    v.push(vec3(x, y, z));
                } else if line.starts_with("vn ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    vn.push(vec3(x, y, z));
                } else if line.starts_with("vt ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    vt.push(vec2(x, y));
                } else if line.starts_with("f ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let to_vertex = |s: &str| {
                        let mut parts = s.split('/');
                        let i_v: usize = parts.next().unwrap().parse().unwrap();
                        let i_vt: Option<usize> = match parts.next().unwrap() {
                            "" => None,
                            s => Some(s.parse().unwrap()),
                        };
                        let i_vn: usize = parts.next().unwrap().parse().unwrap();
                        geng::obj::Vertex {
                            a_v: v[i_v - 1],
                            a_vn: vn[i_vn - 1],
                            a_vt: match i_vt {
                                Some(i_vt) => vt[i_vt - 1],
                                None => vec2(0.0, 0.0),
                            },
                        }
                    };
                    let mut cur = Vec::new();
                    for s in parts {
                        cur.push(to_vertex(s));
                    }
                    for i in 2..cur.len() {
                        current_geometry.push(cur[0]);
                        current_geometry.push(cur[i - 1]);
                        current_geometry.push(cur[i]);
                    }
                } else if line.starts_with("o ")
                    || line.starts_with("g ")
                    || line.starts_with("usemtl ")
                {
                    if !current_geometry.is_empty() {
                        meshes.push(ObjMesh {
                            name: current_name.clone(),
                            geometry: ugli::VertexBuffer::new_static(geng.ugli(), current_geometry),
                            material: current_material.clone().unwrap(),
                        });
                        current_geometry = Vec::new();
                    }
                    if line.starts_with("o ") || line.starts_with("g ") {
                        current_name = String::from(&line[2..]);
                    } else if let Some(name) = line.strip_prefix("usemtl ") {
                        current_material = Some(materials[name].clone());
                    }
                } else if let Some(mtl_path) = line.strip_prefix("mtllib ") {
                    let mtl_source =
                        <String as geng::LoadAsset>::load(&geng, &dir.join(mtl_path)).await?;
                    let mut current_texture = None;
                    let mut current_name = "__unnamed__";
                    let mut current_ambient_color = Rgba::WHITE;
                    let mut current_diffuse_color = Rgba::WHITE;
                    for line in mtl_source.lines().chain(std::iter::once("newmtl _")) {
                        if let Some(texture_path) = line.strip_prefix("map_Kd ") {
                            // WTF .
                            if texture_path != "." {
                                current_texture = Some(
                                    <ugli::Texture as geng::LoadAsset>::load(
                                        &geng,
                                        &dir.join(texture_path),
                                    )
                                    .await?,
                                );
                            }
                        } else if let Some(name) = line.strip_prefix("newmtl ") {
                            materials.insert(
                                current_name.to_owned(),
                                Rc::new(Material {
                                    texture: current_texture.take().map(Rc::new),
                                    ambient_color: current_ambient_color,
                                    diffuse_color: current_diffuse_color,
                                }),
                            );
                            current_name = name;
                        } else if let Some(color) = line.strip_prefix("Ka ") {
                            let mut parts = color.split_whitespace();
                            let r: f32 = parts.next().unwrap().parse().unwrap();
                            let g: f32 = parts.next().unwrap().parse().unwrap();
                            let b: f32 = parts.next().unwrap().parse().unwrap();
                            current_ambient_color = Rgba::new(r, g, b, 1.0);
                        } else if let Some(color) = line.strip_prefix("Kd ") {
                            let mut parts = color.split_whitespace();
                            let r: f32 = parts.next().unwrap().parse().unwrap();
                            let g: f32 = parts.next().unwrap().parse().unwrap();
                            let b: f32 = parts.next().unwrap().parse().unwrap();
                            current_diffuse_color = Rgba::new(r, g, b, 1.0);
                        }
                    }
                }
            }
            let size = meshes
                .iter()
                .flat_map(|mesh| {
                    mesh.geometry
                        .iter()
                        .map(|vertex| r32(vec2(vertex.a_v.x, vertex.a_v.z).len()))
                })
                .max()
                .unwrap()
                .as_f32();
            debug!("{:?} size is {:?}", path.file_name().unwrap(), size);
            Ok(Obj {
                meshes,
                // size,
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("obj");
}
