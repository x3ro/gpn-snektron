use std::ops::Deref;
use crate::data::{ArcGameState, GameState};
use json_patch::Patch;
use rust_socketio::{ClientBuilder, Payload, RawClient};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

pub fn client_thread(ticks: ArcGameState) -> JoinHandle<()> {
    thread::spawn(move || {
        let game_state_value = Arc::new(Mutex::new(Value::default()));

        let ticks_a = ticks.clone();
        let game_state_value_a = game_state_value.clone();
        let init_callback = move |payload: Payload, _socket: RawClient| match payload {
            Payload::String(str) => {
                let mut v = ticks_a.lock().unwrap();
                let mut gsv = game_state_value_a.lock().unwrap();
                *gsv = serde_json::from_str(&str).expect("Failed to parse JSON");
                let mut state: GameState =
                    serde_json::from_value(gsv.get("game").unwrap().clone()).unwrap();
                state.version = 1;
                *v = state;
            }
            Payload::Binary(_) => unimplemented!("init_callback binary"),
        };

        let ticks_b = ticks.clone();
        let game_state_value_b = game_state_value.clone();
        let patch_callback = move |payload: Payload, _socket: RawClient| {
            match payload {
                Payload::String(str) => {
                    // Parse the JSON Patch from the message
                    let patch_value: Value = serde_json::from_str(&str).unwrap();
                    let patch: Patch = serde_json::from_value(patch_value).unwrap();

                    // Apply it to the Value we're keeping in the thread state
                    let mut gsv = game_state_value_b.lock().unwrap();
                    json_patch::patch(&mut gsv, &patch).unwrap();

                    println!("{}\n\n", serde_json::to_string(gsv.deref()).unwrap());

                    // Parse the patched Value into the new GameState and bump the version
                    let mut state: GameState =
                        serde_json::from_value(gsv.get("game").unwrap().clone()).unwrap();
                    let mut v = ticks_b.lock().unwrap();
                    state.version = v.version + 1;

                    // Finally replace the GameState in the Arc
                    *v = state;
                }
                Payload::Binary(_) => unimplemented!("patch_callback binary"),
            }
        };

        let _socket = ClientBuilder::new("http://gpn21-snektron.unprofessional.consulting:4001")
            .namespace("/")
            .on("init", init_callback)
            .on("patch", patch_callback)
            .on("error", |err, _| eprintln!("Error: {:#?}", err))
            .connect()
            .expect("Connection failed");

        println!("Connected!");
    })
}
