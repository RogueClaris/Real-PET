// Increment VERSION_ITERATION src/packets/mod.rs if packets are added or modified

use super::structures::{BattleStatistics, Direction};
use crate::structures::PackageId;
use serde::{Deserialize, Serialize};
use strum::IntoStaticStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ClientAssetType {
    Texture,
    Animation,
    MugshotTexture,
    MugshotAnimation,
}

#[derive(Clone, Debug, Serialize, Deserialize, IntoStaticStr, PartialEq)]
pub enum ClientPacket {
    VersionRequest,
    Authorize {
        origin_address: String,
        identity: Vec<u8>,
        data: Vec<u8>,
    },
    Heartbeat,
    AssetFound {
        path: String,
        last_modified: u64,
    },
    Asset {
        asset_type: ClientAssetType,
        data: Vec<u8>,
    },
    Login {
        username: String,
        identity: Vec<u8>,
        data: String,
    },
    Logout,
    RequestJoin,
    Ready {
        time: u64,
    },
    TransferredOut,
    Position {
        creation_time: u64,
        x: f32,
        y: f32,
        z: f32,
        direction: Direction,
    },
    Boost {
        health_boost: i32,
        blocks: Vec<PackageId>,
    },
    AvatarChange {
        name: String,
        element: String,
        base_health: i32,
    },
    Emote {
        emote_id: u8,
    },
    CustomWarp {
        tile_object_id: u32,
    },
    ObjectInteraction {
        tile_object_id: u32,
        button: u8,
    },
    ActorInteraction {
        actor_id: String,
        button: u8,
    },
    TileInteraction {
        x: f32,
        y: f32,
        z: f32,
        button: u8,
    },
    TextBoxResponse {
        response: u8,
    },
    PromptResponse {
        response: String,
    },
    BoardOpen,
    BoardClose,
    PostRequest,
    PostSelection {
        post_id: String,
    },
    ShopClose,
    ShopPurchase {
        item_name: String,
    },
    EncounterStart,
    BattleResults {
        battle_stats: BattleStatistics,
    },
}
