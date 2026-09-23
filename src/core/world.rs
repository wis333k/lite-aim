// world: game state + FX (PORT NGUYEN tu world.rs, bo project() — Bevy lo camera)
use crate::core::math;
use crate::core::player::Pose;
use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Static,
    Track,
    Bot,
}

#[derive(Clone, Copy)]
pub struct Target {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub r: f32,
    pub kind: Kind,
    pub alive: bool,
    pub hp: f32,
    pub max_hp: f32,
    pub hit_flash: f32,
}

impl Target {
    pub fn new(kind: Kind) -> Self {
        Target { x: 0.0, y: 1.6, z: -8.0, r: 0.55, kind, alive: true, hp: 100.0, max_hp: 100.0, hit_flash: 0.0 }
    }
}

#[derive(Resource)]
pub struct World {
    pub pose: Pose,
    pub fov: f32,
    pub shake: f32,
    pub flash: f32,
    pub tracers: Vec<[f32; 7]>,
    pub wall_hits: Vec<[f32; 2]>,
    pub dmg_nums: Vec<[f32; 6]>,
    pub hitmark: f32,
    pub kill_banner: f32,
    pub hurt: f32,
    pub fx_parts: bool,
    pub fx_dmg: bool,
    pub fx_hit: bool,
    pub shake_on: bool,
    pub parts: Vec<[f32; 8]>,
    pub sfx: Vec<Sfx>,
    // snapshot cho render moi frame (lay tu drill hien tai)
    pub disp_targets: Vec<Target>,
    pub show_blocks: bool,
    pub show_gun: bool,
    // thoi gian tich luy cho shake visual (thay now_ms)
    pub tick_time: f32,
    // viewmodel: loai sung + do giat khi ban (0..1)
    pub gun_kind: u8,
    pub gun_kick: f32,
    // ban: cooldown giua 2 phat
    pub fire_cd: f32,
    // ads: do zoom muot 0..1 (sniper)
    pub ads: f32,
}

// SFX queue: core khong biet bevy, chi ghi nhan su kien de lop render/audio phat
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Sfx {
    Shoot,
    Hit,
    Head,
    Kill,
    Click,
    Death,
}

impl Default for World {
    fn default() -> Self {
        World {
            pose: Pose::default(),
            fov: 103.0,
            shake: 0.0,
            flash: 0.0,
            tracers: Vec::with_capacity(8),
            wall_hits: Vec::with_capacity(32),
            dmg_nums: Vec::with_capacity(12),
            hitmark: 0.0,
            kill_banner: 0.0,
            hurt: 0.0,
            fx_parts: true,
            fx_dmg: true,
            fx_hit: true,
            shake_on: true,
            parts: Vec::with_capacity(120),
            sfx: Vec::with_capacity(8),
            disp_targets: Vec::new(),
            show_blocks: false,
            show_gun: false,
            tick_time: 0.0,
            gun_kind: 0,
            gun_kick: 0.0,
            fire_cd: 0.0,
            ads: 0.0,
        }
    }
}

impl World {
    pub fn forward(&self) -> (f32, f32, f32) {
        math::forward(self.pose.yaw, self.pose.pitch)
    }

    pub fn ground_height(&self, px: f32, pz: f32) -> f32 {
        math::ground_height(px, pz, self.pose.feet)
    }

    pub fn block_forward_dist(&self, max_d: f32) -> f32 {
        let (dx, dy, dz) = self.forward();
        let c = self.pose.pos;
        math::block_forward_dist(c[0], c[1], c[2], dx, dy, dz, max_d)
    }

    pub fn segment_blocked(&self, fx: f32, fy: f32, fz: f32, tx: f32, ty: f32, tz: f32) -> bool {
        math::segment_blocked(fx, fy, fz, tx, ty, tz)
    }

    pub fn kick(&mut self, amount: f32) {
        self.flash = 0.045;
        if self.shake_on {
            self.shake = (self.shake + amount).min(2.0);
        }
    }

    pub fn burst(&mut self, x: f32, y: f32, z: f32, n: usize, speed: f32, kind: usize, ttl: f32) {
        if !self.fx_parts {
            return;
        }
        for i in 0..n {
            if self.parts.len() >= 120 {
                self.parts.remove(0);
            }
            let a = (i as f32 * 2.39996) + (self.pose.bob + x + y) * 0.37;
            let up = ((i as f32 * 0.73).sin() * 0.5 + 0.5) * speed;
            self.parts.push([
                x, y, z,
                a.cos() * speed * 0.7, up * 0.6, a.sin() * speed * 0.7,
                ttl * (0.7 + (i as f32 * 0.13).sin().abs() * 0.6), kind as f32,
            ]);
        }
    }

    pub fn add_tracer(&mut self, x1: f32, y1: f32, z1: f32, x2: f32, y2: f32, z2: f32) {
        if self.tracers.len() >= 8 { self.tracers.remove(0); }
        self.tracers.push([x1, y1, z1, x2, y2, z2, 0.07]);
    }

    pub fn add_dmg(&mut self, x: f32, y: f32, z: f32, amount: f32, head: bool) {
        if self.dmg_nums.len() >= 12 { self.dmg_nums.remove(0); }
        self.dmg_nums.push([x, y, z, amount, if head { 1.0 } else { 0.0 }, 0.7]);
    }

    // thong so sung theo kind: (rpm, auto?, zoom_fov_deg)
    // 0=Pistol 1=Rifle 2=Sniper 3=Smg
    pub fn gun_stats(&self) -> (f32, bool, f32) {
        match self.gun_kind {
            0 => (450.0, false, 0.0),
            1 => (750.0, true, 0.0),
            2 => (150.0, false, 28.0),
            3 => (1050.0, true, 0.0),
            _ => (500.0, false, 0.0),
        }
    }

    // fov thuc te = fov goc + zoom ads (sniper)
    pub fn eff_fov(&self) -> f32 {
        let (_, _, zoom) = self.gun_stats();
        if zoom > 0.0 && self.ads > 0.0 {
            self.fov + (zoom - self.fov) * self.ads
        } else {
            self.fov
        }
    }

    pub fn tick_fx(&mut self, dt: f32) {
        self.tick_time += dt;
        if self.flash > 0.0 { self.flash -= dt; }
        if self.shake > 0.0 { self.shake = (self.shake - dt * 4.0).max(0.0); }
        if self.hitmark > 0.0 { self.hitmark -= dt; }
        if self.kill_banner > 0.0 { self.kill_banner -= dt; }
        if self.hurt > 0.0 { self.hurt = (self.hurt - dt * 1.4).max(0.0); }
        if self.gun_kick > 0.0 { self.gun_kick = (self.gun_kick - dt * 9.0).max(0.0); }
        self.tracers.retain_mut(|tr| { tr[6] -= dt; tr[6] > 0.0 });
        self.dmg_nums.retain_mut(|d| { d[5] -= dt; d[5] > 0.0 });
        self.parts.retain_mut(|p| {
            p[6] -= dt;
            p[0] += p[3] * dt;
            p[1] += p[4] * dt;
            p[2] += p[5] * dt;
            p[4] -= 4.0 * dt;
            p[6] > 0.0
        });
    }

    // bot hien tai (Target co kind Bot) tu snapshot
    pub fn bot_target(&self) -> Option<Target> {
        self.disp_targets.iter().copied().find(|t| t.kind == Kind::Bot)
    }

    // bot dang flash (bi ban)
    pub fn bot_hit(&self) -> bool {
        self.disp_targets.iter().any(|t| t.kind == Kind::Bot && t.hit_flash > 0.0)
    }
}
