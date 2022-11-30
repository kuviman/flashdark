use super::*;

#[derive(Clone)]
pub struct Material {
    pub name: String,
    pub texture: Option<Rc<ugli::Texture>>,
    pub dark_texture: Option<Rc<ugli::Texture>>,
    // pub ambient_color: Rgba<f32>,
    // pub diffuse_color: Rgba<f32>,
}

#[derive(ugli::Vertex, Debug, Copy, Clone)]
pub struct Vertex {
    pub a_b: f32,
    pub a_v: Vec3<f32>,
    pub a_bv: Vec3<f32>,
    pub a_vt: Vec2<f32>,
    pub a_vn: Vec3<f32>,
}

pub struct ObjMesh {
    pub name: String,
    pub geometry: ugli::VertexBuffer<Vertex>,
    pub material: Material,
}

pub struct Obj {
    pub meshes: Vec<ObjMesh>,
    // pub size: f32,
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
            let mut current_material: Option<Material> = Some(Material {
                name: "".to_owned(),
                texture: None,
                dark_texture: None,
                // ambient_color: Rgba::WHITE,
                // diffuse_color: Rgba::WHITE,
            });
            let mut current_geometry = Vec::new();
            let mut materials = HashMap::<String, Material>::new();
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
                        Vertex {
                            a_b: 0.0,
                            a_v: v[i_v - 1],
                            a_bv: Vec3::ZERO,
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
                            material: {
                                let mut result = current_material.clone().unwrap();
                                if current_name.ends_with("_Dark") {
                                    mem::swap(&mut result.texture, &mut result.dark_texture);
                                }
                                result
                            },
                        });
                        current_geometry = Vec::new();
                    }
                    if line.starts_with("o ") || line.starts_with("g ") {
                        current_name = String::from(&line[2..]);
                    } else if let Some(name) = line.strip_prefix("usemtl ") {
                        current_material = Some(materials[name].clone());
                    }
                } else if let Some(mtl_path) = line.strip_prefix("mtllib ") {
                    for material in parse_mtl(&geng, dir, &dir.join(mtl_path)).await? {
                        materials.insert(material.name.clone(), material);
                    }
                }
            }
            for mesh in &mut meshes {
                if mesh.name.starts_with("B_") {
                    let center = find_center(&mesh.geometry);
                    for v in mesh.geometry.iter_mut() {
                        v.a_b = 1.0;
                        v.a_bv = v.a_v - center;
                        v.a_v = center;
                    }
                }
            }
            for i in (0..meshes.len()).rev() {
                for j in ((i + 1)..meshes.len()).rev() {
                    fn mergable(name: &str) -> bool {
                        if name.starts_with("D")
                            || name.starts_with("I")
                            || name.starts_with("RDC")
                            || name.starts_with("TC")
                            || name.contains("Spawn")
                            || name.contains("Light")
                            || name.contains("Symbol")
                            || name.contains("SwingingSwing")
                            || name.contains("SingingGirl")
                            || name.contains("S_PianoKeys")
                            || name.starts_with("AF_")
                            || name.starts_with("B_Candle")
                            || name.starts_with("S_MusicBox")
                        // TODO || name.starts_with("B_")
                        {
                            return false;
                        }
                        true
                    }
                    if meshes[i].material.name == meshes[j].material.name
                        && mergable(&meshes[i].name)
                        && mergable(&meshes[j].name)
                    {
                        let append = meshes
                            .remove(j)
                            .geometry
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>();
                        meshes[i].geometry.extend(append);
                    }
                }
            }
            info!(
                "{:?}",
                meshes
                    .iter()
                    .map(|mesh| mesh.name.as_str())
                    .collect::<Vec<_>>()
            );
            Ok(Obj {
                meshes,
                // size,
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("obj");
}

async fn parse_mtl(
    geng: &Geng,
    dir: &std::path::Path,
    path: &std::path::Path,
) -> anyhow::Result<Vec<Material>> {
    struct MaterialFuture {
        name: String,
        texture: geng::AssetFuture<Option<ugli::Texture>>,
        dark_texture: geng::AssetFuture<Option<ugli::Texture>>,
    }

    impl MaterialFuture {
        async fn into_future(self) -> Material {
            let (texture, dark_texture) = future::join(self.texture, self.dark_texture).await;
            let texture = texture.unwrap().map(Rc::new);
            let dark_texture = dark_texture.unwrap().map(Rc::new);
            Material {
                name: self.name,
                texture,
                dark_texture,
            }
        }
    }

    let mut materials = Vec::<MaterialFuture>::new();
    let mtl_source = <String as geng::LoadAsset>::load(geng, path).await?;
    let mut current_texture = future::ready(Ok(None)).boxed_local();
    let mut current_dark_texture = future::ready(Ok(None)).boxed_local();
    let mut current_name = "__unnamed__";
    let mut current_ambient_color = Rgba::WHITE;
    let mut current_diffuse_color = Rgba::WHITE;
    for line in mtl_source.lines().chain(std::iter::once("newmtl _")) {
        if let Some(texture_path) = line.strip_prefix("map_Kd ") {
            let texture_path = texture_path.split_whitespace().last().unwrap();
            // WTF .
            if texture_path != "." {
                current_texture =
                    <ugli::Texture as geng::LoadAsset>::load(&geng, &dir.join(texture_path))
                        .map_ok(|mut texture| {
                            make_repeated(&mut texture);
                            Some(texture)
                        })
                        .boxed_local();
                current_dark_texture = <ugli::Texture as geng::LoadAsset>::load(
                    &geng,
                    &dir.join(texture_path.strip_suffix(".png").unwrap().to_owned() + "_Dark.png"),
                )
                .map_ok(|mut texture| {
                    make_repeated(&mut texture);
                    Some(texture)
                })
                .map(|result| Ok(result.ok().flatten()))
                .boxed_local();
            }
        } else if let Some(name) = line.strip_prefix("newmtl ") {
            let name = name.trim();
            materials.push(MaterialFuture {
                name: current_name.to_owned(),
                texture: mem::replace(&mut current_texture, future::ready(Ok(None)).boxed_local()),
                dark_texture: mem::replace(
                    &mut current_dark_texture,
                    future::ready(Ok(None)).boxed_local(),
                ),
                // ambient_color: current_ambient_color,
                // diffuse_color: current_diffuse_color,
            });
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
    Ok(future::join_all(materials.into_iter().map(MaterialFuture::into_future)).await)
}
