// drills: gridshot/flick/tracking/recoil (PORT NGUYEN, hitscan = ray 3D that)
use crate::core::arena::{DMG_BODY, DMG_HEAD};
use crate::core::config::Results;
use crate::core::math;
use crate::core::world::{Kind, Sfx, Target, World};
use bevy::prelude::*;

fn rnd(a: f32, b: f32) -> f32 {
    fastrand::f32() * (b - a) + a
}

pub fn grid_pos() -> (f32, f32, f32) {
    (rnd(-5.5, 5.5), 1.6 + rnd(-1.3, 1.3), -8.0)
}

pub fn flick_pos() -> (f32, f32, f32) {
    (rnd(-7.0, 7.0), 1.6 + rnd(-1.8, 1.8), -8.0 - rnd(0.0, 4.0))
}

// hitscan 3D: tia tu camera, trai cau nao gan nhat (thay project() screen-space)
pub fn hitscan(w: &World, targets: &[Target]) -> Option<usize> {
    let (dx, dy, dz) = w.forward();
    let (fx, fy, fz) = (w.pose.pos[0], w.pose.pos[1], w.pose.pos[2]);
    let mut best: Option<usize> = None;
    let mut best_t = 1e9f32;
    for (i, t) in targets.iter().enumerate() {
        if !t.alive {
            continue;
        }
        if let Some(th) = math::ray_sphere(fx, fy, fz, dx, dy, dz, t.x, t.y, t.z, t.r) {
            if th < best_t {
                best = Some(i);
                best_t = th;
            }
        }
    }
    best
}

pub enum Drill {
    Gridshot(Gridshot),
    Flick(Flick),
    Tracking(Tracking),
    Recoil(Recoil),
    Duel(Duel),
}

impl Drill {
    pub fn update(&mut self, w: &mut World, dt: f32, mouse_down: bool, keys: &Keys) {
        match self {
            Drill::Gridshot(d) => d.update(w, dt),
            Drill::Flick(d) => d.update(w, dt),
            Drill::Tracking(d) => d.update(w, dt),
            Drill::Recoil(d) => d.update(w, dt, mouse_down),
            Drill::Duel(d) => d.update(w, dt, keys),
        }
    }
    pub fn on_mousedown(&mut self, w: &mut World) {
        match self {
            Drill::Gridshot(d) => d.on_mousedown(w),
            Drill::Flick(d) => d.on_mousedown(w),
            Drill::Tracking(_) => {}
            Drill::Recoil(_) => {}
            Drill::Duel(d) => d.on_mousedown(w),
        }
    }
    pub fn timer(&self) -> (f32, f32) {
        match self {
            Drill::Gridshot(d) => d.timer(),
            Drill::Flick(d) => d.timer(),
            Drill::Tracking(d) => d.timer(),
            Drill::Recoil(d) => d.timer(),
            Drill::Duel(d) => d.timer(),
        }
    }
    pub fn score(&self) -> String {
        match self {
            Drill::Gridshot(d) => d.score(),
            Drill::Flick(d) => d.score(),
            Drill::Tracking(d) => d.score(),
            Drill::Recoil(d) => d.score(),
            Drill::Duel(d) => d.score(),
        }
    }
    pub fn hp(&self) -> f32 {
        match self {
            Drill::Duel(d) => d.hp(),
            _ => 100.0,
        }
    }
    pub fn show_msg(&self) -> bool {
        match self {
            Drill::Duel(d) => d.show_msg(),
            _ => false,
        }
    }
    pub fn targets(&self) -> &[Target] {
        match self {
            Drill::Gridshot(d) => &d.targets,
            Drill::Flick(d) => &d.targets,
            Drill::Tracking(d) => std::slice::from_ref(&d.target),
            Drill::Recoil(_) => &[],
            Drill::Duel(d) => std::slice::from_ref(&d.bot),
        }
    }
    pub fn mode_wall(&self) -> bool {
        matches!(self, Drill::Recoil(_))
    }
    pub fn draw_blocks(&self) -> bool {
        matches!(self, Drill::Duel(_))
    }
    pub fn has_gun(&self) -> bool {
        matches!(self, Drill::Duel(_))
    }
    pub fn results(&self) -> Results {
        match self {
            Drill::Gridshot(d) => d.results(),
            Drill::Flick(d) => d.results(),
            Drill::Tracking(d) => d.results(),
            Drill::Recoil(d) => d.results(),
            Drill::Duel(d) => d.results(),
        }
    }
    pub fn mode_name(&self) -> &'static str {
        match self {
            Drill::Gridshot(_) => "GRIDSHOT",
            Drill::Flick(_) => "FLICK",
            Drill::Tracking(_) => "TRACKING",
            Drill::Recoil(_) => "RECOIL",
            Drill::Duel(_) => "BOT DUEL",
        }
    }
}

/* ============ GRIDSHOT ============ */
pub struct Gridshot {
    pub targets: Vec<Target>,
    borns: [f32; 3],
    time: f32,
    duration: f32,
    shots: u32,
    hits: u32,
    ttks: Vec<f32>,
}

impl Gridshot {
    pub fn new(duration: f32) -> Self {
        let mut targets = Vec::with_capacity(3);
        let mut borns = [0.0f32; 3];
        for i in 0..3 {
            let mut t = Target::new(Kind::Static);
            t.r = 0.55;
            let p = grid_pos();
            t.x = p.0;
            t.y = p.1;
            t.z = p.2;
            borns[i] = 0.0;
            targets.push(t);
        }
        Gridshot { targets, borns, time: 0.0, duration, shots: 0, hits: 0, ttks: Vec::new() }
    }
    fn update(&mut self, _w: &mut World, dt: f32) {
        self.time += dt;
        for t in self.targets.iter_mut() {
            if t.hit_flash > 0.0 { t.hit_flash -= dt; }
        }
    }
    fn on_mousedown(&mut self, w: &mut World) {
        self.shots += 1;
        w.sfx.push(Sfx::Shoot);
        if let Some(i) = hitscan(w, &self.targets) {
            self.hits += 1;
            self.ttks.push(0.0);
            let p = grid_pos();
            self.targets[i].hit_flash = 0.08;
            self.targets[i].x = p.0;
            self.targets[i].y = p.1;
            self.targets[i].z = p.2;
            w.kick(0.05);
            w.sfx.push(Sfx::Hit);
        } else {
            w.kick(0.03);
        }
    }
    fn timer(&self) -> (f32, f32) { (self.duration - self.time, self.duration) }
    fn score(&self) -> String { self.hits.to_string() }
    fn results(&self) -> Results {
        let acc = if self.shots > 0 { (100.0 * self.hits as f32 / self.shots as f32).round() as u32 } else { 0 };
        let ttk = if !self.ttks.is_empty() { self.ttks.iter().sum::<f32>() / self.ttks.len() as f32 } else { self.duration };
        Results::new(0, "gridshot", self.hits.to_string(), self.hits as f32, acc, ttk)
    }
}

/* ============ FLICK ============ */
pub struct Flick {
    pub targets: Vec<Target>,
    time: f32,
    duration: f32,
    shots: u32,
    hits: u32,
    ttks: Vec<f32>,
}

impl Flick {
    pub fn new(duration: f32) -> Self {
        let mut t = Target::new(Kind::Static);
        t.r = 0.5;
        let p = flick_pos();
        t.x = p.0;
        t.y = p.1;
        t.z = p.2;
        Flick { targets: vec![t], time: 0.0, duration, shots: 0, hits: 0, ttks: Vec::new() }
    }
    fn update(&mut self, _w: &mut World, dt: f32) {
        self.time += dt;
        for t in self.targets.iter_mut() {
            if t.hit_flash > 0.0 { t.hit_flash -= dt; }
        }
    }
    fn on_mousedown(&mut self, w: &mut World) {
        self.shots += 1;
        w.sfx.push(Sfx::Shoot);
        if let Some(i) = hitscan(w, &self.targets) {
            self.hits += 1;
            self.ttks.push(0.0);
            let p = flick_pos();
            self.targets[i].hit_flash = 0.08;
            self.targets[i].x = p.0;
            self.targets[i].y = p.1;
            self.targets[i].z = p.2;
            w.kick(0.05);
            w.sfx.push(Sfx::Hit);
        } else {
            w.kick(0.03);
        }
    }
    fn timer(&self) -> (f32, f32) { (self.duration - self.time, self.duration) }
    fn score(&self) -> String { self.hits.to_string() }
    fn results(&self) -> Results {
        let acc = if self.shots > 0 { (100.0 * self.hits as f32 / self.shots as f32).round() as u32 } else { 0 };
        let ttk = if !self.ttks.is_empty() { self.ttks.iter().sum::<f32>() / self.ttks.len() as f32 } else { 0.0 };
        Results::new(1, "flick", self.hits.to_string(), self.hits as f32, acc, ttk)
    }
}

/* ============ TRACKING ============ */
pub struct Tracking {
    pub target: Target,
    vx: f32,
    vy: f32,
    dir_t: f32,
    time: f32,
    duration: f32,
    on_t: f32,
}

impl Tracking {
    pub fn new(duration: f32) -> Self {
        let mut t = Target::new(Kind::Track);
        t.r = 0.5;
        t.x = 0.0;
        t.y = 1.6;
        t.z = -8.0;
        Tracking { target: t, vx: 3.5, vy: 0.0, dir_t: 0.0, time: 0.0, duration, on_t: 0.0 }
    }
    fn update(&mut self, w: &mut World, dt: f32) {
        self.time += dt;
        self.dir_t -= dt;
        if self.dir_t <= 0.0 {
            self.dir_t = rnd(0.6, 1.8);
            self.vy = rnd(-2.5, 2.5);
            self.vx = rnd(3.0, 6.0) * if rnd(0.0, 1.0) < 0.5 { -1.0 } else { 1.0 };
        }
        let t = &mut self.target;
        t.x += self.vx * dt;
        t.y += self.vy * dt;
        if t.x.abs() > 8.0 { self.vx = -t.x.signum() * self.vx.abs(); }
        if t.y < 0.4 || t.y > 3.2 { self.vy = -self.vy; }

        // dang ngam trung? ray 3D tu camera toi cau
        let (dx, dy, dz) = w.forward();
        let (fx, fy, fz) = (w.pose.pos[0], w.pose.pos[1], w.pose.pos[2]);
        if let Some(th) = math::ray_sphere(fx, fy, fz, dx, dy, dz, t.x, t.y, t.z, t.r) {
            if th < w.block_forward_dist(500.0) {
                self.on_t += dt;
            }
        }
    }
    fn timer(&self) -> (f32, f32) { (self.duration - self.time, self.duration) }
    fn score(&self) -> String {
        format!("{}%", (100.0 * self.on_t / self.time.max(0.01)).round() as u32)
    }
    fn results(&self) -> Results {
        let pct = (100.0 * self.on_t / self.duration).round() as u32;
        Results::new(2, "tracking", format!("{}%", pct), pct as f32, pct, 0.0)
    }
}

/* ============ RECOIL ============ */
fn make_pattern(mag: usize, spread: f32) -> Vec<[f32; 2]> {
    let mut pts = Vec::with_capacity(mag);
    let (mut x, mut y) = (0.0f32, 0.0f32);
    for i in 0..mag {
        if i < 4 {
            y += 0.10;
        } else if i < 9 {
            y += 0.28;
        } else if i < 14 {
            y += 0.05;
            x += 0.16;
        } else {
            x += if i % 4 < 2 { 0.12 } else { -0.10 };
            y += 0.03;
        }
        pts.push([x * spread, y * spread]);
    }
    pts
}

pub struct Recoil {
    time: f32,
    duration: f32,
    shots: u32,
    mag: usize,
    pts: Vec<[f32; 2]>,
    fired: usize,
    cooldown: f32,
    reload_t: Option<f32>,
}

impl Recoil {
    pub fn new(duration: f32, mag: usize, spread: f32) -> Self {
        Recoil { time: 0.0, duration, shots: 0, mag, pts: make_pattern(mag, spread * 0.5), fired: 0, cooldown: 0.0, reload_t: None }
    }
    fn fire(&mut self, w: &mut World) {
        self.fired += 1;
        self.shots += 1;
        let pt = self.pts[self.fired - 1];
        w.kick(0.15);
        w.sfx.push(Sfx::Shoot);
        w.wall_hits.push([pt[0] + rnd(-0.03, 0.03), 1.6 + pt[1] + rnd(-0.03, 0.03)]);
    }
    fn update(&mut self, w: &mut World, dt: f32, mouse_down: bool) {
        self.time += dt;
        self.cooldown -= dt;
        if self.fired < self.mag && mouse_down && self.cooldown <= 0.0 {
            self.cooldown = 0.095;
            self.fire(w);
        }
        if self.fired >= self.mag && self.reload_t.is_none() { self.reload_t = Some(0.6); }
        if let Some(rt) = self.reload_t.as_mut() {
            *rt -= dt;
            if *rt <= 0.0 {
                self.fired = 0;
                self.reload_t = None;
                w.wall_hits.clear();
            }
        }
    }
    fn timer(&self) -> (f32, f32) { (self.duration - self.time, self.duration) }
    fn score(&self) -> String { format!("{}/{}", self.mag - self.fired, self.mag) }
    fn results(&self) -> Results {
        Results::new(3, "recoil", self.shots.to_string(), self.shots as f32, 0, 0.0)
    }
}

/* ============ BOT DUEL (port bots.rs) ============ */
pub struct Keys {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub sprint: bool,
    pub crouch: bool,
    pub jump: bool,
}

pub struct Duel {
    pub bot: Target,
    respawn_t: f32,
    vx: f32,
    dir_t: f32,
    peeking: bool,
    peek_t: f32,
    fire_t: f32,
    time: f32,
    duration: f32,
    shots: u32,
    player_hp: f32,
    player_respawn_t: f32,
    dead_msg: f32,
    speed: f32,
    fire_cd: f32,
    miss_rate: f32,
    dmg: f32,
    kills: u32,
    deaths: u32,
    dust_t: f32,
}

impl Duel {
    pub fn new(duration: f32, difficulty: usize, bot_hp: f32) -> Self {
        let (speed, fire_cd, miss_rate, dmg) = match difficulty {
            0 => (2.2, 2.0, 0.55, 18.0),
            1 => (3.2, 1.5, 0.35, 26.0),
            _ => (4.2, 1.1, 0.22, 32.0),
        };
        let mut bot = Target::new(Kind::Bot);
        bot.x = 0.0;
        bot.y = 1.05;
        bot.z = -10.0;
        bot.hp = bot_hp;
        bot.max_hp = bot_hp;
        Duel {
            bot, respawn_t: 0.0, vx: speed, dir_t: 0.6, peeking: false, peek_t: 1.5,
            fire_t: fire_cd * 1.2, time: 0.0, duration, shots: 0, player_hp: 100.0,
            player_respawn_t: 0.0, dead_msg: 0.0, speed, fire_cd, miss_rate, dmg,
            kills: 0, deaths: 0, dust_t: 0.0,
        }
    }

    fn update(&mut self, w: &mut World, dt: f32, keys: &Keys) {
        self.time += dt;
        if self.time <= dt + 1e-6 {
            w.pose.pos = [0.0, 1.6, 5.0];
            w.pose.feet = 0.0;
            w.pose.vy = 0.0;
            w.pose.eye_h = crate::core::arena::EYE_STAND;
            w.pose.yaw = 0.0;
            w.pose.pitch = 0.0;
        }
        let sy = w.pose.yaw.to_radians().sin();
        let cy = w.pose.yaw.to_radians().cos();
        let (mut wx, mut wz) = (0.0f32, 0.0f32);
        if keys.w { wx += -sy; wz += -cy; }
        if keys.s { wx += sy; wz += cy; }
        if keys.d { wx += cy; wz += -sy; }
        if keys.a { wx += -cy; wz += sy; }
        let l = (wx * wx + wz * wz).sqrt();
        if l > 0.0 { wx /= l; wz /= l; }
        let crouch = keys.crouch;
        let speed = if crouch { 2.2 } else if keys.sprint { 6.2 } else { 4.0 };
        crate::core::player::step_player(&mut w.pose, dt, wx, wz, speed, crouch, keys.jump);
        if self.bot.alive {
            let dxb = w.pose.pos[0] - self.bot.x;
            let dzb = w.pose.pos[2] - self.bot.z;
            let d2b = dxb * dxb + dzb * dzb;
            if d2b < 0.64 && d2b > 1e-6 {
                let db = d2b.sqrt();
                w.pose.pos[0] = self.bot.x + dxb / db * 0.8;
                w.pose.pos[2] = self.bot.z + dzb / db * 0.8;
            }
        }
        self.dust_t += dt;
        if self.dust_t > 0.12 {
            self.dust_t = 0.0;
            w.burst(w.pose.pos[0] + rnd(-6.0, 6.0), rnd(0.3, 2.5), w.pose.pos[2] + rnd(-6.0, 6.0), 1, 0.15, 3, 2.5);
        }
        if self.bot.hit_flash > 0.0 { self.bot.hit_flash -= dt; }

        if self.player_respawn_t > 0.0 {
            self.player_respawn_t -= dt;
            if self.player_respawn_t <= 0.0 {
                self.player_hp = 100.0;
                w.pose.pos = [0.0, 1.6, 5.0];
                w.pose.feet = 0.0;
                w.pose.vy = 0.0;
            }
        }

        if self.bot.alive {
            self.peek_t -= dt;
            if self.peek_t <= 0.0 {
                if self.peeking {
                    self.peeking = false;
                    self.peek_t = rnd(1.0, 2.2);
                    self.vx = self.speed * if rnd(0.0, 1.0) < 0.5 { -1.0 } else { 1.0 };
                } else {
                    self.peeking = true;
                    self.peek_t = rnd(0.55, 0.9);
                }
            }
            self.dir_t -= dt;
            if self.dir_t <= 0.0 {
                self.dir_t = rnd(0.4, 1.1);
                self.vx = self.speed * if rnd(0.0, 1.0) < 0.5 { -1.0 } else { 1.0 };
            }
            self.bot.x += self.vx * dt * if self.peeking { 0.35 } else { 1.0 };
            if self.bot.x.abs() > 5.0 { self.vx = -self.bot.x.signum() * self.vx.abs(); }

            self.fire_t -= dt;
            if self.fire_t <= 0.0 && self.peeking && self.peek_t < 0.5 && self.player_respawn_t <= 0.0 {
                self.fire_t = self.fire_cd;
                let blocked = math::segment_blocked(self.bot.x, 1.3, self.bot.z,
                    w.pose.pos[0], w.pose.pos[1], w.pose.pos[2]);
                if !blocked {
                    w.add_tracer(self.bot.x, self.bot.y, self.bot.z,
                        w.pose.pos[0], w.pose.pos[1], w.pose.pos[2]);
                    let dxp = self.bot.x - w.pose.pos[0];
                    let dzp = self.bot.z - w.pose.pos[2];
                    let dist = (dxp * dxp + dzp * dzp).sqrt();
                    let air = w.pose.feet > math::ground_height(w.pose.pos[0], w.pose.pos[2], w.pose.feet) + 0.05;
                    let mut miss = self.miss_rate + dist * 0.012;
                    if air { miss += 0.12; }
                    if w.pose.psprint { miss += 0.12; }
                    else if w.pose.pmove { miss += 0.05; }
                    if w.pose.pcrouch { miss += 0.06; }
                    if miss > 0.85 { miss = 0.85; }
                    if rnd(0.0, 1.0) > miss {
                        self.player_hp -= self.dmg;
                        w.hurt = 0.6;
                        w.kick(0.35);
                        if self.player_hp <= 0.0 {
                            self.player_respawn_t = 1.4;
                            self.deaths += 1;
                            self.dead_msg = 1.2;
                            w.hurt = 1.0;
                            w.burst(w.pose.pos[0], w.pose.pos[1], w.pose.pos[2], 16, 3.5, 0, 0.6);
                            w.kick(1.2);
                        }
                    }
                }
            }
        } else {
            self.respawn_t -= dt;
            if self.respawn_t <= 0.0 {
                self.bot.alive = true;
                self.bot.hp = self.bot.max_hp;
                self.bot.x = rnd(-4.0, 4.0);
                self.bot.z = -10.0 + rnd(-1.5, 1.5);
                self.peek_t = 1.0;
            }
        }
        if self.dead_msg > 0.0 { self.dead_msg -= dt; }
    }

    fn on_mousedown(&mut self, w: &mut World) {
        if !self.bot.alive { return; }
        self.shots += 1;
        w.kick(0.12);
        w.sfx.push(Sfx::Shoot);
        let (dx, dy, dz) = w.forward();
        let (fx, fy, fz) = (w.pose.pos[0], w.pose.pos[1], w.pose.pos[2]);
        w.burst(fx + dx * 0.6, fy + dy * 0.6, fz + dz * 0.6, 2, 1.2, 2, 0.2);
        let t_block = w.block_forward_dist(500.0);
        let t_body = math::ray_sphere(fx, fy, fz, dx, dy, dz, self.bot.x, crate::core::arena::BOT_BODY_Y, self.bot.z, crate::core::arena::BOT_BODY_R);
        let t_head = math::ray_sphere(fx, fy, fz, dx, dy, dz, self.bot.x, crate::core::arena::BOT_HEAD_Y, self.bot.z, crate::core::arena::BOT_HEAD_R);
        let hit = match (t_body, t_head) {
            (Some(b), Some(h)) => Some(if h < b { (h, true) } else { (b, false) }),
            (Some(b), None) => Some((b, false)),
            (None, Some(h)) => Some((h, true)),
            (None, None) => None,
        };
        match hit {
            Some((th, head)) if th < t_block => {
                w.add_tracer(fx, fy, fz, self.bot.x, self.bot.y + 0.3, self.bot.z);
                let dmg = if head { DMG_HEAD } else { DMG_BODY };
                // headshot: chet luon
                if head {
                    self.bot.hp = 0.0;
                } else {
                    self.bot.hp -= dmg;
                }
                self.bot.hit_flash = 0.08;
                if w.fx_hit { w.hitmark = 0.18; }
                let hy = if head { crate::core::arena::BOT_HEAD_Y } else { crate::core::arena::BOT_BODY_Y };
                if w.fx_dmg { w.add_dmg(self.bot.x, hy, self.bot.z, dmg, head); }
                w.burst(self.bot.x, hy, self.bot.z, 8, 3.0, if head { 2 } else { 0 }, 0.4);
                w.sfx.push(if head { Sfx::Head } else { Sfx::Hit });
                self.vx = -self.vx;
                if self.bot.hp <= 0.0 {
                    self.bot.alive = false;
                    self.respawn_t = 0.6;
                    self.kills += 1;
                    w.kill_banner = 1.0;
                    w.burst(self.bot.x, 1.0, self.bot.z, 22, 4.5, 1, 0.7);
                    w.sfx.push(Sfx::Kill);
                }
            }
            _ => {
                if t_block < 500.0 {
                    w.add_tracer(fx, fy, fz, fx + dx * t_block, fy + dy * t_block, fz + dz * t_block);
                } else {
                    w.add_tracer(fx, fy, fz, fx + dx * 60.0, fy + dy * 60.0, fz + dz * 60.0);
                }
            }
        }
    }

    fn timer(&self) -> (f32, f32) { (self.duration - self.time, self.duration) }
    fn score(&self) -> String { self.kills.to_string() }
    fn hp(&self) -> f32 { if self.player_respawn_t > 0.0 { 0.0 } else { self.player_hp } }
    fn show_msg(&self) -> bool { self.player_respawn_t > 0.0 }
    fn results(&self) -> Results {
        let mut r = Results::new(4, "duel", self.kills.to_string(), self.kills as f32, 0, 0.0);
        r.deaths = self.deaths;
        r.hp_left = self.player_hp.max(0.0).round() as u32;
        r.shots = self.shots;
        r
    }
}
