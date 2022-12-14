use super::*;

impl Game {
    pub fn draw_texture(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        program: &ugli::Program,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
        dissolve: f32,
        color: Rgba<f32>,
    ) {
        let size = vec2(
            size * texture.size().x as f32 / texture.size().y as f32,
            size,
        );
        #[derive(ugli::Vertex)]
        struct Vertex {
            a_pos: Vec2<f32>,
        }
        self.draw_calls.set(self.draw_calls.get() + 1);
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
                    u_noise: &self.noise,
                    u_texture: texture,
                    u_dissolve: dissolve,
                    u_color: color,
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

    pub fn draw_billboard(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        texture: &ugli::Texture,
        pos: Vec3<f32>,
        size: f32,
        rot: f32,
        dissolve: f32,
    ) {
        self.draw_texture(
            framebuffer,
            &self.assets.shaders.billboard,
            texture,
            pos,
            size,
            rot,
            dissolve,
            Rgba::WHITE,
        );
    }

    // pub fn draw_sprite(
    //     &self,
    //     framebuffer: &mut ugli::Framebuffer,
    //     texture: &ugli::Texture,
    //     pos: Vec3<f32>,
    //     size: f32,
    //     rot: f32,
    // ) {
    //     self.draw_texture(
    //         framebuffer,
    //         &self.assets.shaders.sprite,
    //         texture,
    //         pos,
    //         size,
    //         rot,
    //     );
    // }

    // pub fn draw_horizontal_sprite(
    //     &self,
    //     framebuffer: &mut ugli::Framebuffer,
    //     texture: &ugli::Texture,
    //     pos: Vec3<f32>,
    //     size: f32,
    //     rot: f32,
    // ) {
    //     self.draw_texture(
    //         framebuffer,
    //         &self.assets.shaders.horizontal_sprite,
    //         texture,
    //         pos,
    //         size,
    //         rot,
    //     );
    // }

    // pub fn draw_vertical_sprite(
    //     &self,
    //     framebuffer: &mut ugli::Framebuffer,
    //     texture: &ugli::Texture,
    //     pos: Vec3<f32>,
    //     size: f32,
    //     rot: f32,
    // ) {
    //     self.draw_texture(
    //         framebuffer,
    //         &self.assets.shaders.vertical_sprite,
    //         texture,
    //         pos,
    //         size,
    //         rot,
    //     );
    // }

    pub fn draw_skybox_mesh(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        mesh: &ObjMesh,
        matrix: Mat4<f32>,
    ) {
        let texture = mesh.material.texture.as_deref().unwrap();
        self.draw_calls.set(self.draw_calls.get() + 1);
        ugli::draw(
            framebuffer,
            &self.assets.shaders.skybox,
            ugli::DrawMode::Triangles,
            &mesh.geometry,
            (
                ugli::uniforms! {
                    u_model_matrix: matrix,
                    u_texture: texture,
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
            ),
            ugli::DrawParameters {
                write_depth: false,
                ..default()
            },
        );
    }

    pub fn draw_mesh(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        mesh: &ObjMesh,
        matrix: Mat4<f32>,
        mut texture_matrix: Mat3<f32>,
        color: Rgba<f32>,
        shine: bool,
    ) {
        let mut matrix = matrix;
        if mesh.name == "PlayerSpawn" {
            return;
        }
        if mesh.name == "S_Bat" {
            if !self.bat_go || self.bat_t > 1.0 {
                return;
            }
            matrix = Mat4::translate(vec3(self.bat_t * 4.0, 0.0, self.bat_t * 0.3));
            if (self.time * 10.0) as i64 % 2 == 0 {
                texture_matrix = Mat3::translate(vec2(0.5, 0.0));
            }
        }
        if let Some(i) = mesh.name.strip_prefix("S_PianoKeys") {
            let i = i.chars().next().unwrap().to_digit(10).unwrap();
            if i != ((self.time * 4.0) as u32) % 4 {
                return;
            }
        }
        if let Some(i) = mesh.name.strip_prefix("AF_TV_Static") {
            if self.key_puzzle_state == KeyPuzzleState::Finish {
                return;
            }
            let i = i.chars().next().unwrap().to_digit(10).unwrap();
            if i != ((self.time * 10.0) as u32) % 2 + 1 {
                return;
            }
        }
        if self.fuse_spawned && mesh.name.contains("SwingingSwing") {
            let center = self.level.trigger_cubes["SwingingSwing"].center();
            matrix = matrix
                * Mat4::translate(center)
                * Mat4::rotate_x(self.time.sin() * 0.5)
                * Mat4::translate(-center);
        }
        if mesh.name == "S_GrandfatherClock" {
            let shake_t = (60.0 - self.gf_clock_timer).clamp(0.0, 1.0);
            if shake_t < 1.0 {
                let center = find_center(&mesh.geometry).xy().extend(0.0);
                matrix = Mat4::translate(center)
                    * Mat4::rotate_y((self.rng.get(10.0) * 0.5 + 0.5) * 0.02)
                    * Mat4::translate(-center)
                    * matrix;
            }
        }
        if mesh.name.starts_with("S_Piano") {
            let center = find_center(&mesh.geometry).xy().extend(0.0);
            matrix = Mat4::translate(center)
                * Mat4::rotate_x((self.rng.get(10.0) * 0.5 + 0.5) * 0.02)
                * Mat4::translate(-center)
                * matrix;
        }
        // TODO
        if false && mesh.name.starts_with("B_") {
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
        // if mesh.name.starts_with("HB_") {
        //     // TODO: only once
        //     let mut sum = Vec3::ZERO;
        //     for v in &*mesh.geometry {
        //         sum += v.a_v;
        //     }
        //     let center = sum / mesh.geometry.len() as f32;
        //     matrix = matrix
        //         * Mat4::translate(center)
        //         * Mat4::rotate_x(if center.y > self.camera.pos.y {
        //             self.camera.rot_v
        //         } else {
        //             -self.camera.rot_v
        //         })
        //         * Mat4::translate(-center);
        // }
        let texture = mesh
            .material
            .texture
            .as_deref()
            .unwrap_or(if mesh.name.ends_with("_Dark") {
                &self.transparent_black_texture
            } else {
                &self.white_texture
            });
        let lights = self.light_uniforms();

        self.draw_calls.set(self.draw_calls.get() + 1);

        let ambient_light = if mesh.name == "S_Pentagram"
            || mesh.name.starts_with("B_Candle")
            || mesh.name.starts_with("AF_TV_Static")
            || mesh.name == "I_HintKey"
            || mesh.name.contains("StudyKey")
        {
            Rgba::WHITE
        } else {
            self.ambient_light
        };

        ugli::draw(
            framebuffer,
            &self.assets.shaders.obj,
            ugli::DrawMode::Triangles,
            &mesh.geometry,
            (
                ugli::uniforms! {
                    u_flashdark_pos: self.player.flashdark.pos,
                    u_flashdark_dir: self.player.flashdark.dir,
                    u_flashdark_angle: f32::PI / 4.0,
                    u_flashdark_strength: self.player.flashdark.strength,
                    u_flashdark_dark: self.player.flashdark.dark,
                    u_ambient_light_color: ambient_light,
                    u_model_matrix: matrix,
                    u_color: color,
                    u_noise: &self.noise,
                    u_camera_rot: self.camera.rot_h,
                    u_texture: texture,
                    u_texture_matrix: texture_matrix,
                    u_dark_texture: mesh.material.dark_texture.as_deref().unwrap_or(texture),
                    u_darkness: if self.fuse_placed || self.main_menu { 1000.0 } else { -6.0 },
                    u_time: self.time,
                    u_should_shine: if shine { 1.0 } else { 0.0 },
                },
                geng::camera3d_uniforms(&self.camera, self.framebuffer_size),
                lights,
            ),
            ugli::DrawParameters {
                blend_mode: Some(ugli::BlendMode::default()),
                depth_func: Some(ugli::DepthFunc::Less),
                cull_face: None, // TODO: maybe but probably not
                reset_uniforms: false,
                ..default()
            },
        );
    }

    pub fn draw_obj(
        &self,
        framebuffer: &mut ugli::Framebuffer,
        obj: &Obj,
        matrix: Mat4<f32>,
        color: Rgba<f32>,
    ) {
        for mesh in &obj.meshes {
            self.draw_mesh(framebuffer, mesh, matrix, Mat3::identity(), color, false);
        }
    }

    pub fn obj_shadow(
        &self,
        light: &Light,
        framebuffer: &mut ugli::Framebuffer,
        obj: &Obj,
        matrix: Mat4<f32>,
        cull_face: Option<ugli::CullFace>,
    ) {
        for mesh in &obj.meshes {
            let mut matrix = matrix;
            if mesh.name == "PlayerSpawn" {
                continue;
            }
            if self.fuse_spawned && mesh.name.contains("SwingingSwing") {
                let center = self.level.trigger_cubes["SwingingSwing"].center();
                matrix = matrix
                    * Mat4::translate(center)
                    * Mat4::rotate_x(self.time.sin() * 0.5)
                    * Mat4::translate(-center);
            }
            // TODO
            if false && mesh.name.starts_with("B_") {
                // // TODO: only once
                let mut sum = Vec3::ZERO;
                for v in &*mesh.geometry {
                    sum += v.a_v;
                }
                let center = sum / mesh.geometry.len() as f32;
                matrix = matrix
                    * Mat4::translate(center)
                    * Mat4::rotate_z(self.camera.rot_h)
                    * Mat4::translate(-center);
                // continue; // Ignore billboards for lighting for now
            }
            let texture = mesh
                .material
                .texture
                .as_deref()
                .unwrap_or(&self.transparent_black_texture);
            self.draw_calls.set(self.draw_calls.get() + 1);
            ugli::draw(
                framebuffer,
                &self.assets.shaders.shadow,
                ugli::DrawMode::Triangles,
                &mesh.geometry,
                (
                    ugli::uniforms! {
                        u_model_matrix: matrix,
                        u_shadow_size: framebuffer.size(),
                        u_texture: texture,
                        u_texture_matrix: Mat3::identity(),
                        u_camera_rot: self.camera.rot_h,
                    },
                    geng::camera3d_uniforms(light, framebuffer.size().map(|x| x as f32)),
                ),
                ugli::DrawParameters {
                    // blend_mode: Some(ugli::BlendMode::default()),
                    depth_func: Some(ugli::DepthFunc::Less),
                    // cull_face,
                    reset_uniforms: false,
                    ..default()
                },
            );
        }
    }

    pub fn light_uniforms(&self) -> LightsUniform {
        let mut lights: Vec<&Light> = self.lights.iter().collect();
        lights.sort_by_key(|light| r32((light.pos - self.camera.pos).len()));
        let shadow_maps = &self.shadow_calc.as_ref().unwrap().shadow_maps;
        LightsUniform {
            u_lights: lights
                .into_iter()
                .filter(|light| {
                    if self.key_puzzle_state == KeyPuzzleState::LightOut && light.id.0 != 0 {
                        return false;
                    }
                    true
                })
                .map(|light| {
                    let shadow_map = shadow_maps.get(&light.id).unwrap();
                    LightUniform {
                        pos: light.pos,
                        matrix: light.matrix(shadow_map.size().map(|x| x as f32)),
                        intensity: light.intensity
                            * if light.flicker_time <= 0.0 {
                                1.0
                            } else {
                                self.rng.get(10.0) * 0.5 + 0.5
                            },
                        shadow_map,
                    }
                })
                .take(10) // TODO: not hardcode
                .collect(),
        }
    }
}
