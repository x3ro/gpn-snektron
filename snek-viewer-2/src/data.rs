use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub type ArcGameState = Arc<Mutex<GameState>>;

pub struct ServerGameState {
    pub game_state: ArcGameState,
    pub current_game_id: String,
    pub current_version: usize,
    pub last_update: Instant,
}

impl ServerGameState {
    pub fn new(game_state: ArcGameState) -> ServerGameState {
        ServerGameState {
            game_state,
            current_game_id: "".to_string(),
            current_version: 0,
            last_update: Instant::now(),
        }
    }
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

pub struct PlayerId(pub usize);

// TODO: it probably makes more sense to call `versions` "turns" instead
pub struct VersionId(pub usize);

pub struct Size {
    pub width: f32,
    pub height: f32,
}
impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}
