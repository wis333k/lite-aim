// sfx: synth WAV procedural (PORT NGUYEN) — phat qua bevy_audio

use bevy::audio::{AudioPlayer, AudioSource, PlaybackSettings, Volume};
use bevy::prelude::*;
use std::sync::atomic::{AtomicU32, Ordering};

const SR: u32 = 44100;

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

fn to_wav(samples: &[f32]) -> Vec<u8> {
    let mut v = Vec::with_capacity(44 + samples.len() * 2);
    v.extend_from_slice(b"RIFF");
    v.extend_from_slice(&((36 + samples.len() * 2) as u32).to_le_bytes());
    v.extend_from_slice(b"WAVEfmt ");
    v.extend_from_slice(&16u32.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&1u16.to_le_bytes());
    v.extend_from_slice(&SR.to_le_bytes());
    v.extend_from_slice(&(SR * 2).to_le_bytes());
    v.extend_from_slice(&2u16.to_le_bytes());
    v.extend_from_slice(&16u16.to_le_bytes());
    v.extend_from_slice(b"data");
    v.extend_from_slice(&((samples.len() * 2) as u32).to_le_bytes());
    for s in samples {
        v.extend_from_slice(&((s.clamp(-1.0, 1.0) * 32767.0) as i16).to_le_bytes());
    }
    v
}

fn sine(f: f32, t: f32) -> f32 {
    (t * f * std::f32::consts::TAU).sin()
}

fn noise(seed: &mut u32) -> f32 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*seed >> 9) as f32 / 8388608.0) * 2.0 - 1.0
}

fn w_shoot() -> Vec<u8> {
    let n = (SR as f32 * 0.14) as usize;
    let mut seed = 12345u32;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        s.push(noise(&mut seed) * (-t * 30.0).exp() * 0.7 + sine(110.0, t) * (-t * 25.0).exp() * 0.6);
    }
    to_wav(&s)
}
fn w_hit() -> Vec<u8> {
    let n = (SR as f32 * 0.07) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        s.push(sine(880.0, t) * (-t * 40.0).exp() * 0.7);
    }
    to_wav(&s)
}
fn w_head() -> Vec<u8> {
    let n = (SR as f32 * 0.10) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        s.push(sine(1318.0, t) * (-t * 35.0).exp() * 0.8);
    }
    to_wav(&s)
}
fn w_kill() -> Vec<u8> {
    let n = (SR as f32 * 0.30) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let f = if t < 0.12 { 660.0 } else { 990.0 };
        s.push(sine(f, t) * (-t * 8.0).exp() * 0.7);
    }
    to_wav(&s)
}
fn w_click() -> Vec<u8> {
    let n = (SR as f32 * 0.045) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        s.push(sine(1900.0, t) * (-t * 80.0).exp() * 0.45);
    }
    to_wav(&s)
}
fn w_death() -> Vec<u8> {
    let n = (SR as f32 * 0.45) as usize;
    let mut s = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f32 / SR as f32;
        let ph = 320.0 * t - 280.0 * t * t;
        s.push((ph * std::f32::consts::TAU).sin() * (-t * 6.0).exp() * 0.7);
    }
    to_wav(&s)
}

pub fn init(assets: &mut Assets<AudioSource>) -> SfxBank {
    let mut add = |bytes: Vec<u8>| assets.add(AudioSource { bytes: bytes.into() });
    SfxBank {
        shoot: add(w_shoot()),
        hit: add(w_hit()),
        head: add(w_head()),
        kill: add(w_kill()),
        click: add(w_click()),
        death: add(w_death()),
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
