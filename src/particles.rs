use super::*;

#[derive(ugli::Vertex)]
pub struct Particle {
    i_pos: Vec3<f32>,
    i_vel: Vec3<f32>,
    i_life: f32,
    i_size: f32,
}

pub struct Particles {
    next_particle: f32,
    instances: ugli::VertexBuffer<Particle>,
}

impl Particles {
    pub fn new(geng: &Geng) -> Self {
        Self {
            next_particle: 0.0,
            instances: ugli::VertexBuffer::new_dynamic(geng.ugli(), vec![]),
        }
    }
}

impl Game {
    pub fn update_particles(&mut self, delta_time: f32) {
        self.particles.next_particle -= delta_time;
        while self.particles.next_particle < 0.0 {
            self.particles.next_particle += 1.0 / 100.0;
            let range = 10.0;
            self.particles.instances.push(Particle {
                i_pos: (self.camera.pos.xy()
                    + vec2(
                        global_rng().gen_range(-range..range),
                        global_rng().gen_range(-range..range),
                    ))
                .extend(global_rng().gen_range(0.1..1.9)),
                i_vel: vec3(
                    global_rng().gen_range(-1.0..1.0),
                    global_rng().gen_range(-1.0..1.0),
                    global_rng().gen_range(-1.0..1.0),
                ) * 0.05,
                i_life: 1.0,
                i_size: global_rng().gen_range(0.01..0.025),
            });
        }
        for p in self.particles.instances.iter_mut() {
            p.i_pos += p.i_vel * delta_time;
            p.i_life -= delta_time / 5.0;
        }
        self.particles
            .instances
            .retain(|particle| particle.i_life > 0.0);
    }
    pub fn draw_particles(&self, framebuffer: &mut ugli::Framebuffer) {
        ugli::draw(
            framebuffer,
            &self.assets.shaders.particle,
            ugli::DrawMode::TriangleFan,
            ugli::instanced(&self.quad_geometry, &self.particles.instances),
            (
                ugli::uniforms! {
                    u_color: Rgba::WHITE,
                    u_flashdark_pos: self.player.flashdark.pos,
                    u_flashdark_dir: self.player.flashdark.dir,
                    u_flashdark_angle: f32::PI / 4.0,
                    u_flashdark_strength: self.player.flashdark.strength,
                    u_flashdark_dark: self.player.flashdark.dark,
                    u_ambient_light_color: self.ambient_light,
                    u_noise: &self.noise,
                    u_texture: &self.assets.dust_particle,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                self.light_uniforms(),
            ),
            ugli::DrawParameters {
                depth_func: Some(default()),
                blend_mode: Some(ugli::BlendMode::default()),
                ..default()
            },
        );
    }
}
