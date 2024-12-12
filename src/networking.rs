use serde::{Deserialize, Serialize};

use crate::impl_resource;

pub struct NetworkingCreationInfo {
    pub max_players: u16,
    pub max_synced_objects: u32,
    pub packet_per_frame_limit: u32,
}

impl Default for NetworkingCreationInfo {
    fn default() -> Self {
        Self {
            max_players: 32,
            max_synced_objects: 1024,
            packet_per_frame_limit: 64,
        }
    }
}

pub struct Networking {
    pub max_players: u16,
    pub max_synced_objects: u32,
    pub packet_per_frame_limit: u32,

    pub connected: bool,
}
impl_resource!(Networking, 1);

impl Networking {
    pub fn new(info: NetworkingCreationInfo) -> Self {
        Networking {
            max_players: info.max_players,
            max_synced_objects: info.max_synced_objects,
            packet_per_frame_limit: info.packet_per_frame_limit,
            connected: false,
        }
    }

    pub fn update(&mut self) {}
}

#[repr(u8)]
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum EventType {
    EntityCreate,
    EntityDelete,
    EntityUpdate,
    PlayerJoin,
    PlayerLeave,
    Event,
}
