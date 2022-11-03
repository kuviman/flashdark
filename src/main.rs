use geng::prelude::*;

mod obj;

use obj::*;

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
}

pub struct Wall {
    pub a: Vec2<f32>,
    pub b: Vec2<f32>,
}

pub struct Level {
    walls: Vec<Wall>,
}

#[derive(ugli::Vertex, Copy, Clone)]
pub struct WallVertex {
    pub a_pos: Vec3<f32>,
    pub a_uv: Vec2<f32>,
}

pub struct LevelMesh {
    walls: ugli::VertexBuffer<WallVertex>,
}

impl LevelMesh {
    pub fn new(geng: &Geng, level: &Level) -> Self {
        Self {
            walls: ugli::VertexBuffer::new_static(
                geng.ugli(),
                level
                    .walls
                    .iter()
                    .flat_map(|wall| {
                        let len = (wall.b - wall.a).len();
                        let quad = [
                            WallVertex {
                                a_pos: wall.a.extend(0.0),
                                a_uv: vec2(0.0, 0.0),
                            },
                            WallVertex {
                                a_pos: wall.b.extend(0.0),
                                a_uv: vec2(len, 0.0),
                            },
                            WallVertex {
                                a_pos: wall.b.extend(1.0),
                                a_uv: vec2(len, 1.0),
                            },
                            WallVertex {
                                a_pos: wall.a.extend(1.0),
                                a_uv: vec2(0.0, 1.0),
                            },
                        ];
                        [quad[0], quad[1], quad[2], quad[0], quad[2], quad[3]]
                    })
                    .collect(),
            ),
        }
    }
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera,
    sens: f32,
    level: Level,
    level_mesh: LevelMesh,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        geng.window().lock_cursor();
        let level = Level {
            walls: vec![
                Wall {
                    a: vec2(-1.0, -1.0),
                    b: vec2(-0.2, -1.0),
                },
                Wall {
                    a: vec2(0.2, -1.0),
                    b: vec2(1.0, -1.0),
                },
                Wall {
                    a: vec2(1.0, -1.0),
                    b: vec2(1.0, 1.0),
                },
                Wall {
                    a: vec2(1.0, 1.0),
                    b: vec2(-1.5, 1.0),
                },
                Wall {
                    a: vec2(-1.5, 1.0),
                    b: vec2(-1.0, -1.0),
                },
                Wall {
                    a: vec2(-1.0, -2.0),
                    b: vec2(1.0, -2.0),
                },
            ],
        };
        let level_mesh = LevelMesh::new(geng, &level);
        Self {
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera {
                pos: vec3(0.0, 0.0, 0.5),
                fov: f32::PI / 2.0,
                rot_h: 0.0,
                rot_v: 0.0,
            },
            sens: 0.001,
            level,
            level_mesh,
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
                        u_texture: mesh.material.texture.as_deref().unwrap_or(&self.assets.box_texture),
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
        let delta_time = delta_time as f32;
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
        self.camera.pos += mov.rotate(self.camera.rot_h).extend(0.0) * delta_time;
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), Some(1.0), None);

        ugli::draw(
            framebuffer,
            &self.assets.shaders.wall,
            ugli::DrawMode::Triangles,
            &self.level_mesh.walls,
            (
                ugli::uniforms! {
                    u_texture: &self.assets.wall,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
        // floor
        ugli::draw(
            framebuffer,
            &self.assets.shaders.wall,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
                let v = |x: f32, y: f32| -> WallVertex {
                    let p = vec2(x, y) * 100.0 + self.camera.pos.xy();
                    WallVertex {
                        a_pos: p.extend(0.0),
                        a_uv: p,
                    }
                };
                vec![v(-1.0, -1.0), v(1.0, -1.0), v(1.0, 1.0), v(-1.0, 1.0)]
            }),
            (
                ugli::uniforms! {
                    u_texture: &self.assets.floor,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );
        // ceiling
        ugli::draw(
            framebuffer,
            &self.assets.shaders.wall,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(self.geng.ugli(), {
                let v = |x: f32, y: f32| -> WallVertex {
                    let p = vec2(x, y) * 100.0 + self.camera.pos.xy();
                    WallVertex {
                        a_pos: p.extend(1.0),
                        a_uv: p,
                    }
                };
                vec![v(-1.0, -1.0), v(1.0, -1.0), v(1.0, 1.0), v(-1.0, 1.0)]
            }),
            (
                ugli::uniforms! {
                    u_texture: &self.assets.ceiling,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                depth_func: Some(ugli::DepthFunc::Less),
                ..default()
            },
        );

        self.draw_billboard(
            framebuffer,
            &self.assets.ghost,
            vec3(-0.7, 0.0, 0.0),
            0.7,
            0.0,
        );
        self.draw_horizontal_sprite(
            framebuffer,
            &self.assets.table_top,
            vec3(0.0, 0.0, 0.3),
            0.3,
            0.0,
        );
        self.draw_billboard(
            framebuffer,
            &self.assets.table_leg,
            vec3(-0.1, -0.1, 0.0),
            0.3,
            0.0,
        );
        self.draw_billboard(
            framebuffer,
            &self.assets.table_leg,
            vec3(0.1, -0.1, 0.0),
            0.3,
            0.0,
        );
        self.draw_billboard(
            framebuffer,
            &self.assets.table_leg,
            vec3(0.1, 0.1, 0.0),
            0.3,
            0.0,
        );
        self.draw_billboard(
            framebuffer,
            &self.assets.table_leg,
            vec3(-0.1, 0.1, 0.0),
            0.3,
            0.0,
        );

        self.draw_sprite(
            framebuffer,
            &self.assets.key,
            vec3(0.0, 0.0, 0.35),
            0.05,
            0.0,
        );

        let bed_pos = vec2(0.5, 0.0);
        let bed_rot = 0.3;
        let bed_size = 0.4;
        self.draw_vertical_sprite(
            framebuffer,
            &self.assets.bed_back,
            (bed_pos + vec2(0.0, -bed_size).rotate(bed_rot)).extend(0.0),
            bed_size,
            bed_rot,
        );
        self.draw_vertical_sprite(
            framebuffer,
            &self.assets.bed_back,
            (bed_pos + vec2(0.0, bed_size).rotate(bed_rot)).extend(0.0),
            bed_size,
            bed_rot,
        );
        self.draw_horizontal_sprite(
            framebuffer,
            &self.assets.bed_bottom,
            bed_pos.extend(bed_size * 0.3),
            bed_size,
            bed_rot + f32::PI / 2.0,
        );

        let box_pos = vec2(0.0, 0.7);
        let box_size = 0.3;
        let box_rot = 0.7;
        self.draw_horizontal_sprite(
            framebuffer,
            &self.assets.box_texture,
            box_pos.extend(box_size),
            box_size,
            box_rot,
        );
        self.draw_vertical_sprite(
            framebuffer,
            &self.assets.box_texture,
            (box_pos + vec2(0.0, box_size / 2.0).rotate(box_rot)).extend(0.0),
            box_size,
            box_rot,
        );
        self.draw_vertical_sprite(
            framebuffer,
            &self.assets.box_texture,
            (box_pos + vec2(0.0, -box_size / 2.0).rotate(box_rot)).extend(0.0),
            box_size,
            box_rot,
        );
        self.draw_vertical_sprite(
            framebuffer,
            &self.assets.box_texture,
            (box_pos + vec2(box_size / 2.0, 0.0).rotate(box_rot)).extend(0.0),
            box_size,
            box_rot + f32::PI / 2.0,
        );
        self.draw_vertical_sprite(
            framebuffer,
            &self.assets.box_texture,
            (box_pos + vec2(-box_size / 2.0, 0.0).rotate(box_rot)).extend(0.0),
            box_size,
            box_rot + f32::PI / 2.0,
        );

        self.draw_obj(
            framebuffer,
            &self.assets.obj,
            Mat4::translate(vec3(0.0, 0.0, 0.5)) * Mat4::scale_uniform(0.1),
        );
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { .. } => {
                self.geng.window().lock_cursor();
            }
            geng::Event::MouseMove { position, delta } => {
                let delta = delta.map(|x| x as f32);
                self.camera.rot_h -= delta.x * self.sens;
                self.camera.rot_v = (self.camera.rot_v + delta.y * self.sens)
                    .clamp(Camera::MIN_ROT_V, Camera::MAX_ROT_V);
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
