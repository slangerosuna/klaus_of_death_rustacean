use crate::*;
use egui::Event;
use egui::Key;
use std::sync::mpsc;

pub struct Input {
    rx: mpsc::Receiver<Vec<Event>>,
    keys_down: Vec<bool>,
}
impl_resource!(Input, 3);

pub struct InputSender {
    pub tx: mpsc::Sender<Vec<Event>>,
}

impl Input {
    pub fn new() -> (Self, InputSender) {
        let key_count = std::mem::variant_count::<Key>();
        let keys_down = vec![false; key_count];
        let (tx, rx) = mpsc::channel();

        (Self { rx, keys_down }, InputSender { tx })
    }

    pub fn handle_events(&mut self) {
        let events = self.rx.recv().unwrap();
        for event in events {
            match event {
                Event::Key { key, pressed, .. } => {
                    let key_idx = key as usize;
                    self.keys_down[key_idx] = pressed;
                }
                _ => (),
            }
        }
    }

    pub fn is_key_pressed(&self, key: Key) -> bool {
        self.keys_down[key as usize]
    }
}

create_system!(handle_input, get_handle_input_system;
    uses Input);
pub async fn handle_input(game_state: &mut GameState, _t: f64, _dt: f64) {
    let input = game_state.get_resource_mut::<Input>().unwrap();
    input.handle_events();
}
