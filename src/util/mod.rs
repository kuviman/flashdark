use super::*;

mod rng;
mod spatial_map;

pub use rng::*;
pub use spatial_map::*;

pub const EPS: f32 = 1e-7;

pub fn normalize_angle(mut a: f32) -> f32 {
    while a > f32::PI {
        a -= 2.0 * f32::PI;
    }
    while a < -f32::PI {
        a += 2.0 * f32::PI;
    }
    a
}

pub fn nlerp2(a: Vec2<f32>, b: Vec2<f32>, t: f32) -> Vec2<f32> {
    (a * (1.0 - t) + b * t).normalize_or_zero()
}

pub fn nlerp3(a: Vec3<f32>, b: Vec3<f32>, t: f32) -> Vec3<f32> {
    (a * (1.0 - t) + b * t).normalize_or_zero()
}

pub fn find_center(mesh: &[geng::obj::Vertex]) -> Vec3<f32> {
    let mut sum = Vec3::ZERO;
    for v in mesh {
        sum += v.a_v;
    }
    sum / mesh.len() as f32
}

pub fn intersect_ray_with_triangle(
    mut tri: [Vec3<f32>; 3],
    ext: f32,
    ray: geng::CameraRay,
) -> Option<f32> {
    let n = Vec3::cross(tri[1] - tri[0], tri[2] - tri[0]).normalize_or_zero();
    let mut new_tri = tri;
    for i in 0..3 {
        let j = (i + 1) % 3;
        let n = Vec3::cross(tri[j] - tri[i], n).normalize_or_zero();
        new_tri[i] += n * ext;
        new_tri[j] += n * ext;
    }
    tri = new_tri;
    // dot(ray.from + ray.dir * t - tri[0], n) = 0
    if Vec3::dot(ray.dir, n).abs() < EPS {
        return None;
    }
    let t = -Vec3::dot(ray.from - tri[0], n) / Vec3::dot(ray.dir, n);
    if t < EPS {
        return None;
    }
    let p = ray.from + ray.dir * t;
    // assert!(Vec3::dot(p - tri[0], n).abs() < EPS);
    for i in 0..3 {
        let p1 = tri[i];
        let p2 = tri[(i + 1) % 3];
        let v_inside = Vec3::cross(n, p2 - p1);
        if Vec3::dot(v_inside, p - p1) <= EPS {
            return None;
        }
    }
    Some(t)
}

pub fn intersect_ray_with_mesh(
    mesh: &ObjMesh,
    matrix: Mat4<f32>,
    ext: f32,
    ray: geng::CameraRay,
) -> Option<f32> {
    mesh.geometry
        .chunks(3)
        .flat_map(|tri| {
            intersect_ray_with_triangle(
                [tri[0].a_v, tri[1].a_v, tri[2].a_v].map(|pos| (matrix * pos.extend(1.0)).xyz()),
                ext,
                ray,
            )
        })
        .min_by_key(|&x| r32(x))
}

pub fn intersect_ray_with_obj(
    obj: &Obj,
    matrix: Mat4<f32>,
    ext: f32,
    ray: geng::CameraRay,
) -> Option<f32> {
    obj.meshes
        .iter()
        .flat_map(|mesh| intersect_ray_with_mesh(mesh, matrix, ext, ray))
        .min_by_key(|&x| r32(x))
}

pub fn vector_from_triangle(tri: [Vec3<f32>; 3], p: Vec3<f32>) -> Vec3<f32> {
    let mut options = vec![]; // TODO: optimize
    for v in tri {
        options.push(p - v);
    }
    for i in 0..3 {
        let p1 = tri[i];
        let p2 = tri[(i + 1) % 3];
        if Vec3::dot(p - p1, p2 - p1) <= EPS {
            continue;
        }
        if Vec3::dot(p - p2, p1 - p2) <= EPS {
            continue;
        }
        let v = (p2 - p1).normalize_or_zero();
        options.push(Vec3::cross(Vec3::cross(v, p - p1), v));
    }
    let n = Vec3::cross(tri[1] - tri[0], tri[2] - tri[0]).normalize_or_zero();
    let mut inside = true;
    for i in 0..3 {
        let p1 = tri[i];
        let p2 = tri[(i + 1) % 3];
        let v_inside = Vec3::cross(n, p2 - p1);
        if Vec3::dot(v_inside, p - p1) <= EPS {
            inside = false;
            break;
        }
    }
    if inside {
        options.push(n * Vec3::dot(n, p - tri[0]));
    }

    options.into_iter().min_by_key(|v| r32(v.len())).unwrap()
}

pub fn vector_from_obj(mesh: &Obj, matrix: Mat4<f32>, p: Vec3<f32>) -> Vec3<f32> {
    mesh.meshes
        .iter()
        .filter(|mesh| {
            if mesh.name.starts_with("B_SmallGrass")
                || mesh.name.starts_with("B_TallGrass")
                || mesh.name.starts_with("B_Tree")
            {
                return false;
            }
            true
        })
        .flat_map(|mesh| {
            mesh.geometry.chunks(3).map(|tri| {
                vector_from_triangle(
                    [tri[0].a_v, tri[1].a_v, tri[2].a_v]
                        .map(|pos| (matrix * pos.extend(1.0)).xyz()),
                    p,
                )
            })
        })
        .min_by_key(|v| r32(v.len()))
        .unwrap()
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Button {
    Key(geng::Key),
    Mouse(#[serde(with = "mouse_button")] geng::MouseButton),
}

impl Button {
    pub fn matches(&self, event: &geng::Event) -> bool {
        match *event {
            geng::Event::KeyDown { key } => *self == Self::Key(key),
            geng::Event::MouseDown { button, .. } => *self == Self::Mouse(button),
            _ => false,
        }
    }
    pub fn is_pressed(&self, geng: &Geng) -> bool {
        match *self {
            Button::Key(key) => geng.window().is_key_pressed(key),
            Button::Mouse(button) => geng.window().is_button_pressed(button),
        }
    }
}

mod mouse_button {
    use super::*;

    pub fn serialize<S>(value: &geng::MouseButton, ser: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("Mouse{value:?}").serialize(ser)
    }

    pub fn deserialize<'de, D>(de: D) -> Result<geng::MouseButton, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(de)?;
        if let Some(s) = s.strip_prefix("Mouse") {
            return Ok(match s {
                "Left" => geng::MouseButton::Left,
                "Right" => geng::MouseButton::Right,
                "Middle" => geng::MouseButton::Middle,
                _ => panic!(),
            });
        }
        unreachable!()
    }
}
