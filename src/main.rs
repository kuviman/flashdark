use geng::prelude::*;

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
        Mat4::rotate_x(-self.rot_v) * Mat4::rotate_y(-self.rot_h) * Mat4::translate(-self.pos)
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
}

#[derive(geng::Assets)]
pub struct Assets {
    pub shaders: Shaders,
}

pub struct Game {
    framebuffer_size: Vec2<f32>,
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera,
    sens: f32,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        geng.window().lock_cursor();
        Self {
            framebuffer_size: vec2(1.0, 1.0),
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera {
                pos: vec3(0.0, 0.0, 1.0),
                fov: f32::PI / 2.0,
                rot_h: 0.0,
                rot_v: 0.0,
            },
            sens: 0.001,
        }
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        self.framebuffer_size = framebuffer.size().map(|x| x as f32);
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);

        #[derive(ugli::Vertex)]
        pub struct Vertex {
            pub a_pos: Vec3<f32>,
        }

        ugli::draw(
            framebuffer,
            &self.assets.shaders.wall,
            ugli::DrawMode::TriangleFan,
            &ugli::VertexBuffer::new_dynamic(
                self.geng.ugli(),
                vec![
                    Vertex {
                        a_pos: vec3(0.0, 0.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec3(1.0, 0.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec3(1.0, 1.0, 0.0),
                    },
                    Vertex {
                        a_pos: vec3(0.0, 1.0, 0.0),
                    },
                ],
            ),
            (
                ugli::uniforms! {},
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters { ..default() },
        );
    }

    fn handle_event(&mut self, event: geng::Event) {
        match event {
            geng::Event::MouseDown { .. } => {
                self.geng.window().lock_cursor();
            }
            geng::Event::MouseMove { position, delta } => {
                let delta = dbg!(delta.map(|x| x as f32));
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
