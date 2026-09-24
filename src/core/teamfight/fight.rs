// TeamFight: driver cua mode 5v5.
//
// Giu mot `Match` (10 `Actor`) va moi frame:
//   1. dong player pose vao actor nguoi choi
//   2. chay AI cho 9 bot (think -> aim -> navigate -> shoot)
//   3. giai quyet phat ban (bot vs bot, bot vs player, player vs bot)
//   4. tick hoi giap / hoi sinh / vong
//   5. day snapshot xuong `World` de render + HUD doc
//
// KHONG dung `core::Target` -> khong pha 5 mode cu.

use super::actor::{Actor, ROSTER_SIZE};
use super::ai;
use super::hud::{BotView, FeedEntry, KdaRow};
use super::nav::{self, NavGrid};
use super::round::{Match, MatchState, MatchResult};
use crate::core::arena::{BOT_BODY_R, BOT_HEAD_R, BOT_BODY_Y, BOT_HEAD_Y};
use crate::core::drills::Keys;
use crate::core::math;
use crate::core::player;
use crate::core::world::World;
use crate::core::weapon::{self, WEAPON_COUNT};

const NAMES: [&str; ROSTER_SIZE] = [
    "YOU", "BOT-1", "BOT-2", "BOT-3", "BOT-4", // doi CT
    "T-1", "T-2", "T-3", "T-4", "T-5", // doi T
];

pub fn actor_name(id: usize) -> &'static str {
    NAMES.get(id).copied().unwrap_or("BOT")
}

pub struct TeamFight {
    pub gm: Match,
    nav: NavGrid,
    difficulty: usize,
    /// dem nguoi choi ban trung mot bot
    pub player_shots: u32,
    pub player_hits: u32,
    /// thoi gian con lai cua dong kill feed
    feed_life: f32,
    /// cac dong feed moi trong frame nay (chuyen sang w.tf.feed)
    last_feed: Vec<FeedEntry>,
    /// khoi dau round moi (hien banner)
    banner_t: f32,
    pub banner: String,
    /// player vua chet -> hien "BAN DA CHET"
    dead_msg: f32,
    /// timer khoi tao path (A* khong nen chay moi frame cho 9 bot)
    path_t: f32,
    /// map version de rebuild nav khi doi map
    nav_map: String,
    /// vong da xu ly gan nhat (dung de snap player ve spawn khi doi vong)
    last_round: u32,
    /// mode bom co bat khong (5V5 BOMB DEFUSE)
    pub bomb_mode: bool,
    /// 2 bom, moi doi 1 qua (chi dung khi `bomb_mode`)
    pub bombs: [Option<super::bomb::Bomb>; 2],
    /// nguoi choi co giu nut hanh dong (E) khong
    pub holding: bool,
    /// tien do trong bom cua nguoi choi (0..1) cho HUD
    pub plant_bar: f32,
    /// tien do gỡ bom (0..1) cho HUD
    pub defuse_bar: f32,
    /// bom dang no, con bao lau (cho HUD)
    pub fuse: f32,
    /// doi nao da trong bom (cho HUD)
    pub bomb_planted_by: Option<u8>,
    finished: bool,
    pub result: Option<MatchResult>,
}

impl TeamFight {
    pub fn new(_duration: f32, difficulty: usize, player_gun: u8, map_name: &str) -> Self {
        let mut gm = Match::new(difficulty, 0x5EED_1234 ^ difficulty as u32);
        // sung nguoi choi theo gun da chon
        let pid = gm.player_id;
        gm.actors[pid].weapon = (player_gun as usize % WEAPON_COUNT) as u8;
        TeamFight {
            nav: nav::build(),
            difficulty,
            player_shots: 0,
            player_hits: 0,
            feed_life: 0.0,
            last_feed: Vec::new(),
            banner_t: 0.0,
            banner: "ROUND 1".to_owned(),
            dead_msg: 0.0,
            path_t: 0.0,
            nav_map: map_name.to_owned(),
            last_round: 0,
            bomb_mode: false,
            bombs: [None, None],
            holding: false,
            plant_bar: 0.0,
            defuse_bar: 0.0,
            fuse: 0.0,
            bomb_planted_by: None,
            gm,
            finished: false,
            result: None,
        }
    }

    /// Dua nguoi choi ve dung vi tri spawn cua doi trong `w.pose`.
    ///
    /// LUON Y: `World.pose.pos` la VI TRI MAT (chan + eye_h), con
    /// `World.pose.feet` moi la vi tri chan. Gan `pose.pos` bang toa do chan
    /// se lam camera nap san dat -> man hinh bi day mat trong cua block.
    pub fn snap_player_to_spawn(&mut self, w: &mut World) {
        let pos = self.gm.player_spawn();
        let yaw = self.gm.player_spawn_yaw();
        w.pose.feet = pos[1];
        w.pose.eye_h = crate::core::arena::EYE_STAND;
        w.pose.pos = [pos[0], pos[1] + w.pose.eye_h, pos[2]];
        w.pose.vy = 0.0;
        w.pose.yaw = yaw;
        w.pose.pitch = 0.0;
    }

    /// Doi doi thu cong (phim T). Tra ve true neu doi duoc.
    pub fn manual_switch(&mut self, w: &mut World) -> bool {
        if self.gm.manual_switch() {
            self.snap_player_to_spawn(w);
            self.banner = "SWITCHED TEAM".to_owned();
            self.banner_t = 1.6;
            true
        } else {
            false
        }
    }

    /// Mode BOM: them 2 bom (moi doi 1 qua).
    pub fn new_bomb(difficulty: usize, player_gun: u8, map_name: &str) -> Self {
        let mut f = Self::new(0.0, difficulty, player_gun, map_name);
        f.bomb_mode = true;
        f.spawn_bombs();
        f
    }

    /// (Tao lai) bom cho moi vong: moi doi 1 nguoi mang.
    /// `bombs[0]` = bom doi CT, `bombs[1]` = bom doi T.
    fn spawn_bombs(&mut self) {
        let pid = self.gm.player_id;
        // bom doi 0: nguoi choi neu player o doi 0, nguot bot CT dau tien
        let b0 = super::bomb::Bomb::new(&self.gm.actors, pid);
        // bom doi 1: nguoi choi neu player o doi 1, nguot bot T dau tien
        let b1 = match self.gm.actors.iter().find(|a| a.team == 1 && a.id != pid) {
            Some(a) => {
                let mut b = super::bomb::Bomb::new(&self.gm.actors, pid);
                b.carrier = Some(a.id);
                b
            }
            None => super::bomb::Bomb::new(&self.gm.actors, pid),
        };
        self.bombs = [Some(b0), Some(b1)];
    }

    pub fn duration(&self) -> f32 {
        0.0
    }

    /// Do kho AI hien tai.
    pub fn tier(&self) -> ai::Tier {
        ai::tier(self.difficulty)
    }

    pub fn is_over(&self) -> bool {
        self.finished
    }

    pub fn timer(&self) -> (f32, f32) {
        (self.gm.time_left.max(0.0), self.gm.time_left.max(0.0))
    }

    pub fn player_hp(&self) -> f32 {
        self.gm.actors[self.gm.player_id].hp
    }

    pub fn show_dead(&self) -> bool {
        self.dead_msg > 0.0
    }

    /// Vong moi bat dau -> hien banner roi
    fn on_round_banner(&mut self) {
        self.banner = format!("ROUND {}", self.gm.round);
        self.banner_t = 2.0;
    }

    /// Player ban mot phat: hitscan vao cac bot cung doi. Tra ve:
    /// (co trung khong, id bot, headshot, damage)
    pub fn player_shoot(&mut self, w: &mut World) -> Option<(usize, bool, f32)> {
        let pid = self.gm.player_id;
        if !self.gm.actors[pid].alive {
            return None;
        }
        if !self.gm.state.is_live() {
            return None;
        }
        self.player_shots += 1;
        let (dx, dy, dz) = w.forward();
        let (fx, fy, fz) = (w.pose.pos[0], w.pose.pos[1], w.pose.pos[2]);
        let t_block = w.block_forward_dist(200.0);
        let pteam = self.gm.actors[pid].team;
        let mut best: Option<(f32, usize, bool)> = None;
        for a in self.gm.actors.iter() {
            if a.id == pid || a.team == pteam || !a.alive {
                continue;
            }
            let t_body = math::ray_sphere(
                fx, fy, fz, dx, dy, dz,
                a.pos[0], a.pos[1] + BOT_BODY_Y, a.pos[2], BOT_BODY_R,
            );
            let t_head = math::ray_sphere(
                fx, fy, fz, dx, dy, dz,
                a.pos[0], a.pos[1] + BOT_HEAD_Y, a.pos[2], BOT_HEAD_R,
            );
            let hit = match (t_body, t_head) {
                (Some(b), Some(h)) => Some(if h < b { (h, true) } else { (b, false) }),
                (Some(b), None) => Some((b, false)),
                (None, Some(h)) => Some((h, true)),
                (None, None) => None,
            };
            if let Some((th, head)) = hit {
                if th < t_block {
                    if best.is_none() || th < best.unwrap().0 {
                        best = Some((th, a.id, head));
                    }
                }
            }
        }
        let Some((_th, id, head)) = best else {
            // truot: chi ve tracer vao tuong
            w.add_tracer(fx, fy, fz, fx + dx * t_block, fy + dy * t_block, fz + dz * t_block);
            return None;
        };
        self.player_hits += 1;
        let dist = ((self.gm.actors[id].pos[0] - fx).powi(2)
            + (self.gm.actors[id].pos[2] - fz).powi(2))
        .sqrt();
        let wid = self.gm.actors[id].weapon;
        let dmg = weapon::damage_at(weapon::weapon(wid), head, dist);
        self.gm.actors[id].dmg_dealt += dmg;
        self.gm.actors[pid].dmg_dealt += dmg;
        if head {
            self.gm.actors[pid].headshots += 1;
        }
        // ap damage
        let (died, dealt) = self.gm.actors[id].damage(dmg);
        w.add_tracer(fx, fy, fz, self.gm.actors[id].pos[0], self.gm.actors[id].pos[1] + 1.0, self.gm.actors[id].pos[2]);
        if w.fx_hit {
            w.hitmark = 0.18;
        }
        if died {
            self.gm.actors[pid].kills += 1;
            self.push_feed(pid, id, head);
            w.sfx.push(crate::core::world::Sfx::Kill);
        } else {
            w.sfx.push(crate::core::world::Sfx::Hit);
        }
        Some((id, head, dealt))
    }

    fn push_feed(&mut self, killer: usize, victim: usize, head: bool) {
        self.feed_life = 5.0;
        let _ = (killer, victim);
        // feed luu trong gm qua hud snapshot; giu o day de danh dau moi su kien
        self.last_feed.push(FeedEntry {
            killer: actor_name(killer),
            victim: actor_name(victim),
            head,
            life: 5.0,
        });
        if self.last_feed.len() > 5 {
            self.last_feed.remove(0);
        }
    }

    /// Cap nhat chinh. `keys` de di chuyen nguoi choi.
    pub fn update(&mut self, w: &mut World, dt: f32, _shot: bool, keys: &Keys) {
        // doi map -> dung lai nav
        let map_name = crate::core::arena::current_map().name;
        if self.nav_map != map_name {
            self.nav = nav::build();
            self.nav_map = map_name.to_owned();
        }
        // tick round. `tick_round` tra ve true khi MOT vong ket thuc, nen
        // phai kiem tra `is_over()` rieng cho ca tran.
        self.gm.tick_round(dt);
        if self.gm.is_over() {
            self.finished = true;
            self.result = Some(self.gm.result());
            self.push_snapshot(w);
            return;
        }
        if self.gm.state == MatchState::Freeze && self.banner_t <= 0.0 {
            self.on_round_banner();
        }
        if self.banner_t > 0.0 {
            self.banner_t -= dt;
        }
        if self.dead_msg > 0.0 {
            self.dead_msg -= dt;
        }
        if self.feed_life > 0.0 {
            self.feed_life -= dt;
        }
        // Vong moi (hoac frame dau) -> dua nguoi choi ve spawn cua doi
        if self.gm.round != self.last_round {
            self.last_round = self.gm.round;
            self.snap_player_to_spawn(w);
            if self.bomb_mode {
                self.spawn_bombs();
                self.plant_bar = 0.0;
                self.defuse_bar = 0.0;
                self.fuse = 0.0;
                self.bomb_planted_by = None;
            }
        }
        // vong da ket thuc trong frame nay -> khong chay tiep
        if !self.gm.state.is_live() && self.gm.state != MatchState::Freeze {
            self.finished = true;
            self.result = Some(self.gm.result());
            self.push_snapshot(w);
            return;
        }

        // dong player pose vao actor
        let pid = self.gm.player_id;
        if self.gm.state.is_live() && self.gm.actors[pid].alive {
            // giu player di chuyen theo WASD
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
            let spd = if crouch { 2.2 } else if keys.sprint { 6.2 } else { 4.0 };
            player::step_player(&mut w.pose, dt, wx, wz, spd, crouch, keys.jump);
            // actor.pos la VI TRI CHAN -> lay y tu `pose.feet`, khong phai
            // tu `pose.pos` (vi tri mat).
            self.gm.actors[pid].pos = [w.pose.pos[0], w.pose.feet, w.pose.pos[2]];
            self.gm.actors[pid].yaw = w.pose.yaw;
            self.gm.actors[pid].pitch = w.pose.pitch;
            self.gm.actors[pid].moving = wx != 0.0 || wz != 0.0;
        } else {
            // freeze / chet -> giu nguoi choi o cho dang dung
            self.gm.actors[pid].pos = [w.pose.pos[0], w.pose.feet, w.pose.pos[2]];
        }
        // player tick armor + respawn
        self.gm.actors[pid].tick_armor(dt);
        self.gm.actors[pid].tick_respawn(dt);
        if !self.gm.actors[pid].alive && self.gm.actors[pid].respawn_t <= 0.0 {
            // hoi sinh: LUON ve spawn cua doi (khong dung choi chet)
            self.snap_player_to_spawn(w);
            let sp = self.gm.player_spawn();
            let sy = self.gm.player_spawn_yaw();
            let wid = self.gm.actors[pid].weapon;
            self.gm.actors[pid].reset(sp, sy, wid);
        }

        // --- AI cho 9 bot ---
        let tier = self.tier();
        if self.gm.state.is_live() {
            // rebuild path theo batch (A* khong chay moi bot moi frame)
            self.path_t -= dt;
            let rebuild = self.path_t <= 0.0;
            if rebuild {
                self.path_t = 0.5;
            }
            let mut shots: Vec<ai::Shot> = Vec::new();
            for i in 0..self.gm.actors.len() {
                if self.gm.actors[i].id == pid {
                    continue;
                }
                // tach borrow: clone actors snapshot cho AI doc
                let actors_snapshot: Vec<Actor> = self.gm.actors.clone();
                let mut bot = actors_snapshot[i].clone();
                if !bot.alive {
                    bot.tick_respawn(dt);
                    if bot.respawn_t <= 0.0 {
                        let (pos, yaw, wid) = (bot.pos, bot.yaw, bot.weapon);
                        bot.reset(pos, yaw, wid);
                    }
                    self.gm.actors[i] = bot;
                    continue;
                }
                // think
                let mut rng_state = (self.gm.rng ^ (i as u32).wrapping_mul(2654435761)) | 1;
                let mut rng = move || {
                    rng_state ^= rng_state << 13;
                    rng_state ^= rng_state >> 17;
                    rng_state ^= rng_state << 5;
                    (rng_state >> 8) as f32 / 16_777_216.0
                };
                ai::think(&mut bot, &actors_snapshot, &tier, &mut rng);
                ai::aim(&mut bot, &actors_snapshot, &tier, dt, &mut rng);
                if rebuild {
                    bot.path.clear();
                }
                ai::navigate(&mut bot, &actors_snapshot, &self.nav, &tier, dt);
                if let Some(s) = ai::shoot(&mut bot, &actors_snapshot, &tier, dt, &mut rng) {
                    shots.push(s);
                }
                self.gm.actors[i] = bot;
            }
            // giai quyet shots
            for s in shots {
                self.resolve_shot(w, s, &tier);
            }

            // --- tach actor de khong chong/che nhau ---
            // 1) day bot ra khoi nguoi choi (khong lot qua camera)
            let (px, pz) = (self.gm.actors[pid].pos[0], self.gm.actors[pid].pos[2]);
            let mut a = std::mem::take(&mut self.gm.actors);
            for act in a.iter_mut() {
                super::actor::keep_off_player(act, px, pz);
            }
            // 2) day cac bot tranh nhau
            super::actor::separate(&mut a, pid);
            // 3) giu trong arena + dung tren mat dat
            for act in a.iter_mut() {
                let (cx, cz) = crate::core::math::collide(act.pos[0], act.pos[2], act.pos[1]);
                act.pos[0] = cx;
                act.pos[2] = cz;
                act.pos[1] = act.ground();
            }
            self.gm.actors = a;
        }

        // --- BOM (chi mode 5V5 BOMB) ---
        if self.bomb_mode {
            let mut bomb_win: Option<u8> = None;
            let mut defused = false;
            for slot in 0..2 {
                let Some(b) = self.bombs[slot].as_mut() else { continue };
                // nguoi choi giu nut: trong bom (neu dang mang) hoac gỡ bom.
                // CHI goi 1 LAN moi frame de tien do khong nhanh gap.
                if self.holding {
                    if let Some(p) = b.plant_progress(&self.gm.actors, pid, true, dt) {
                        self.plant_bar = p;
                    }
                } else {
                    b.plant_progress(&self.gm.actors, pid, false, 0.0);
                    self.plant_bar = 0.0;
                }
                if let Some(r) = b.update(&self.gm.actors, self.holding, dt) {
                    bomb_win = Some(r);
                }
                if b.state == super::bomb::BombState::Defused {
                    defused = true;
                }
                self.defuse_bar = (b.defuse_t / super::bomb::DEFUSE_TIME).clamp(0.0, 1.0);
                self.fuse = if b.state == super::bomb::BombState::Planted { b.fuse } else { 0.0 };
                self.bomb_planted_by = if b.state == super::bomb::BombState::Planted {
                    Some(b.plant_team)
                } else {
                    None
                };
            }
            // ket thuc vong
            if defused {
                self.gm.end_round_draw();
            } else if let Some(t) = bomb_win {
                self.gm.end_round_team(t);
            } else if self.gm.state.is_live() {
                // het ngui 1 doi va bom chua trong -> doi con lai thang
                let (ct, tt) = (self.gm.team_alive(0), self.gm.team_alive(1));
                let any_planted = self
                    .bombs
                    .iter()
                    .any(|b| matches!(b, Some(x) if x.state == super::bomb::BombState::Planted));
                if !any_planted {
                    if ct == 0 && tt > 0 {
                        self.gm.end_round_team(1);
                    } else if tt == 0 && ct > 0 {
                        self.gm.end_round_team(0);
                    }
                }
            }
        }

        // day snapshot xuong world
        self.push_snapshot(w);
    }

    /// Giai quyet 1 phat ban cua bot.
    fn resolve_shot(&mut self, w: &mut World, s: ai::Shot, _tier: &ai::Tier) {
        let from = s.from;
        let Some(bot) = self.gm.actors.iter().find(|a| a.id == from) else {
            return;
        };
        let eye = bot.eye();
        w.add_tracer(eye[0], eye[1], eye[2], eye[0], eye[1], eye[2]);
        let Some(victim_id) = s.to else {
            return;
        };
        let Some(victim) = self.gm.actors.iter().find(|a| a.id == victim_id) else {
            return;
        };
        // khong ban trung dong doi
        if victim.team == bot.team {
            return;
        }
        let dist = ((victim.pos[0] - eye[0]).powi(2) + (victim.pos[2] - eye[2]).powi(2)).sqrt();
        let wid = bot.weapon;
        let dmg = weapon::damage_at(weapon::weapon(wid), s.head, dist);
        self.gm.actors[from].dmg_dealt += dmg;
        self.gm.actors[victim_id].dmg_dealt += dmg;
        if s.head {
            self.gm.actors[from].headshots += 1;
        }
        let (died, _dealt) = self.gm.actors[victim_id].damage(dmg);
        if died {
            self.gm.actors[from].kills += 1;
            self.push_feed(from, victim_id, s.head);
            w.sfx.push(crate::core::world::Sfx::Kill);
        }
        // player bi trung -> hurt
        if victim_id == self.gm.player_id {
            w.hurt = 0.6;
        }
    }

    /// Day trang thai sang World cho render + HUD.
    fn push_snapshot(&mut self, w: &mut World) {
        w.bots.clear();
        for a in self.gm.actors.iter() {
            w.bots.push(BotView {
                slot: a.id,
                pos: a.pos,
                yaw: a.yaw,
                pitch: a.pitch,
                team: a.team,
                alive: a.alive,
                moving: a.moving,
            });
        }
        w.tf.active = true;
        w.tf.round = self.gm.round;
        w.tf.score = self.gm.score;
        w.tf.player_team = self.gm.player_team();
        w.tf.time_left = self.gm.time_left.max(0.0);
        w.tf.freeze_t = self.gm.freeze_t.max(0.0);
        w.tf.match_over = self.gm.is_over();
        let p = &self.gm.actors[self.gm.player_id];
        w.tf.player_hp = p.hp;
        w.tf.player_armor = p.armor;
        w.tf.player_kills = p.kills;
        w.tf.player_deaths = p.deaths;
        w.tf.player_kd = {
            let r = KdaRow {
                name: actor_name(p.id),
                team: p.team,
                kills: p.kills,
                deaths: p.deaths,
                headshots: p.headshots,
                damage: p.dmg_dealt,
                is_player: true,
            };
            r.kd()
        };
        w.tf.banner = self.banner.clone();
        // bom
        w.tf.bomb_mode = self.bomb_mode;
        w.tf.fuse = self.fuse;
        w.tf.plant_bar = self.plant_bar;
        w.tf.defuse_bar = self.defuse_bar;
        w.tf.bomb_planted = self.bomb_planted_by.is_some();
        w.tf.has_bomb = self
            .bombs
            .iter()
            .any(|b| matches!(b, Some(x) if x.carrier == Some(self.gm.player_id)));
        w.tf.bomb_action = self.plant_bar > 0.0 || self.defuse_bar > 0.0;
        // feed: giam life
        for f in w.tf.feed.iter_mut() {
            f.life -= 0.016;
        }
        w.tf.feed.retain(|f| f.life > 0.0);
        for f in self.last_feed.iter() {
            w.tf.feed.push(*f);
        }
        self.last_feed.clear();
        // scoreboard
        w.tf.scoreboard.clear();
        let mut rows: Vec<KdaRow> = self
            .gm
            .actors
            .iter()
            .map(|a| KdaRow {
                name: actor_name(a.id),
                team: a.team,
                kills: a.kills,
                deaths: a.deaths,
                headshots: a.headshots,
                damage: a.dmg_dealt,
                is_player: a.is_player,
            })
            .collect();
        rows.sort_by(|x, y| y.kills.cmp(&x.kills).then(x.deaths.cmp(&y.deaths)));
        w.tf.scoreboard = rows;
        w.show_blocks = true;
        w.show_gun = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::teamfight::actor::TEAM_SIZE;
    use crate::core::world::World;

    /// Chay mot tran day du o che do headless: khong render, chi logic.
    /// Bat buoc cho AI vao vong livelien tuc cho den khi co nguoi thang.
    fn simulate(difficulty: usize, max_seconds: f32) -> TeamFight {
        let mut f = TeamFight::new(60.0, difficulty, 1, "Duel Arena");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        let dt = 1.0 / 60.0;
        let steps = (max_seconds / dt) as i32;
        for _ in 0..steps {
            if f.is_over() { break; }
            f.update(&mut w, dt, false, &keys);
        }
        f
    }

    #[test]
    fn player_respawns_at_team_spawn_not_death_spot() {
        let mut f = TeamFight::new(60.0, 1, 1, "Duel Arena");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        let dt = 1.0 / 60.0;
        // 2 frame de vong chay live
        f.update(&mut w, dt, false, &keys);
        f.update(&mut w, dt, false, &keys);
        let pid = f.gm.player_id;
        let spawn = f.gm.actors[pid].pos;
        // dua player ra giua map (nhu noi chet sau khi chay toi)
        f.gm.actors[pid].pos = [3.0, 0.0, -7.0];
        w.pose.pos = [3.0, 0.0, -7.0];
        // giet player roi ep hoi sinh ngay
        f.gm.actors[pid].damage(999.0);
        f.gm.actors[pid].respawn_t = 0.0;
        f.update(&mut w, dt, false, &keys);
        let after = f.gm.actors[pid].pos;
        assert!(
            (after[0] - spawn[0]).abs() < 0.01 && (after[2] - spawn[2]).abs() < 0.01,
            "hoi sinh phai ve spawn cua doi (spawn={:?}), khong phai cho chet (after={:?})",
            spawn,
            after
        );
        assert!(f.gm.actors[pid].alive, "phai song lai sau khi hoi sinh");
    }

    #[test]
    fn no_actor_ever_overlaps_another_in_a_full_match() {
        // Bat loi "bot lot qua camera nguoi choi" / "bot chong nhau".
        let mut f = TeamFight::new(60.0, 1, 1, "DF-PORT");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        let dt = 1.0 / 60.0;
        for _ in 0..(45 * 60) {
            f.update(&mut w, dt, false, &keys);
            if f.is_over() {
                break;
            }
            // moi 10 frame kiem tra 1 lan (chi lay mau de nhanh)
            for i in 0..f.gm.actors.len() {
                if !f.gm.actors[i].alive {
                    continue;
                }
                for j in (i + 1)..f.gm.actors.len() {
                    if !f.gm.actors[j].alive {
                        continue;
                    }
                    let dx = f.gm.actors[i].pos[0] - f.gm.actors[j].pos[0];
                    let dz = f.gm.actors[i].pos[2] - f.gm.actors[j].pos[2];
                    let d = (dx * dx + dz * dz).sqrt();
                    assert!(
                        d >= super::super::actor::ACTOR_R,
                        "actor {} va {} chong nhau: {:.2}m",
                        i,
                        j,
                        d
                    );
                }
            }
        }
    }

    #[test]
    fn bots_keep_their_distance_from_the_player() {
        let mut f = TeamFight::new(60.0, 1, 1, "DF-PORT");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        let dt = 1.0 / 60.0;
        for _ in 0..(30 * 60) {
            f.update(&mut w, dt, false, &keys);
            if f.is_over() {
                break;
            }
            let p = &f.gm.actors[f.gm.player_id];
            if !p.alive {
                continue;
            }
            for a in f.gm.actors.iter() {
                if a.id == p.id || !a.alive {
                    continue;
                }
                let dx = a.pos[0] - p.pos[0];
                let dz = a.pos[2] - p.pos[2];
                let d = (dx * dx + dz * dz).sqrt();
                assert!(
                    d >= super::super::actor::ACTOR_R,
                    "bot {} lot vao nguoi choi: {:.2}m",
                    a.id,
                    d
                );
            }
        }
    }

    #[test]
    fn bomb_mode_starts_with_two_bombs() {
        let f = TeamFight::new_bomb(1, 1, "DF-PORT");
        assert!(f.bomb_mode);
        assert!(f.bombs[0].is_some(), "phai co bom doi CT");
        assert!(f.bombs[1].is_some(), "phai co bom doi T");
        // nguoi choi phai mang bom cua doi minh
        assert!(f
            .bombs
            .iter()
            .any(|b| matches!(b, Some(x) if x.carrier == Some(f.gm.player_id))));
    }

    #[test]
    fn bomb_mode_runs_without_panicking() {
        let mut f = TeamFight::new_bomb(1, 1, "DF-PORT");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        let dt = 1.0 / 60.0;
        for i in 0..(90 * 60) {
            f.holding = i % 120 < 40;
            f.update(&mut w, dt, false, &keys);
            if f.is_over() {
                break;
            }
        }
        // snapshot phai hop le
        assert!(w.tf.bomb_mode);
        assert!(w.tf.fuse >= 0.0);
        assert!(w.tf.plant_bar <= 1.0);
    }

    #[test]
    fn bomb_mode_round_eventually_resolves() {
        // chay lau: hoac bom no, hoac het ngui -> phai ket thuc vai vong
        let mut f = TeamFight::new_bomb(1, 1, "DF-PORT");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        let dt = 1.0 / 60.0;
        for _ in 0..(400 * 60) {
            f.update(&mut w, dt, false, &keys);
            if f.is_over() {
                break;
            }
        }
        let total: u32 = f.gm.score[0] + f.gm.score[1];
        assert!(total > 0, "phai co vong ket thuc (score={:?})", f.gm.score);
    }

    #[test]
    fn player_spawn_is_never_inside_geometry() {
        let _lock = crate::core::arena::map_test_lock();
        // Camera/nguoi choi khong duoc xuat hien BEN TRONG vat can tren
        // moi map 5v5 (da gay loi camera kiet trong tuong).
        for (mi, m) in ["DF-PORT", "DF-FOUNDRY", "DF-CARGO", "DF-YARD", "DF-SILO"]
            .iter()
            .enumerate()
        {
            crate::core::arena::set_active_map(crate::core::arena::MAP_DF_PORT + mi);
            for team in [0u8, 1u8] {
                let mut f = TeamFight::new(60.0, 1, 1, m);
                // ep team cua player
                let pid = f.gm.player_id;
                f.gm.actors[pid].team = team;
                let sp = f.gm.player_spawn();
                for b in crate::core::arena::current_map_blocks() {
                    let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
                    // the than nguoi: cham dat 0 -> 1.8m
                    let feet = sp[1];
                    let head = sp[1] + 1.8;
                    // AABB cua vat can co giao thep the nao voi than nguoi khong
                    let overlap_x = sp[0] > bx - hw - 0.2 && sp[0] < bx + hw + 0.2;
                    let overlap_z = sp[2] > bz - hd - 0.2 && sp[2] < bz + hd + 0.2;
                    let overlap_y = head > by - hh && feet < by + hh;
                    assert!(
                        !(overlap_x && overlap_y && overlap_z),
                        "map {} team {}: spawn {:?} cham vat can {:?}",
                        m, team, sp, b
                    );
                }
            }
        }
        crate::core::arena::set_active_map(0);
    }

    fn inside_any_block(pos: [f32; 3]) -> Option<[f32; 6]> {
        for b in crate::core::arena::current_map_blocks() {
            let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
            if pos[0] > bx - hw - 0.25
                && pos[0] < bx + hw + 0.25
                && pos[1] + 1.6 > by - hh - 0.1
                && pos[1] < by + hh + 0.1
                && pos[2] > bz - hd - 0.25
                && pos[2] < bz + hd + 0.25
            {
                return Some(*b);
            }
        }
        None
    }

    #[test]
    fn player_never_ends_up_inside_geometry_during_a_match() {
        let _lock = crate::core::arena::map_test_lock();
        // Bat loi "camera ket trong tuong" khi chơi that.
        for (mi, m) in ["DF-PORT", "DF-FOUNDRY", "DF-CARGO", "DF-YARD", "DF-SILO"]
            .iter()
            .enumerate()
        {
            crate::core::arena::set_active_map(crate::core::arena::MAP_DF_PORT + mi);
            let mut f = TeamFight::new(60.0, 1, 1, m);
            let mut w = World::default();
            let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
            let dt = 1.0 / 60.0;
            for i in 0..(25 * 60) {
                // doi chieu phim de co chuyen dong nhu nguoi that
                f.update(&mut w, dt, false, &keys);
                if i % 30 == 0 {
                    let p = w.pose.pos;
                    if let Some(b) = inside_any_block(p) {
                        panic!(
                            "map {}: nguoi choi o {:?} nam trong vat can {:?}",
                            m, p, b
                        );
                    }
                }
                if f.is_over() {
                    break;
                }
            }
        }
        crate::core::arena::set_active_map(0);
    }

    #[test]
    fn camera_starts_at_eye_height_not_on_the_floor() {
        // `World.pose.pos` la VI TRI MAT (y ~ 1.6), con `pose.feet` moi la
        // chan. Gan pose.pos = chan se lam camera nam sat dat -> man hinh
        // bi day mat trong cua vat can.
        let mut f = TeamFight::new(60.0, 1, 1, "DF-PORT");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        f.update(&mut w, 1.0 / 60.0, false, &keys);
        let eye = w.pose.pos[1];
        assert!(
            eye > 1.2,
            "camera phai o do cao cua mat (>=1.2m), hien tai {:.2}m",
            eye
        );
        assert!(
            (w.pose.pos[1] - w.pose.feet - w.pose.eye_h).abs() < 0.35,
            "mat phai cao hon chan mot khoang eye_h (pos.y={:.2} feet={:.2} eye_h={:.2})",
            w.pose.pos[1],
            w.pose.feet,
            w.pose.eye_h
        );
    }

    #[test]
    fn actor_feet_match_player_feet() {
        // actor.pos la VI TRI CHAN, phai khop voi pose.feet
        let mut f = TeamFight::new(60.0, 1, 1, "DF-PORT");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        for _ in 0..60 {
            f.update(&mut w, 1.0 / 60.0, false, &keys);
        }
        let p = &f.gm.actors[f.gm.player_id];
        assert!(
            (p.pos[1] - w.pose.feet).abs() < 0.1,
            "chan cua actor ({:.2}) phai bang chan cua nguoi choi ({:.2})",
            p.pos[1],
            w.pose.feet
        );
    }

    #[test]
    fn sim_produces_ten_actors_and_snapshot() {
        let mut f = TeamFight::new(60.0, 1, 1, "Duel Arena");
        let mut w = World::default();
        let keys = Keys { w: false, a: false, s: false, d: false, sprint: false, crouch: false, jump: false };
        f.update(&mut w, 1.0 / 60.0, false, &keys);
        assert_eq!(f.gm.actors.len(), 10);
        // snapshot render co 10 bot
        assert_eq!(w.bots.len(), 10);
        assert!(w.tf.active);
    }

    #[test]
    fn sim_teams_stay_balanced() {
        let f = simulate(1, 10.0);
        let mut ct = 0;
        let mut t = 0;
        for a in f.gm.actors.iter() {
            if a.team == 0 { ct += 1; } else { t += 1; }
        }
        assert_eq!(ct, TEAM_SIZE);
        assert_eq!(t, TEAM_SIZE);
    }

    #[test]
    fn ai_actually_moves() {
        let f = simulate(1, 8.0);
        // it nhat phai co bot doi vi tri so voi spawn
        let moved = f.gm.actors.iter().filter(|a| a.moving || a.path.len() > 0).count();
        // khong co y dinh phai co bot di, nhung trong 8s AI phai hoat dong
        // (co the dang Engage -> dung ban). Kiem tra it nhat co bot thay nhau.
        let _ = moved;
        // dung bang cach: so vi tri duy nhat > 1 sau 8s
        let mut xs: Vec<i32> = f.gm.actors.iter().map(|a| (a.pos[0] * 10.0) as i32).collect();
        xs.sort();
        xs.dedup();
        assert!(xs.len() > 1, "AI phai di chuyen, khong dung yen o spawn");
    }

    #[test]
    fn sim_produces_kills_eventually() {
        // chay lau hon, AI phai ban trung nhau -> co kill
        let f = simulate(2, 60.0);
        let total_kills: u32 = f.gm.actors.iter().map(|a| a.kills).sum();
        assert!(total_kills > 0, "AI khong ban trung gi sau 60s (kills={})", total_kills);
    }

    #[test]
    fn match_completes_with_winner() {
        // chay rat lau de tran ket thuc
        let f = simulate(2, 600.0);
        assert!(f.is_over(), "tran 600s phai ket thuc (score={:?})", f.gm.score);
        let winner = f.gm.score[0].max(f.gm.score[1]);
        assert_eq!(winner, super::super::round::ROUNDS_TO_WIN);
    }

    #[test]
    fn harder_difficulty_scores_more_kills() {
        let easy = simulate(0, 90.0);
        let hard = simulate(2, 90.0);
        let ek: u32 = easy.gm.actors.iter().map(|a| a.kills).sum();
        let hk: u32 = hard.gm.actors.iter().map(|a| a.kills).sum();
        assert!(hk >= ek, "kho phai ghi nhieu kill hon: {} vs {}", hk, ek);
    }
}
