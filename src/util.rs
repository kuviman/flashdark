use super::*;

pub const EPS: f32 = 1e-7;

pub fn intersect_ray_with_triangle(tri: [Vec3<f32>; 3], ray: geng::CameraRay) -> Option<f32> {
    let n = Vec3::cross(tri[1] - tri[0], tri[2] - tri[0]).normalize_or_zero();
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

pub fn intersect_ray_with_obj(mesh: &Obj, matrix: Mat4<f32>, ray: geng::CameraRay) -> Option<f32> {
    mesh.meshes
        .iter()
        .flat_map(|mesh| {
            mesh.geometry.chunks(3).flat_map(|tri| {
                intersect_ray_with_triangle(
                    [tri[0].a_v, tri[1].a_v, tri[2].a_v]
                        .map(|pos| (matrix * pos.extend(1.0)).xyz()),
                    ray,
                )
            })
        })
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
