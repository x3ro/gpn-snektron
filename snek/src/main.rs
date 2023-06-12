use std::io::BufReader;
use std::io::prelude::*;
use std::net::{Ipv6Addr, TcpStream};
use std::{iter, thread};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use anyhow::{Result, bail, Context};


// fn read_one_line(reader: &mut BufReader<TcpStream>) {
//     let mut line = String::new();
//     assert!(reader.read_line(&mut line).unwrap() > 0);
//     println!("recv: {}", line);
// }

#[derive(Debug)]
enum Message {
    Motd(String),
    Error(String),
    Game { width: usize, height: usize, player_id: usize },
    Pos { player_id: usize, x: usize, y: usize },
    Tick,
    Die(Vec<usize>),
    Message { player_id: usize, msg: String },
    Win { wins: usize, losses: usize },
    Lose { wins: usize, losses: usize },
}

#[derive(Debug)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn as_str(&self) -> &'static str {
        match self {
            Direction::Up => "up",
            Direction::Right => "right",
            Direction::Down => "down",
            Direction::Left => "left",
        }
    }
}

impl Message {
    fn from(s: String) -> Result<Message> {
        let trimmed = s.trim();
        let parts: Vec<_> = trimmed.split("|").collect();
        match &parts.as_slice() {
            ["motd", msg] => {
                Ok(Message::Motd(msg.to_string()))
            }

            ["error", msg] => {
                Ok(Message::Error(msg.to_string()))
            }

            ["game", width, height, player_id] => {
                Ok(Message::Game {
                    width: width.parse::<usize>().context("Game.width")?,
                    height: height.parse::<usize>().context("Game.height")?,
                    player_id: player_id.parse::<usize>().context("Game.player_id")?,
                })
            }

            ["pos", id, x, y] => {
                Ok(Message::Pos {
                    player_id: id.parse::<usize>().context("Pos.player_id")?,
                    x: x.parse::<usize>().context("Pos.x")?,
                    y: y.parse::<usize>().context("Pos.y")?,
                })
            },

            ["tick"] => {
                Ok(Message::Tick)
            }

            ["die", players@ ..] => {
                let list: Vec<_> = players.iter().map(|id| id.parse::<usize>().unwrap()).collect();
                Ok(Message::Die(list))
            }

            ["message", player_id, msg] => {
                let player_id = player_id.parse::<usize>().context("Message.player_id")?;
                Ok(Message::Message { player_id, msg: msg.to_string()})
            }

            ["win", wins, losses] => {
                Ok(Message::Win {
                    wins: wins.parse::<usize>().context("Win.wins")?,
                    losses: losses.parse::<usize>().context("Win.losses")?,
                })
            }

            ["lose", wins, losses] => {
                Ok(Message::Lose {
                    wins: wins.parse::<usize>().context("Lose.wins")?,
                    losses: losses.parse::<usize>().context("Lose.losses")?,
                })
            }

            x => {
                bail!("Failed to parse message '{}'", trimmed)
            }
        }

    }
}

const MARKERS: &[char] = &['X', 'O', 'V', 'B'];


#[derive(Debug)]
struct GameRound {
    width: usize,
    height: usize,
    player_id: usize,
    alive_players: usize,
    first_tick: bool,
    player_state: Vec<Option<usize>>,
    x: usize,
    y: usize,
    stream: Rc<RefCell<TcpStream>>,
}

impl GameRound {
    pub fn new(stream: Rc<RefCell<TcpStream>>, player_id: usize, width: usize, height: usize) -> GameRound {
        GameRound {
            width,
            height,
            player_id,
            alive_players: 0,
            first_tick: true,
            player_state: iter::repeat(None).take(width * height).collect(),
            x: 0,
            y: 0,
            stream,
        }
    }

    pub fn offset(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn next_offset(&self, mut x: usize, mut y: usize, dir: Direction) -> usize {
        let mut x = x as i32;
        let mut y = y as i32;
        match dir {
            Direction::Up => y -= 1,
            Direction::Down => y += 1,
            Direction::Right => x += 1,
            Direction::Left => x -= 1,
        };

        let width = self.width as i32;
        let height = self.height as i32;
        x = ((x % width) + width) % width;
        y = ((y % height) + height) % height;
        self.offset(x as usize, y as usize)
    }

    pub fn is_move_blocked(&self, mut x: usize, mut y: usize, dir: Direction) -> bool {
        let next_offset = self.next_offset(x, y, dir);
        self.player_state[next_offset].is_some()
    }

    pub fn send_move(&self, dir: Direction) {
        let join_msg = format!("move|{}\n", dir.as_str());
        self.stream.borrow_mut().write(join_msg.as_bytes()).expect("Failed to send join message");
        println!("Moving {}!", dir.as_str());
    }
}


fn round_loop(reader: &mut BufReader<TcpStream>, mut info: GameRound) {
    println!("\n\n\nStarting a new round. Player {}, Map: {}x{}", info.player_id, info.width, info.height);

    loop {
        let msg = read_next_message(reader);
        match msg {
            Message::Pos { player_id, x, y } => {
                let offset = info.offset(x, y);
                info.player_state[offset] = Some(player_id);

                if player_id == info.player_id {
                    info.x = x;
                    info.y = y;
                }

                if info.first_tick {
                    info.alive_players += 1;
                }
            }

            Message::Tick => {
                // In the first tick we need to collect information on who
                // we are playing against, so it gets special handling.
                info.first_tick = false;

                if info.is_move_blocked(info.x, info.y, Direction::Up) {
                    println!("Up is not free");
                    if !info.is_move_blocked(info.x, info.y, Direction::Right) {
                        info.send_move(Direction::Right);
                    } else {
                        info.send_move(Direction::Left);
                    }
                }

                // if info.player_state[next_offset].is_some() {
                //     info.send_move(Direction::Right);
                //     println!("Moving right!");
                // }

                for y in 0..info.height  {
                    for x in 0..info.width {
                        let offset = y * info.width + x;
                        if let Some(player_id) = info.player_state[offset] {
                            let marker = MARKERS[player_id];
                            print!("{} ", marker);
                        } else {
                            print!("  ");
                        }
                    }
                    print!("\n")
                }
                println!("{}", "-".repeat(info.width*2));
            }

            Message::Die(ids) => {
                info.alive_players -= ids.len();
                println!("Players left alive: {}", info.alive_players);
                for x in info.player_state.iter_mut() {
                    if x.is_none() {
                        continue;
                    }

                    let id = x.unwrap();
                    if ids.contains(&id) {
                        *x = None;
                    }
                }
            }

            Message::Win { .. } => {
                println!("Won!");
                return
            }

            Message::Lose { .. } => {
                println!("Lost!");
                break
            }

            Message::Message { .. } => { /* Don't care */ }

            msg => {
                println!("Unhandled message in round loop: {:?}", msg);
            }
        }
    }

}

fn read_next_message(reader: &mut BufReader<TcpStream>) -> Message {
    let mut line = String::new();
    let size = reader.read_line(&mut line).expect("Couldn't read message from Game Server");
    assert!(size > 0, "Connection to Game Server seems to have been lost");
    Message::from(line.clone()).expect("Failed to parse message from Game Server")
}

fn main() {
    loop {
        println!("Attempting connection");
        connect_loop();
        println!("Connection closed, waiting for some time");
        thread::sleep(Duration::from_secs(2));
    }
}

fn connect_loop() {
    let ip = "127.0.0.1:4000";
    let mut stream = TcpStream::connect(ip)
        .expect("Connection to game server failed");
    let mut stream = Rc::new(RefCell::new(stream));

    let mut reader = BufReader::new(stream.borrow().try_clone().unwrap());

    loop {
        let msg = read_next_message(&mut reader);
        match msg {
            Message::Game { width, height, player_id } => {
                round_loop(&mut reader, GameRound::new(stream.clone(), player_id, width, height))
            }

            Message::Error(msg) => {
                println!("Game Server sends error: {msg}");
                return;
            }

            Message::Motd(msg) => {
                println!("MOTD: {msg}");
                let join_msg = "join|Snekisnek|jkasdfjkshdfjksdfkjhsdkjhfsdjk\n";
                stream.borrow_mut().write(join_msg.as_bytes()).expect("Failed to send join message");
            }

            x => {
                println!("ignoring: {:?}", x);
            }
        }
    }
}
