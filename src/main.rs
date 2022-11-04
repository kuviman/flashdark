use geng::prelude::*;

mod obj;

use obj::*;

const EPS: f32 = 1e-7;

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

pub fn intersect_ray_with_obj(mesh: &Obj, ray: geng::CameraRay) -> Option<f32> {
    mesh.meshes
        .iter()
        .flat_map(|mesh| {
            mesh.geometry.chunks(3).flat_map(|tri| {
                intersect_ray_with_triangle([tri[0].a_v, tri[1].a_v, tri[2].a_v], ray)
            })
        })
        .min_by_key(|&x| r32(x))
}

pub fn vector_from_triangle(tri: [Vec3<f32>; 3], p: Vec3<f32>) -> Vec3<f32> {
    let mut options = vec![];
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

pub fn vector_from_obj(mesh: &Obj, p: Vec3<f32>) -> Vec3<f32> {
    mesh.meshes
        .iter()
        .flat_map(|mesh| {
            mesh.geometry
                .chunks(3)
                .map(|tri| vector_from_triangle([tri[0].a_v, tri[1].a_v, tri[2].a_v], p))
        })
        .min_by_key(|v| r32(v.len()))
        .unwrap()
}

pub struct Camera {
    pub fov: f32,
    pub pos: Vec3<f32>,
    pub rot_h: f32,
    pub rot_v: f32,
}

impl Camera {
    pub const MIN_ROT_V: f32 = -f32::PI / 2.0;
    pub const MAX_ROT_V: f32 = f32::PI / 2.0;
}

impl geng::AbstractCamera3d for Camera {
    fn view_matrix(&self) -> Mat4<f32> {
        Mat4::rotate_x(-self.rot_v)
            * Mat4::rotate_y(-self.rot_h)
            * Mat4::rotate_x(-f32::PI / 2.0)
            * Mat4::translate(-self.pos)
    }

    fn projection_matrix(&self, framebuffer_size: Vec2<f32>) -> Mat4<f32> {
        Mat4::perspective(
            self.fov,
            framebuffer_size.x / framebuffer_size.y,
            0.1,
            1000.0,
        )
    }
}

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
    #[asset(path = "box.png")]
    pub box_texture: ugli::Texture,
    #[asset(path = "table.obj")]
    pub obj: Obj,
    #[asset(path = "JumpScare1.wav")]
    pub jumpscare: geng::Sound,
    #[asset(path = "MainCreepyToneAmbient.wav", postprocess = "loop_sound")]
    pub music: geng::Sound,
    pub level: LevelData,
}

pub struct Wall {
    pub a: Vec2<f32>,
    pub b: Vec2<f32>,
}

struct Door {
    obj: Obj,
    pivot: Vec3<f32>,
}

pub struct LevelData {
    obj: Obj,
    doors: Vec<Door>,
    spawn_point: Vec3<f32>,
}

impl geng::LoadAsset for LevelData {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        async move {
            let mut obj = <Obj as geng::LoadAsset>::load(&geng, &path.join("roomMVP.obj")).await?;
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
                doors: {
                    let mut doors = Vec::new();
                    for i in (0..obj.meshes.len()).rev() {
                        if obj.meshes[i].name.starts_with("D_") {
                            let mesh = obj.meshes.remove(i);
                            let pivot = mesh.geometry[2].a_v;
                            doors.push(Door {
                                obj: Obj { meshes: vec![mesh] },
                                pivot,
                            });
                        }
                    }
                    doors
                },
                obj,
            })
        }
        .boxed_local()
    }

    const DEFAULT_EXT: Option<&'static str> = None;
}

struct Player {
    pos: Vec3<f32>,
    vel: Vec3<f32>,
}

struct DoorState {
    open: bool,
    rot: f32,
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera,
    sens: f32,
    white_texture: ugli::Texture,
    player: Player,
    waypoints: Vec<Vec3<f32>>,
    doors: Vec<DoorState>,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        geng.window().lock_cursor();
        let mut music = assets.music.effect();
        music.set_volume(0.5);
        music.play();
        let waypoints = {
            let obj = &assets.level.obj;
            let mut points = Vec::new();
            const HOR_GRID_SIZE: usize = 20;
            const VER_GRID_SIZE: usize = 5;
            let hor_range = -10.0..10.0;
            let ver_range = 0.0..1.0;
            for x in 0..=HOR_GRID_SIZE {
                let x = hor_range.start
                    + (hor_range.end - hor_range.start) * x as f32 / HOR_GRID_SIZE as f32;
                for y in 0..=HOR_GRID_SIZE {
                    let y = hor_range.start
                        + (hor_range.end - hor_range.start) * y as f32 / HOR_GRID_SIZE as f32;
                    for z in 0..=VER_GRID_SIZE {
                        let z = ver_range.start
                            + (ver_range.end - ver_range.start) * z as f32 / VER_GRID_SIZE as f32;
                        let p = vec3(x, y, z);
                        if let Some(t) = intersect_ray_with_obj(
                            obj,
                            geng::CameraRay {
                                from: p,
                                dir: vec3(0.0, 0.0, -1.0),
                            },
                        ) {
                            if t > 0.1 {
                                continue;
                            }
                        }
                        points.push(p);
                    }
                }
            }
            points
        };
        Self {
            doors: assets
                .level
                .doors
                .iter()
                .map(|_| DoorState {
                    open: false,
                    rot: 0.0,
                })
                .collect(),
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            player: Player {
                pos: assets.level.spawn_point,
                vel: Vec3::ZERO,
            },
            camera: Camera {
                pos: assets.level.spawn_point,
                fov: f32::PI / 2.0,
                rot_h: 0.0,
                rot_v: 0.0,
            },
            sens: 0.001,
            white_texture: ugli::Texture::new_with(geng.ugli(), vec2(1, 1), |_| Rgba::WHITE),
            waypoints,
        }
    }

    fn draw_texture(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        program: &ugli::Program,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        let size = vec2(
            size * texture.size().x as f32 / texture.size().y as f32,
            size,
        );
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: Vec2<f32>,
        }
        ugli::draw(
            framebuffer,
            program,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
                vec![
                    Vertex {
                        a_pos: vec2(0.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    Vertex {
                        a_pos: vec2(0.0, 1.0),
                    },
                ]
            }),
            (
                ugli::uniforms! {
                    u_pos: pos,
                    u_size: size,
                    u_rot: rot,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
    }

    fn draw_billboard(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.billboard,
            texture,
            pos,
            size,
            rot,
        );
    }

    fn draw_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.sprite,
            texture,
            pos,
            size,
            rot,
        );
    }

    fn draw_horizontal_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.horizontal_sprite,
            texture,
            pos,
            size,
            rot,
        );
    }

    fn draw_vertical_sprite(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.vertical_sprite,
            texture,
            pos,
            size,
            rot,
        );
    }

    fn draw_obj(&self, framebuffer: &mut ugli::Framebuffer, obj: &Obj, matrix: Mat4<f32>) {
        for mesh in &obj.meshes {
            let mut matrix = matrix;
            if mesh.name == "PlayerSpawn" {
                continue;
            }
            if mesh.name.starts_with("B_") {
                // TODO: only once
                let mut sum = Vec3::ZERO;
                for v in &*mesh.geometry {
                    sum += v.a_v;
                }
                let center = sum / mesh.geometry.len() as f32;
                matrix = matrix
                    * Mat4::translate(center)
                    * Mat4::rotate_z(self.camera.rot_h)
                    * Mat4::translate(-center);
            }
            ugli::draw(
                framebuffer,
                &self.assets.shaders.obj,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_model_matrix: matrix,
                        u_texture: mesh.material.texture.as_deref().unwrap_or(&self.white_texture),
                    },
                    geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                ),
                ugli::DrawParameters {
                    blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::Less),
                    ..default()
                },
            );
        }
    }
}

impl geng::State for Game {
    fn update(&mut self, delta_time: f64) {
        let walk_speed = 3.0;
        self.geng
            .audio()
            .set_listener_position(self.camera.pos.map(|x| x as f64));
        self.geng.audio().set_listener_orientation(
            { Mat4::rotate_z(self.camera.rot_h) * vec4(0.0, 1.0, 0.0, 1.0) }
                .xyz()
                .map(|x| x as f64),
            vec3(0.0, 0.0, 1.0),
        );
        let delta_time = delta_time as f32;
        let delta_time = delta_time.min(1.0 / 30.0);
        let mut mov = vec2(0.0, 0.0);
        if self.geng.window().is_key_pressed(geng::Key::W)
            || self.geng.window().is_key_pressed(geng::Key::Up)
        {
            mov.y += 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::A)
            || self.geng.window().is_key_pressed(geng::Key::Left)
        {
            mov.x -= 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::S)
            || self.geng.window().is_key_pressed(geng::Key::Down)
        {
            mov.y -= 1.0;
        }
        if self.geng.window().is_key_pressed(geng::Key::D)
            || self.geng.window().is_key_pressed(geng::Key::Right)
        {
            mov.x += 1.0;
        }
        let mov = mov.clamp_len(..=1.0);
        let target_vel = mov.rotate(self.camera.rot_h) * walk_speed;
        let accel = 50.0;
        self.player.vel += (target_vel - self.player.vel.xy())
            .clamp_len(..=accel * delta_time)
            .extend(0.0);

        // if self.geng.window().is_key_pressed(geng::Key::Space) {
        //     self.player.pos.z += delta_time * walk_speed;
        // }
        // if self.geng.window().is_key_pressed(geng::Key::LCtrl) {
        let gravity = 5.0;
        self.player.vel.z -= gravity * delta_time;
        // }

        self.player.pos += self.player.vel * delta_time;

        for _ in 0..3 {
            let v = vector_from_obj(&self.assets.level.obj, self.player.pos);
            let radius = 0.25;
            if v.len() < radius {
                let n = v.normalize_or_zero();
                self.player.vel -= n * Vec3::dot(n, self.player.vel);
                self.player.pos += v.normalize_or_zero() * (radius - v.len());
            }
        }

        for door_state in &mut self.doors {
            if door_state.open {
                door_state.rot += delta_time;
            } else {
                door_state.rot -= delta_time;
            }
            door_state.rot = door_state.rot.clamp(0.0, f32::PI / 2.0);
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

        self.camera.pos = self.player.pos + vec3(0.0, 0.0, 1.0);
        self.draw_obj(framebuffer, &self.assets.level.obj, Mat4::identity());
        for (door_data, door_state) in izip![&self.assets.level.doors, &self.doors] {
            self.draw_obj(
                framebuffer,
                &door_data.obj,
                Mat4::translate(door_data.pivot)
                    * Mat4::rotate_z(door_state.rot)
                    * Mat4::translate(-door_data.pivot),
            );
        }

        let mut ray = self
            .camera
            .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
        ray.dir = ray.dir.normalize_or_zero();
        if let Some(t) = intersect_ray_with_obj(&self.assets.level.obj, ray) {
            // self.draw_sprite(
            //     framebuffer,
            //     &self.assets.key,
            //     ray.from + ray.dir * (t - 0.05),
            //     0.05,
            //     0.0,
            // );
        }
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { .. } => {
                self.geng.window().lock_cursor();

                let mut ray = self
                    .camera
                    .pixel_ray(self.framebuffer_size, self.framebuffer_size / 2.0);
                ray.dir = ray.dir.normalize_or_zero();
                let max_t = intersect_ray_with_obj(&self.assets.level.obj, ray).unwrap_or(1e9);
                for (door_data, door_state) in izip![&self.assets.level.doors, &mut self.doors] {
                    if let Some(t) = intersect_ray_with_obj(&door_data.obj, ray) {
                        if t < max_t {
                            door_state.open = !door_state.open;
                        }
                    }
                }
            }
            geng::Event::MouseMove { position, delta } => {
                let delta = delta.map(|x| x as f32);
                self.camera.rot_h -= delta.x * self.sens;
                self.camera.rot_v = (self.camera.rot_v + delta.y * self.sens)
                    .clamp(Camera::MIN_ROT_V, Camera::MAX_ROT_V);
            }
            geng::Event::KeyDown { key: geng::Key::J } => {
                self.assets.jumpscare.play();
            }
            _ => {}
        }
    }
}

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();

    let geng = Geng::new("FlashDark");
    geng::run(
        &geng,
        geng::LoadingScreen::new(
            &geng,
            geng::EmptyLoadingScreen,
            <Assets as geng::LoadAsset>::load(&geng, &static_path()),
            {
                let geng = geng.clone();
                move |assets| Game::new(&geng, &Rc::new(assets.unwrap()))
            },
        ),
    );
}
