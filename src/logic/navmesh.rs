use super::*;

#[derive(geng::Assets, Serialize, Deserialize, Clone)]
#[asset(json)]
pub struct NavMesh {
    pub waypoints: Vec<Vec3<f32>>,
    pub edges: Vec<Vec<usize>>,
    // pub debug_obj: Obj,
}

impl NavMesh {
    pub fn closest_waypoint(&self, p: Vec3<f32>) -> usize {
        self.waypoints
            .iter()
            .copied()
            .enumerate()
            .min_by_key(|(index, w)| r32((*w - p).len_sqr()))
            .unwrap()
            .0
    }
    pub fn pathfind(&self, p1: Vec3<f32>, p2: Vec3<f32>) -> Vec3<f32> {
        let s = self.closest_waypoint(p2);
        let t = self.closest_waypoint(p1);

        let h = |v| -> R32 { r32((self.waypoints[t] - self.waypoints[v]).len()) };

        let mut d = vec![1e9f32; self.waypoints.len()];
        let mut f = vec![r32(1e9); self.waypoints.len()];
        let mut p = vec![0; self.waypoints.len()];

        let mut q = std::collections::BinaryHeap::new();
        d[s] = 0.0;
        p[s] = s;
        f[s] = h(s);
        q.push((-f[s], s));
        while let Some((ff, v)) = q.pop() {
            if v == t {
                return self.waypoints[p[v]];
            }
            if ff != -f[v] {
                continue;
            }
            for u in self.edges[v].iter().copied() {
                let cost = (self.waypoints[v] - self.waypoints[u]).len();
                d[u] = d[u].min(d[v] + cost);
                let new_f = r32(d[u]) + h(u);
                if new_f < f[u] {
                    f[u] = new_f;
                    q.push((-f[u], u));
                    p[u] = v;
                }
            }
        }
        error!("Could not pathfind");
        p1
    }

    pub fn remove_unreachable_from(&mut self, p: Vec3<f32>) {
        let mut q = std::collections::VecDeque::new();
        let mut reachable = vec![false; self.waypoints.len()];
        let s = self.closest_waypoint(p);
        q.push_back(s);
        reachable[s] = true;
        while let Some(v) = q.pop_front() {
            for &u in &self.edges[v] {
                if !reachable[u] {
                    reachable[u] = true;
                    q.push_back(u);
                }
            }
        }
        let mut new_indices = vec![0; self.waypoints.len()];
        let mut next = 0;
        let mut new_waypoints = Vec::new();
        for v in 0..self.waypoints.len() {
            if reachable[v] {
                new_waypoints.push(self.waypoints[v]);
                new_indices[v] = next;
                next += 1;
            }
        }
        let mut new_edges = vec![vec![]; new_waypoints.len()];
        for v in 0..self.waypoints.len() {
            if reachable[v] {
                new_edges[new_indices[v]] = self.edges[v].iter().map(|&u| new_indices[u]).collect();
            }
        }
        *self = Self {
            waypoints: new_waypoints,
            edges: new_edges,
        }
    }
}

impl Game {
    pub fn init_navmesh(geng: &Geng, level: &LevelData) -> NavMesh {
        let hor_range = -15.0..15.0;
        let ver_range = 0.0..1.0;
        const HOR_GRID_SIZE: usize = 30;
        const VER_GRID_SIZE: usize = 5;
        const MIN_DISTANCE_TO_MESH: f32 = 0.1;
        let hor_step = (hor_range.end - hor_range.start) / HOR_GRID_SIZE as f32;
        let waypoints: Vec<Vec3<f32>> = {
            let obj = &level.obj;
            let mut points = HashMap::<Vec2<usize>, Vec<Vec3<f32>>>::new();
            for xi in 0..=HOR_GRID_SIZE {
                let x = hor_range.start
                    + (hor_range.end - hor_range.start) * xi as f32 / HOR_GRID_SIZE as f32;
                for yi in 0..=HOR_GRID_SIZE {
                    let y = hor_range.start
                        + (hor_range.end - hor_range.start) * yi as f32 / HOR_GRID_SIZE as f32;
                    let points = points.entry(vec2(xi, yi)).or_default();
                    for zi in 0..=VER_GRID_SIZE {
                        let z = ver_range.start
                            + (ver_range.end - ver_range.start) * zi as f32 / VER_GRID_SIZE as f32;
                        let mut p = vec3(x, y, z);
                        if let Some(t) = intersect_ray_with_obj(
                            obj,
                            Mat4::identity(),
                            geng::CameraRay {
                                from: p,
                                dir: vec3(0.0, 0.0, -1.0),
                            },
                        ) {
                            p.z -= t;
                        }
                        p.z += MIN_DISTANCE_TO_MESH * 2.0;
                        if vector_from_obj(obj, Mat4::identity(), p).len() < MIN_DISTANCE_TO_MESH {
                            continue;
                        }
                        if points
                            .iter()
                            .any(|&other| (other - p).len() < MIN_DISTANCE_TO_MESH)
                        {
                            continue;
                        }
                        points.push(p);
                    }
                }
            }
            points.into_values().flatten().collect()
        };
        let mut edges = vec![vec![]; waypoints.len()];
        let max_hor_connectivity = hor_step * 3.0;
        let max_ver_connectivity = 0.7;

        let mut spatial_map = SpatialMap::new(r32(max_hor_connectivity));
        for v in 0..waypoints.len() {
            spatial_map.insert(v, waypoints[v].xy().map(|x| r32(x)), R32::ZERO);
        }
        for v in 0..waypoints.len() {
            for u in spatial_map.lookup(
                AABB::point(waypoints[v].xy())
                    .extend_uniform(max_hor_connectivity)
                    .map(|x| r32(x)),
            ) {
                if v == u {
                    continue;
                }
                if (waypoints[v] - waypoints[u]).xy().len() > max_hor_connectivity {
                    continue;
                }
                if (waypoints[v] - waypoints[u]).z.abs() > max_ver_connectivity {
                    continue;
                }
                let ray = geng::CameraRay {
                    from: waypoints[v],
                    dir: waypoints[u] - waypoints[v],
                };
                if let Some(t) = intersect_ray_with_obj(&level.obj, Mat4::identity(), ray) {
                    if t < 1.0 {
                        continue;
                    }
                }
                edges[v].push(u);
                edges[u].push(v);
            }
        }
        let result = NavMesh {
            waypoints,
            edges,
            // debug_obj,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            serde_json::to_writer(
                std::fs::File::create(static_path().join("assets").join("navmesh.json")).unwrap(),
                &result,
            )
            .unwrap();
        }
        result
    }

    pub fn draw_debug_navmesh(&self, framebuffer: &mut ugli::Framebuffer) {
        return;
        let debug_obj = Obj {
            meshes: vec![ObjMesh {
                name: "debug navmesh".to_owned(),
                geometry: ugli::VertexBuffer::new_static(self.geng.ugli(), {
                    let mut vs = Vec::new();
                    for v in 0..self.navmesh.waypoints.len() {
                        for u in self.navmesh.edges[v].iter().copied() {
                            let v = self.navmesh.waypoints[v];
                            let u = self.navmesh.waypoints[u];
                            let n = (v.xy() - u.xy())
                                .rotate_90()
                                .normalize_or_zero()
                                .extend(0.0)
                                * 0.05;
                            let quad = [v + n, u + n, u - n, v - n];

                            fn vertex(p: Vec3<f32>) -> geng::obj::Vertex {
                                geng::obj::Vertex {
                                    a_v: p,
                                    a_vt: Vec2::ZERO,
                                    a_vn: Vec3::ZERO,
                                }
                            }
                            vs.push(vertex(quad[0]));
                            vs.push(vertex(quad[1]));
                            vs.push(vertex(quad[2]));
                            vs.push(vertex(quad[0]));
                            vs.push(vertex(quad[2]));
                            vs.push(vertex(quad[3]));
                        }
                    }
                    vs
                }),
                material: Material {
                    name: "debug".to_owned(),
                    texture: None,
                    dark_texture: None,
                },
            }],
        };
        self.draw_obj(
            framebuffer,
            &debug_obj,
            Mat4::identity(),
            Rgba::new(1.0, 1.0, 1.0, 0.3),
        );
    }
}
