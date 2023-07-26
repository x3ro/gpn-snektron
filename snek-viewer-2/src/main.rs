mod socketio;
mod data;
mod foo_material;
mod snake_textures;

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

use std::{fs, thread};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use cgmath::num_traits::Inv;
use colorsys::{ColorTransform, Hsl, Rgb};
use three_d::*;
use serde::Deserialize;
use itertools::Itertools;
use three_d_asset::io::RawAssets;

use crate::data::{ArcGameState, Position, GameState};
use crate::snake_textures::SnakeTextures;
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
    snake_textures: SnakeTextures,
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

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    pub fn from_pos(pos0: &Position, pos1: &Position) -> Self {
        let mut delta_x = (pos1.x as f32) - (pos0.x as f32);
        let mut delta_y = (pos1.y as f32) - (pos0.y as f32);

        // If delta_x or delta_y are > 1.0, that means the snake wrapped
        // around to the other end of the board.
        if delta_x.abs() > 1.0 {
            delta_x *= -1.0;
        }
        if delta_y.abs() > 1.0 {
            delta_y *= -1.0;
        }

        // We use this function to check orientation between moves.
        // Every move can only be one tile, non-diagonally. As a result,
        // one delta must be non-zero, but never both.
        assert!(!delta_x.is_zero() ^ !delta_y.is_zero());

        if delta_x > 0.0 {
            Self::Right
        } else if delta_x < 0.0 {
            Self::Left
        } else if delta_y > 0.0 {
            Self::Down
        } else {
            Self::Up
        }
    }

    pub fn to_rotation(&self) -> Deg<f32> {
        match self {
            Direction::Up => Deg(0.0),
            Direction::Right => Deg(-90.0),
            Direction::Down => Deg(180.0),
            Direction::Left => Deg(90.0),
        }
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

    fn rect(&self, width: f32, height: f32, rotation: impl Into<Radians>, pos: &Position) -> Rectangle {
        Rectangle::new(
            &self.context,
            self.pos(pos.x as f32 + 0.5, pos.y as f32 + 0.5),
            rotation,
            width,
            height,
        )
    }



    fn draw_snek(&self, mut positions: Vec<Position>, base_color: Color) -> Vec<Gm<Rectangle, ColorMaterial>> {
        assert!(positions.len() > 0);

        let snake_width = self.grid_size_screen() * 0.8 *  self.scale_factor;

        let mut c = Rgb::new(base_color.r as f64, base_color.g as f64, base_color.b as f64, None);
        c.lighten(20.0);

        // If there is only a single position, we don't know in which direction the snake is facing
        // yet. So in that case, we just draw a "this is where the snake will spawn" sprite.
        if positions.len() == 1 {
            let pos = positions.first().unwrap();
            return vec![
                Gm::new(
                    self.rect(
                        snake_width,
                        snake_width,
                        Deg(0.0),
                        pos
                    ),
                    self.snake_textures.spawn(c.to_color()),
                )
            ];
        }

        let mut res = vec![];

        let mut first = true;
        let mut tail_direction: Direction = Direction::Up;

        let mut dir1;
        let mut dir2 = Direction::Up;

        for (prev, pos, next) in positions.iter().tuple_windows() {
            dir1 = Direction::from_pos(prev, pos);
            dir2 = Direction::from_pos(pos, next);

            if first {
                tail_direction = dir1.clone();
                first = false;
            }

            use Direction::*;
            let (material, rotation) = match (&dir1, &dir2) {
                (Right, Down) => (
                    self.snake_textures.right_turn(c.to_color()),
                    Deg(-90.0),
                ),
                (Right, Up) => (
                    self.snake_textures.left_turn(c.to_color()),
                    Deg(-90.0),
                ),
                (Left, Down) => (
                    self.snake_textures.left_turn(c.to_color()),
                    Deg(90.0),
                ),
                (Down, Left) => (
                    self.snake_textures.right_turn(c.to_color()),
                    Deg(180.0),
                ),
                (Down, Right) => (
                    self.snake_textures.left_turn(c.to_color()),
                    Deg(180.0),
                ),
                (Left, Up) => (
                    self.snake_textures.right_turn(c.to_color()),
                    Deg(90.0),
                ),
                (Up, Left) => (
                    self.snake_textures.left_turn(c.to_color()),
                    Deg(0.0),
                ),
                (Up, Right) => (
                    self.snake_textures.right_turn(c.to_color()),
                    Deg(0.0),
                ),
                (_, _) => (
                    self.snake_textures.body(c.to_color()),
                    dir1.to_rotation(),
                )
            };

            res.push(Gm::new(
                self.rect(
                    snake_width,
                    snake_width,
                    rotation,
                    pos
                ),
                material,
            ));

            match (dir1, dir2) {
                (Up | Left | Down, Left) => {
                    res.push(Gm::new(
                        Rectangle::new(
                            &self.context,
                            self.pos(pos.x as f32, pos.y as f32 + 0.5),
                            Deg(90.0),
                            snake_width,
                            snake_width * 0.25,
                        ),
                        self.snake_textures.body(c.to_color()),
                    ));
                }

                (Up | Down | Right, Right) => {
                    res.push(Gm::new(
                        Rectangle::new(
                            &self.context,
                            self.pos(pos.x as f32 + 1.0, pos.y as f32 + 0.5),
                            Deg(90.0),
                            snake_width,
                            snake_width * 0.25,
                        ),
                        self.snake_textures.body(c.to_color()),
                    ));
                }

                (_, Down) => {
                    res.push(Gm::new(
                        Rectangle::new(
                            &self.context,
                            self.pos(pos.x as f32 + 0.5, pos.y as f32 + 1.0),
                            Deg(0.0),
                            snake_width,
                            snake_width * 0.25,
                        ),
                        self.snake_textures.body(c.to_color()),
                    ));
                }

                (_, Up) => {
                    res.push(Gm::new(
                        Rectangle::new(
                            &self.context,
                            self.pos(pos.x as f32 + 0.5, pos.y as f32),
                            Deg(0.0),
                            snake_width,
                            snake_width * 0.25,
                        ),
                        self.snake_textures.body(c.to_color()),
                    ));
                }

                _ => {}
            }

            // let diff_x = (pos.x as f32 - next_pos.x as f32) / 2.0;
            // let diff_y = (pos.y as f32 - next_pos.y as f32) / 2.0;

            // let hsl: Hsl = c.clone().into();
            // if hsl.lightness() < 70.0 {
            //     c.lighten(2.0);
            // }


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

        let last = positions.last().unwrap();
        res.push(Gm::new(
            self.rect(
                snake_width,
                snake_width,
                dir2.to_rotation(),
                &last
            ),
            self.snake_textures.head(c.to_color()),
        ));

        let first = positions.first().unwrap();
        res.push(Gm::new(
            self.rect(
                snake_width,
                snake_width,
                tail_direction.to_rotation(),
                &first
            ),
            self.snake_textures.tail(c.to_color()),
        ));

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

// struct Textures {
//     head_left: Texture2DRef,
// }

// impl Textures {
//     pub fn new(context: &Context, atlas: &CpuTexture) -> Self {
//         let mut texture = Texture2DRef::from_cpu_texture(
//             context,
//             &atlas.to_linear_srgb().unwrap(),
//         );
//
//         let mut head_left = texture.clone();
//         head_left.transformation =
//             Matrix3::from_translation(vec2(0.6, 0.6)) *
//                 Matrix3::from_scale(0.2);
//
//         Textures {
//             head_left,
//         }
//     }
//
//     pub fn head_left(&self) -> ColorMaterial {
//         ColorMaterial {
//             texture: Some(self.head_left.clone()),
//             is_transparent: true,
//             render_states: RenderStates {
//                 blend: Blend::TRANSPARENCY,
//                 ..Default::default()
//             },
//             ..Default::default()
//         }
//     }
// }

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

    // let mut loaded = three_d_asset::io::load(&[
    //     "assets/snake-graphics.png",
    //     "assets/uvchecker.png",
    // ])
    //     .unwrap();

    // let cpu_texture: CpuTexture = loaded.deserialize("snake-graphics").unwrap();
    // let mut texture = Texture2DRef::from_cpu_texture(
    //     &context,
    //     &cpu_texture.to_linear_srgb().unwrap(),
    // );
    // texture.transformation =
    //     Matrix3::from_translation(vec2(0.0, 0.8)) *
    //     Matrix3::from_scale(0.2);


    let snake_textures = SnakeTextures::new(&context);




    // let mut obj = Gm::new(
    //     Rectangle::new(
    //         &context,
    //         ((width as f32/2.0) * scale_factor, (height as f32/2.0) * scale_factor),
    //         Rad(0.0),
    //         1000.0,
    //         1000.0,
    //     ),
    //     ColorMaterial {
    //         //color: Color::BLACK,
    //         texture: Some(texture),
    //         is_transparent: true,
    //         ..Default::default()
    //     }
    // );
    //obj.material.render_states.cull = Cull::Back;
    // let foo = &obj.geometry;
    // obj.material.render_states.blend = Blend::TRANSPARENCY;


    //let textures = Textures::new(&context, &cpu_texture);

    let d = Drawing {
        context,
        scale_factor,
        width: width as f32,
        height: height as f32,
        field_size_in_tiles: state.width,
        //snake_texture: cpu_texture,
        snake_textures,
    };

    let grid_lines = d.draw_grid(state.width);

    // let test_state: GameState = serde_json::from_str(include_str!("../assets/test.json")).unwrap();
    // let game_state: ArcGameState = Arc::new(Mutex::new(test_state));

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
            // Solarized-light background color
            .clear(ClearState::color_and_depth(0.99, 0.96, 0.89, 1.0, 1.0));

        frame_input
            .screen()
            // Solarized-light background color
            //.clear(ClearState::color_and_depth(0.99, 0.96, 0.89, 1.0, 1.0))
            //.clear(ClearState::color_and_depth(0.0, 0.0, 0.0, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                sneks,
                &[],
            );

        frame_input
            .screen()
            .render(&camera2d(frame_input.viewport),
                    grids,
                    &[],);



        thread::sleep(Duration::from_millis(100));

        FrameOutput::default()
    });
}
