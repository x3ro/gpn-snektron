use three_d::*;
use serde::Deserialize;

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
}

const GRID_MARGIN: f32 = 20.0;
const GRID_LINE_THICKNESS: f32 = 2.0;

impl Drawing {
    fn draw_grid(&self, size_in_tiles: usize) -> Vec<Gm<Line, ColorMaterial>> {
        println!("scale factor: {}", self.scale_factor);

        let field_size_px = if self.width > self.height {
            self.height - GRID_MARGIN * 2.0
        } else {
            self.width - GRID_MARGIN * 2.0
        };

        let grid_size_px = field_size_px / (size_in_tiles as f32);

        let mut lines = vec![];

        for x in 0..(size_in_tiles + 1) {
            let mut offset = GRID_MARGIN as f32 + (x as f32 * grid_size_px) as f32;

            let col = (x * 10) as u8;
            let color = Color::BLACK;

            let mut vertical_line = Gm::new(
                Line::new(
                    &self.context,
                    vec2(offset, GRID_MARGIN as f32) * self.scale_factor,
                    vec2(offset, GRID_MARGIN as f32 + field_size_px as f32) * self.scale_factor,
                    GRID_LINE_THICKNESS * self.scale_factor,
                ),
                ColorMaterial {
                    color,
                    ..Default::default()
                },
            );

            let mut horizontal_line = Gm::new(
                Line::new(
                    &self.context,
                    vec2(GRID_MARGIN as f32, offset) * self.scale_factor,
                    vec2(GRID_MARGIN as f32 + field_size_px as f32, offset) * self.scale_factor,
                    GRID_LINE_THICKNESS * self.scale_factor,
                ),
                ColorMaterial {
                    color,
                    ..Default::default()
                },
            );

            lines.push(vertical_line);
            lines.push(horizontal_line)
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
    };

    let grid_lines = d.draw_grid(state.width);

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
        frame_input
            .screen()
            // Solarized-light background color
            .clear(ClearState::color_and_depth(0.99, 0.96, 0.89, 1.0, 1.0))
            .render(
                &camera2d(frame_input.viewport),
                grid_lines.iter(), //.chain(&rectangle).chain(&circle),
                &[],
            );

        FrameOutput::default()
    });
}
