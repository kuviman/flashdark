use super::*;

pub struct LoadingScreen {
    geng: Geng,
    time: f32,
}

impl LoadingScreen {
    pub fn new(geng: &Geng) -> Self {
        Self {
            geng: geng.clone(),
            time: 0.0,
        }
    }
}

impl geng::ProgressScreen for LoadingScreen {
    fn update_progress(&mut self, progress: f64) {
        #![allow(unused_variables)]
    }
}

impl geng::State for LoadingScreen {
    fn update(&mut self, delta_time: f64) {
        self.time += delta_time as f32;
    }
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::BLACK), None, None);
        self.geng.draw_2d(
            framebuffer,
            &geng::Camera2d {
                center: Vec2::ZERO,
                rotation: self.time.sin() * 0.1,
                fov: 10.0,
            },
            &draw_2d::Text::unit(&**self.geng.default_font(), "Loading...", Rgba::WHITE),
        )
    }
}
