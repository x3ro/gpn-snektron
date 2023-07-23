mod socketio;
mod data;
mod foo_material;

// // Entry point for wasm
// #[cfg(target_arch = "wasm32")]
// use wasm_bindgen::prelude::*;
//
// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen(start)]
// pub fn start() -> Result<(), JsValue> {
//     console_log::init_with_level(log::Level::Debug).unwrap();
//
//     use log::info;
//     info!("Logging works!");
//
//     std::panic::set_hook(Box::new(console_error_panic_hook::hook));
//     main();
//     Ok(())
// }

use std::sync::{Arc, Mutex};
use colorsys::{ColorTransform, Hsl, Rgb};
use three_d::*;
use serde::Deserialize;
use itertools::Itertools;
use three_d_asset::io::RawAssets;

use crate::data::{ArcGameState, Position, GameState};
use crate::socketio::client_thread;

struct SnekMaterial {}

impl Material for SnekMaterial {
    fn fragment_shader_source(&self, _lights: &[&dyn Light]) -> String {
        include_str!("snek.frag").to_string()
    }

    fn fragment_attributes(&self) -> FragmentAttributes {
        FragmentAttributes {
            position: true,
            uv: true,
            ..FragmentAttributes::NONE
        }
    }

    fn use_uniforms(&self, _program: &Program, _camera: &Camera, _lights: &[&dyn Light]) {}
    fn render_states(&self) -> RenderStates {
        RenderStates {
            cull: Cull::Back,
            blend: Blend::TRANSPARENCY,
            depth_test: DepthTest::Always,
            write_mask: WriteMask::COLOR_AND_DEPTH,
        }
    }

    fn material_type(&self) -> MaterialType {
        MaterialType::Transparent
    }

    fn id(&self) -> u16 {
        0b11u16
    }
}

struct Drawing {
    context: Context,
    scale_factor: f32,
    width: f32,
    height: f32,
    field_size_in_tiles: usize,
    //snake_texture: CpuTexture,
    textures: Textures,
}

const GRID_MARGIN: f32 = 20.0;
const GRID_LINE_THICKNESS: f32 = 2.0;

trait RgbExt {
    fn to_color(&self) -> Color;
}

impl RgbExt for Rgb {
    fn to_color(&self) -> Color {
        Color::new_opaque(
            self.red() as u8,
            self.green() as u8,
            self.blue() as u8
        )
    }
}

impl Drawing {
    fn field_size_screen(&self) -> f32 {
        if self.width > self.height {
            self.height - GRID_MARGIN * 2.0
        } else {
            self.width - GRID_MARGIN * 2.0
        }
    }

    fn grid_size_screen(&self) -> f32 {
        self.field_size_screen() / (self.field_size_in_tiles as f32)
    }

    fn pos(&self, x: f32, y: f32) -> Vector2<f32> {

        let x = GRID_MARGIN + (x * self.grid_size_screen()) as f32;
        let y = GRID_MARGIN + ((self.field_size_in_tiles as f32 - y) * self.grid_size_screen()) as f32;
        vec2(x, y) * self.scale_factor
    }

    fn draw_snek(&self, mut positions: Vec<Position>, base_color: Color) -> Vec<Gm<Rectangle, ColorMaterial>> {
        positions.reverse();

        assert!(positions.len() > 0);
        let radius = self.grid_size_screen() * 0.8;

        let mut c = Rgb::new(base_color.r as f64, base_color.g as f64, base_color.b as f64, None);

        // let cpu_texture: CpuTexture = self.assets.deserialize("snake-graphics").unwrap();
        // let mut texture = Texture2DRef::from_cpu_texture(
        //     &self.context,
        //     &self.snake_texture.to_linear_srgb().unwrap(),
        // );
        // texture.transformation =
        //     Matrix3::from_translation(vec2(0.0, 0.8)) *
        //         Matrix3::from_scale(0.2);

        let mut res = vec![];
        for (pos, next_pos) in positions.iter().tuple_windows() {
            // res.push(Gm::new(
            //     Circle::new(
            //         &self.context,
            //         self.pos(pos.x as f32 + 0.5, pos.y as f32 + 0.5),
            //         radius,
            //     ),
            //     SnekMaterial {},
            // ));

            let geometry = Rectangle::new(
                &self.context,
                self.pos(pos.x as f32 + 0.5, pos.y as f32 + 0.5),
                Rad(0.0),
                2.0*radius,
                2.0*radius,
            );

            res.push(Gm::new(
                geometry,
                self.textures.head_left(),
                // ColorMaterial {
                //     texture: Some(texture.clone()),
                //     is_transparent: true,
                //     render_states: RenderStates {
                //         blend: Blend::TRANSPARENCY,
                //         ..Default::default()
                //     },
                //     ..Default::default()
                // },
            ));

            let diff_x = (pos.x as f32 - next_pos.x as f32) / 2.0;
            let diff_y = (pos.y as f32 - next_pos.y as f32) / 2.0;

            let hsl: Hsl = c.clone().into();
            if hsl.lightness() < 70.0 {
                c.lighten(2.0);
            }


            // let hsl: Hsl = c.clone().into();
            // println!("lightness: {}", hsl.lightness());


            // if diff_x != 0.0 && diff_x.abs() <= 1.0 {
            //     res.push(Gm::new(
            //         Circle::new(
            //             &self.context,
            //             self.pos(pos.x as f32  + 0.5 - diff_x, pos.y as f32 + 0.5),
            //             radius,
            //         ),
            //         SnekMaterial {},
            //     ));
            // }
            //
            // if diff_y != 0.0 && diff_y.abs() <= 1.0 {
            //     res.push(Gm::new(
            //         Circle::new(
            //             &self.context,
            //             self.pos(pos.x as f32  + 0.5, pos.y as f32 + 0.5 - diff_y),
            //             radius,
            //         ),
            //         SnekMaterial {},
            //     ));
            // }
        }

        // let last = positions.last().unwrap();
        // res.push(Gm::new(
        //     Circle::new(
        //         &self.context,
        //         self.pos(last.x as f32 + 0.5, last.y as f32 + 0.5),
        //         radius,
        //     ),
        //     SnekMaterial {},
        // ));

        res
    }

    fn draw_grid(&self, size_in_tiles: usize) -> Vec<Gm<Line, ColorMaterial>> {
        let color = Color::BLACK;

        let mut lines = vec![];

        for i in 0..(size_in_tiles + 1) {
            let vertical_line = Gm::new(
                Line::new(
                    &self.context,
                    self.pos(i as f32, 0.0),
                    self.pos(i as f32, self.field_size_in_tiles as f32),
                    GRID_LINE_THICKNESS * self.scale_factor,
                ),
                ColorMaterial {
                    color,
                    ..Default::default()
                },
            );

            let horizontal_line = Gm::new(
                Line::new(
                    &self.context,
                    self.pos(0.0, i as f32),
                    self.pos(self.field_size_in_tiles as f32, i as f32),
                    GRID_LINE_THICKNESS * self.scale_factor,
                ),
                ColorMaterial {
                    color,
                    ..Default::default()
                },
            );

            lines.push(vertical_line);
            lines.push(horizontal_line);
        }

        lines
    }
}

struct Textures {
    head_left: Texture2DRef,
}

impl Textures {
    pub fn new(context: &Context, atlas: &CpuTexture) -> Self {
        let mut texture = Texture2DRef::from_cpu_texture(
            context,
            &atlas.to_linear_srgb().unwrap(),
        );

        let mut head_left = texture.clone();
        head_left.transformation =
            Matrix3::from_translation(vec2(0.6, 0.6)) *
                Matrix3::from_scale(0.2);

        Textures {
            head_left,
        }
    }

    pub fn head_left(&self) -> ColorMaterial {
        ColorMaterial {
            texture: Some(self.head_left.clone()),
            is_transparent: true,
            render_states: RenderStates {
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes 2D!".to_string(),
        ..Default::default()
    })
        .unwrap();

    let state = GameState {
        version: 1,
        width: 9,
        height: 9,
        id: "foobar".to_string(),
        players: vec![],
    };

    let context = window.gl();
    let scale_factor = window.device_pixel_ratio();
    let (width, height) = window.size();

    let mut loaded = three_d_asset::io::load(&[
        "assets/snake-graphics.png",
        "assets/uvchecker.png",
    ])
        .unwrap();

    let cpu_texture: CpuTexture = loaded.deserialize("snake-graphics").unwrap();
    let mut texture = Texture2DRef::from_cpu_texture(
        &context,
        &cpu_texture.to_linear_srgb().unwrap(),
    );
    texture.transformation =
        Matrix3::from_translation(vec2(0.0, 0.8)) *
        Matrix3::from_scale(0.2);





    let mut obj = Gm::new(
        Rectangle::new(
            &context,
            ((width as f32/2.0) * scale_factor, (height as f32/2.0) * scale_factor),
            Rad(0.0),
            1000.0,
            1000.0,
        ),
        ColorMaterial {
            //color: Color::BLACK,
            texture: Some(texture),
            is_transparent: true,
            ..Default::default()
        }
    );
    //obj.material.render_states.cull = Cull::Back;
    // let foo = &obj.geometry;
    // obj.material.render_states.blend = Blend::TRANSPARENCY;


    let textures = Textures::new(&context, &cpu_texture);

    let d = Drawing {
        context,
        scale_factor,
        width: width as f32,
        height: height as f32,
        field_size_in_tiles: state.width,
        //snake_texture: cpu_texture,
        textures,
    };

    let grid_lines = d.draw_grid(state.width);

    let game_state: ArcGameState = Arc::new(Mutex::new(GameState::default()));
    client_thread(game_state.clone());

    let colors = vec![
        Color::BLUE,
        Color::RED,
        Color::GREEN,
    ];





    window.render_loop(move |frame_input| {
        // for event in frame_input.events.iter() {
        //     if let Event::MousePress {
        //         button,
        //         position,
        //         modifiers,
        //         ..
        //     } = event
        //     {
        //         if *button == MouseButton::Left && !modifiers.ctrl {
        //             rectangle.set_center(position);
        //         }
        //         if *button == MouseButton::Right && !modifiers.ctrl {
        //             circle.set_center(position);
        //         }
        //         if *button == MouseButton::Left && modifiers.ctrl {
        //             let ep = line.end_point1();
        //             line.set_endpoints(position, ep);
        //         }
        //         if *button == MouseButton::Right && modifiers.ctrl {
        //             let ep = line.end_point0();
        //             line.set_endpoints(ep, position);
        //         }
        //     }
        // }
        // let objects: Vec<_> = grid_lines.iter().map(|x| x.into_iter()).collect();
        // let mut iter = vec![].iter();
        // for foo in grid_lines {
        //     iter = iter.chain(&foo);
        // }


        let mut snek = vec![];

        let state = game_state.lock().unwrap();
        for (idx, player) in state.players.iter().enumerate() {
            if !player.alive {
                continue
            }

            let mut res = d.draw_snek(
                player.moves.clone(),
                colors.get(idx).unwrap().clone(),
            );
            snek.append(&mut res);
        }

        let sneks = snek.iter()
            .flat_map(|m| m.into_iter());

        let grids = grid_lines
            .iter()
            .flat_map(|m| m.into_iter());

        //let objects = sneks.chain(grids);

        frame_input
            .screen()
            .render(&camera2d(frame_input.viewport),
                    grids,
                    &[],);

        frame_input
            .screen()
            // Solarized-light background color
            .clear(ClearState::color_and_depth(0.99, 0.96, 0.89, 1.0, 1.0))
            //.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                sneks,
                &[],
            );



        FrameOutput::default()
    });
}
