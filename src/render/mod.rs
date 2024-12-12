pub mod device;

use crate::*;

create_system!(init, get_init_system; uses GameState);
async fn init(_game_state: &mut GameState, _time: f64, _dt: f64) {}
