use std::sync::Mutex;
use colorsys::{ColorTransform, Rgb};
use three_d::*;
use serde::Deserialize;
use itertools::Itertools;

// Entry point for wasm
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_log::init_with_level(log::Level::Debug).unwrap();

    use log::info;
    info!("Logging works!");

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    main();
    Ok(())
}




#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

#[derive(Debug, Clone)]
pub struct PreviousPos(pub Position);

#[derive(Debug, Deserialize, Clone)]
pub struct PlayerState {
    pub alive: bool,
    pub chat: Option<String>,
    pub name: String,
    pub pos: Position,
    pub moves: Vec<Position>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct GameState {
    #[serde(default)]
    pub version: usize,
    pub height: usize,
    pub width: usize,
    pub id: String,
    pub players: Vec<PlayerState>,
}

struct Drawing {
    context: Context,
    scale_factor: f32,
    width: f32,
    height: f32,
    field_size_in_tiles: usize,
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
        let y = GRID_MARGIN + (y * self.grid_size_screen()) as f32;
        vec2(x, y) * self.scale_factor
    }

    fn draw_snek(&self, positions: Vec<Position>, base_color: Color) -> Vec<Gm<Circle, ColorMaterial>> {
        assert!(positions.len() > 0);
        let radius = self.grid_size_screen() * 0.8;

        let mut c = Rgb::new(base_color.r as f64, base_color.g as f64, base_color.b as f64, None);

        let mut res = vec![];
        for (pos, next_pos) in positions.iter().tuple_windows() {
            res.push(Gm::new(
                Circle::new(
                    &self.context,
                    self.pos(pos.x as f32 + 0.5, pos.y as f32 + 0.5),
                    radius,
                ),
                ColorMaterial {
                    color: c.to_color(),
                    ..Default::default()
                },
            ));

            let diff_x = (pos.x as f32 - next_pos.x as f32) / 2.0;
            let diff_y = (pos.y as f32 - next_pos.y as f32) / 2.0;

            c.lighten(2.0);

            if diff_x != 0.0 {
                res.push(Gm::new(
                    Circle::new(
                        &self.context,
                        self.pos(pos.x as f32  + 0.5 - diff_x, pos.y as f32 + 0.5),
                        radius,
                    ),
                    ColorMaterial {
                        color: c.to_color(),
                        ..Default::default()
                    },
                ));
            }

            if diff_y != 0.0 {
                res.push(Gm::new(
                    Circle::new(
                        &self.context,
                        self.pos(pos.x as f32  + 0.5, pos.y as f32 + 0.5 - diff_y),
                        radius,
                    ),
                    ColorMaterial {
                        color: c.to_color(),
                        ..Default::default()
                    },
                ));
            }
        }

        let last = positions.last().unwrap();
        res.push(Gm::new(
            Circle::new(
                &self.context,
                self.pos(last.x as f32 + 0.5, last.y as f32 + 0.5),
                radius,
            ),
            ColorMaterial {
                color: c.to_color(),
                ..Default::default()
            },
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

pub fn main() {
    let window = Window::new(WindowSettings {
        title: "Shapes 2D!".to_string(),
        ..Default::default()
    })
        .unwrap();

    let state = GameState {
        version: 1,
        width: 8,
        height: 8,
        id: "foobar".to_string(),
        players: vec![],
    };

    let context = window.gl();
    let scale_factor = window.device_pixel_ratio();
    let (width, height) = window.size();

    let d = Drawing {
        context,
        scale_factor,
        width: width as f32,
        height: height as f32,
        field_size_in_tiles: state.width,
    };

    let grid_lines = d.draw_grid(state.width);

    let snek = d.draw_snek(vec![
        Position { x: 3, y: 3 },
        Position { x: 3, y: 2 },
        Position { x: 3, y: 1 },
        Position { x: 2, y: 1 },
        Position { x: 1, y: 1 },
        Position { x: 0, y: 1 },
    ], Color::BLUE);

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



        let sneks = snek.iter()
            .flat_map(|m| m.into_iter());

        let grids = grid_lines
            .iter()
            .flat_map(|m| m.into_iter());

        let objects = grids.chain(sneks);

        frame_input
            .screen()
            // Solarized-light background color
            .clear(ClearState::color_and_depth(0.99, 0.96, 0.89, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                objects,
                &[],
            );

        FrameOutput::default()
    });
}
