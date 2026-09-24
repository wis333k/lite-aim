// sfx: am thanh .ogg that (Kenney CC0) — phat qua bevy_audio

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

pub const SHOOT_OGG: &str = "embedded://assets/audio/shoot.ogg";
pub const HIT_OGG: &str = "embedded://assets/audio/hit.ogg";
pub const HEAD_OGG: &str = "embedded://assets/audio/head.ogg";
pub const KILL_OGG: &str = "embedded://assets/audio/kill.ogg";
pub const CLICK_OGG: &str = "embedded://assets/audio/click.ogg";
pub const DEATH_OGG: &str = "embedded://assets/audio/death.ogg";

#[derive(Resource)]
pub struct SfxBank {
    pub shoot: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub head: Handle<AudioSource>,
    pub kill: Handle<AudioSource>,
    pub click: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
}

static VOL: AtomicU32 = AtomicU32::new(0.6f32.to_bits());

pub fn set_volume(v: f32) {
    VOL.store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
}

fn vol() -> f32 {
    f32::from_bits(VOL.load(Ordering::Relaxed))
}

pub fn init(server: &AssetServer) -> SfxBank {
    SfxBank {
        shoot: server.load(SHOOT_OGG),
        hit: server.load(HIT_OGG),
        head: server.load(HEAD_OGG),
        kill: server.load(KILL_OGG),
        click: server.load(CLICK_OGG),
        death: server.load(DEATH_OGG),
    }
}

pub enum SfxKind {
    Shoot,
    Hit,
    Head,
    Kill,
    Click,
    Death,
}

// spawn audio entity (bevy 0.18: AudioPlayer + Volume)
pub fn play(commands: &mut Commands, bank: &SfxBank, kind: SfxKind) {
    let v = vol();
    if v <= 0.001 {
        return;
    }
    let h = match kind {
        SfxKind::Shoot => bank.shoot.clone(),
        SfxKind::Hit => bank.hit.clone(),
        SfxKind::Head => bank.head.clone(),
        SfxKind::Kill => bank.kill.clone(),
        SfxKind::Click => bank.click.clone(),
        SfxKind::Death => bank.death.clone(),
    };
    commands.spawn((AudioPlayer(h), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(v))));
}
