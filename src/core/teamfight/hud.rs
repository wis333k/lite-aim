// Snapshot types cho render + HUD cua mode 5v5.
// Tach rieng khoi `fight.rs` de `World` (core/world.rs) co the giu chung ma
// khong tao vong import.

use crate::core::teamfight::actor::ROSTER_SIZE;

/// mot dong trong kill feed
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FeedEntry {
    pub killer: &'static str,
    pub victim: &'static str,
    pub head: bool,
    /// con bao lau de hien tren HUD
    pub life: f32,
}

/// mot dong trong bang KDA (Tab)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct KdaRow {
    pub name: &'static str,
    pub team: u8,
    pub kills: u32,
    pub deaths: u32,
    pub headshots: u32,
    pub damage: f32,
    pub is_player: bool,
}

impl KdaRow {
    pub fn kd(&self) -> f32 {
        if self.deaths == 0 {
            if self.kills == 0 { 0.0 } else { self.kills as f32 }
        } else {
            self.kills as f32 / self.deaths as f32
        }
    }
}

/// Snapshot cho renderer: 1 bot (nhe hon `Actor`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BotView {
    pub slot: usize,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub pitch: f32,
    pub team: u8,
    pub alive: bool,
    pub moving: bool,
}

/// Snapshot HUD cua mode 5v5.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TfHud {
    pub active: bool,
    pub round: u32,
    pub score: [u32; 2],
    pub player_team: u8,
    pub time_left: f32,
    pub freeze_t: f32,
    pub match_over: bool,
    pub player_hp: f32,
    pub player_armor: f32,
    pub player_kills: u32,
    pub player_deaths: u32,
    pub player_kd: f32,
    pub feed: Vec<FeedEntry>,
    pub scoreboard: Vec<KdaRow>,
    pub banner: String,
    // --- mode BOM ---
    /// mode bom co bat khong
    pub bomb_mode: bool,
    /// nguoi choi co mang bom khong
    pub has_bomb: bool,
    /// bom da trong chua
    pub bomb_planted: bool,
    /// con bao lau bom no (giay), 0 = chua trong
    pub fuse: f32,
    /// tien do trong bom cua nguoi choi (0..1)
    pub plant_bar: f32,
    /// tien do go bom (0..1)
    pub defuse_bar: f32,
    /// nguoi choi dang trong/gỡ (de hien huong dan)
    pub bomb_action: bool,
}

pub const ROSTER: usize = ROSTER_SIZE;
