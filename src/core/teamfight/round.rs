// Match: vong, doi, bang diem cho mode 5v5.
//
// Troi khu: nguoi choi thuoc doi nao cung duoc, HALFTIME doi chieu,
// nhan T de doi doi thu cong (chi 1 lan/tran). Ket thuc khi doi nao thang
// ROUNDS_TO_WIN vong.

use bevy::prelude::*;

use super::actor::{Actor, ROSTER_SIZE, TEAM_SIZE};
use crate::core::arena::arena_size;
use crate::core::weapon::WEAPON_COUNT;

/// thoi gian 1 vong (giay)
pub const ROUND_TIME: f32 = 60.0;
/// so vong can thang de ket thuc tran
pub const ROUNDS_TO_WIN: u32 = 5;
/// doi doi o giua vong nay
pub const HALFTIME_AFTER: u32 = 4;
/// thoi gian dong bang giua cac vong (giay)
pub const FREEZE_TIME: f32 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchState {
    /// giua cac vong: dong bang, hien banner
    Freeze,
    /// vong dang chay
    Live,
    /// co doi thang ROUNDS_TO_WIN vong -> ket thuc
    Over,
}

impl MatchState {
    pub fn is_live(self) -> bool {
        self == MatchState::Live
    }
}

/// Diem/ket qua cua mot nguoi choi cuoi tran, dung cho `crate::core::Results`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerResult {
    pub kills: u32,
    pub deaths: u32,
    pub headshots: u32,
    pub damage: f32,
    pub team: u8,
    pub won: bool,
}

impl PlayerResult {
    pub fn kd(&self) -> f32 {
        if self.deaths == 0 {
            if self.kills == 0 { 0.0 } else { self.kills as f32 }
        } else {
            self.kills as f32 / self.deaths as f32
        }
    }
}

/// Ket qua cua ca tran.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MatchResult {
    pub score: [u32; 2],
    pub winner: Option<u8>,
    pub player: PlayerResult,
}

#[derive(Resource)]
pub struct Match {
    pub actors: Vec<Actor>,
    pub player_id: usize,
    /// so vong hien tai, 1-based
    pub round: u32,
    /// so vong moi doi thang
    pub score: [u32; 2],
    pub time_left: f32,
    pub state: MatchState,
    /// con bao lau freeze
    pub freeze_t: f32,
    /// da doi doi o giua vong 4 chua
    pub halftime_done: bool,
    /// nguoi choi da dung phep doi doi thu cong trong vong nay chua
    pub switched_this_round: bool,
    /// bien dem de tinh tuy chon diem (test + debug)
    pub rng: u32,
    /// do kho AI (0=de, 1=vua, 2=kho)
    pub difficulty: usize,
}

impl Match {
    /// Tao tran moi: `difficulty` chi dung cho AI, `seed` de tai lap.
    pub fn new(difficulty: usize, seed: u32) -> Self {
        let mut actors: Vec<Actor> = Vec::with_capacity(ROSTER_SIZE);
        for id in 0..ROSTER_SIZE {
            let is_player = id == 0;
            // id 0..4 = doi CT, id 5..9 = doi T (khong xen ke, de spawn
            // de tinh theo `id % TEAM_SIZE`)
            let team = if id < TEAM_SIZE { 0u8 } else { 1u8 };
            let mut a = Actor::new(id, team, is_player);
            a.reset(spawn_point(team, id % TEAM_SIZE, is_player), spawn_yaw(team), 0);
            a.kills = 0;
            a.deaths = 0;
            a.headshots = 0;
            a.dmg_dealt = 0.0;
            actors.push(a);
        }
        let mut m = Match {
            actors,
            player_id: 0,
            round: 1,
            score: [0, 0],
            time_left: ROUND_TIME,
            state: MatchState::Freeze,
            freeze_t: FREEZE_TIME,
            halftime_done: false,
            switched_this_round: false,
            rng: seed | 1,
            difficulty,
        };
        // gan sung ngau nhien cho bot (loop rieng de khong vay mut double borrow)
        let weapons: Vec<u8> = (0..ROSTER_SIZE)
            .filter(|id| *id != 0)
            .map(|_| m.pick_weapon())
            .collect();
        let mut wi = 0;
        for a in m.actors.iter_mut() {
            if !a.is_player {
                a.weapon = weapons[wi];
                wi += 1;
            }
        }
        m
    }

    pub fn difficulty(&self) -> usize {
        self.difficulty
    }

    /// xoay so ngau nhien nhanh, de tranh phu thuoc `fastrand` trong test
    pub fn next_rand(&mut self) -> f32 {
        // xorshift32
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f32 / 16_777_216.0
    }

    /// chon ngau nhien 1 trong WEAPON_COUNT khu vuc — dung cho bot.
    pub fn pick_weapon(&mut self) -> u8 {
        (self.next_rand() * WEAPON_COUNT as f32) as u8 % WEAPON_COUNT as u8
    }

    pub fn player(&self) -> &Actor {
        &self.actors[self.player_id]
    }
    pub fn player_mut(&mut self) -> &mut Actor {
        let i = self.player_id;
        &mut self.actors[i]
    }

    pub fn player_team(&self) -> u8 {
        self.actors[self.player_id].team
    }

    /// Vi tri spawn cua nguoi choi theo doi hien tai.
    /// Dung khi hoi sinh — KHONG duoc dung choi chet (o do co the nam
    /// trong vat can).
    pub fn player_spawn(&self) -> [f32; 3] {
        let a = &self.actors[self.player_id];
        spawn_point(a.team, self.player_id % TEAM_SIZE, a.is_player)
    }

    /// Huong nhin khi hoi sinh (nhin vao giua map).
    pub fn player_spawn_yaw(&self) -> f32 {
        spawn_yaw(self.player_team())
    }

    /// tong so kill cua 1 doi trong vong hien tai
    pub fn team_kills(&self, team: u8) -> u32 {
        self.actors
            .iter()
            .filter(|a| a.team == team)
            .map(|a| a.kills)
            .sum()
    }

    /// So nguoi dang song cua doi
    pub fn team_alive(&self, team: u8) -> usize {
        self.actors
            .iter()
            .filter(|a| a.team == team && a.alive)
            .count()
    }

    #[allow(dead_code)]
    fn team_sizes(&self) -> (usize, usize) {
        let mut c = [0usize; 2];
        for a in self.actors.iter() {
            c[(a.team as usize).min(1)] += 1;
        }
        (c[0], c[1])
    }

    /// Bat dau vong moi: reset vi tri/mau, random lai sung bot, reset score vong.
    pub fn start_round(&mut self) {
        self.round += 1;
        self.time_left = ROUND_TIME;
        self.state = MatchState::Freeze;
        self.freeze_t = FREEZE_TIME;
        self.switched_this_round = false;
        // random sung truoc (loop rieng)
        let pid = self.player_id;
        let weapons: Vec<u8> = (0..ROSTER_SIZE)
            .filter(|id| *id != pid)
            .map(|_| self.pick_weapon())
            .collect();
        let mut wi = 0;
        for a in self.actors.iter_mut() {
            a.kills = 0;
            a.deaths = 0;
            a.headshots = 0;
            a.dmg_dealt = 0.0;
            if a.id != self.player_id {
                a.weapon = weapons[wi];
                wi += 1;
            }
            let (pos, yaw) = (
                spawn_point(a.team, a.id % TEAM_SIZE, a.is_player),
                spawn_yaw(a.team),
            );
            let w = a.weapon;
            a.reset(pos, yaw, w);
        }
    }

    /// Cap nhat dong bang / het gio -> chuyen sang Live, hoac ket thuc vong.
    /// Tra ve true neu vong vua ket thuc (de goi ket qua).
    pub fn tick_round(&mut self, dt: f32) -> bool {
        match self.state {
            MatchState::Over => return false,
            MatchState::Freeze => {
                self.freeze_t -= dt;
                if self.freeze_t <= 0.0 {
                    self.state = MatchState::Live;
                }
                return false;
            }
            MatchState::Live => {
                self.time_left -= dt;
                if self.time_left <= 0.0 {
                    self.time_left = 0.0;
                    self.end_round();
                    return true;
                }
                return false;
            }
        }
    }

    /// Ket thuc vong theo so kill. Tra ve true neu ca tran xong.
    pub fn end_round(&mut self) -> bool {
        let ct = self.team_kills(0);
        let t = self.team_kills(1);
        let winner: Option<u8> = match ct.cmp(&t) {
            std::cmp::Ordering::Greater => Some(0),
            std::cmp::Ordering::Less => Some(1),
            std::cmp::Ordering::Equal => None,
        };
        self.finish_round(winner)
    }

    /// Ket thuc vong voi doi thang cu dinh (dung cho mode bom: bom no /
    /// het ngui). `winner` = None nghia vong hoa.
    pub fn end_round_team(&mut self, winner: u8) -> bool {
        self.finish_round(Some(winner & 1))
    }

    /// Ket thuc vong hoa (bom da bi go).
    pub fn end_round_draw(&mut self) -> bool {
        self.finish_round(None)
    }

    /// Logic chung: cong diem -> chien thang / halftime -> vong moi.
    fn finish_round(&mut self, winner: Option<u8>) -> bool {
        if let Some(w) = winner {
            self.score[w as usize] += 1;
        }
        if self.score[0] >= ROUNDS_TO_WIN || self.score[1] >= ROUNDS_TO_WIN {
            self.state = MatchState::Over;
            return true;
        }
        // halftime: doi chieu sau vong HALFTIME_AFTER
        if !self.halftime_done && self.round >= HALFTIME_AFTER {
            self.halftime_done = true;
            self.swap_all_teams();
        }
        self.start_round();
        false
    }

    /// doi doi chieu: moi doi doi sang doi cua no
    pub fn swap_all_teams(&mut self) {
        for a in self.actors.iter_mut() {
            a.team = 1 - a.team;
        }
    }

    /// nguoi choi bam T: doi doi thu cong. Tra ve true neu doi duoc.
    /// chi 1 lan/tran; doi ca nguoi choi va 1 bot cua doi cu sang doi moi.
    pub fn manual_switch(&mut self) -> bool {
        if self.switched_this_round || self.state == MatchState::Over {
            return false;
        }
        let old_team = self.player_team();
        let new_team = 1 - old_team;
        // tim 1 bot cua doi moi de doi sang doi cu (de so nguoi moi doi luon 5/5)
        let donor = self
            .actors
            .iter()
            .find(|a| !a.is_player && a.team == new_team)
            .map(|a| a.id);
        if let Some(d) = donor {
            self.actors[d].team = old_team;
        }
        let pid = self.player_id;
        self.actors[pid].team = new_team;
        self.switched_this_round = true;
        // cap nhat vi tri spawn ngay cho ca 2
        let pid = self.player_id;
        let pl = &mut self.actors[pid];
        pl.pos = spawn_point(pl.team, pl.id % TEAM_SIZE, pl.is_player);
        pl.yaw = spawn_yaw(pl.team);
        if let Some(d) = donor {
            let a = &mut self.actors[d];
            a.pos = spawn_point(a.team, a.id % TEAM_SIZE, a.is_player);
            a.yaw = spawn_yaw(a.team);
        }
        true
    }

    pub fn is_over(&self) -> bool {
        self.state == MatchState::Over
    }

    /// Ket qua cua nguoi choi cho man hinh Results.
    pub fn result(&self) -> MatchResult {
        let p = self.player();
        let winner = if self.score[0] > self.score[1] {
            Some(0u8)
        } else if self.score[1] > self.score[0] {
            Some(1u8)
        } else {
            None
        };
        MatchResult {
            score: self.score,
            winner,
            player: PlayerResult {
                kills: p.kills,
                deaths: p.deaths,
                headshots: p.headshots,
                damage: p.dmg_dealt,
                team: p.team,
                won: winner == Some(p.team),
            },
        }
    }
}

// Vị trí spawn: 2 khu doi nhau tren truc Z, rai doc X.
// Di chuyen nhanh cho 5 map moi (P4) se override ham nay.
fn spawn_point(team: u8, slot: usize, is_player: bool) -> [f32; 3] {
    let lim = arena_size() - 2.5;
    let dir = if team == 0 { 1.0f32 } else { -1.0f32 };
    // Nguoi choi: giua khu, lui 1m -> bot khong che man hinh luc spawn.
    if is_player {
        return [0.0, 0.0, dir * (lim - 1.0)];
    }
    // 4 bot con lai rai rong 2 ben, 2 hang — moi nguoi 1 o rieng va
    // it nhat ~2.8m cach nguoi choi de khong che man hinh.
    // 5 vi tri distinct cho 1 khu. Nguoi choi (slot 0 cua doi CT) da
    // duoc gan rieng o tren.
    // QUAN TRONG: nguoi choi nhin ve giua map, nen bot phai o BEN hoac
    // SAU — tranh dung chinh phia truoc che man hinh.
    let (x, dz) = match slot {
        0 => (-5.5, 0.5),
        1 => (5.5, 0.5),
        2 => (-2.8, -0.8),
        3 => (2.8, -0.8),
        _ => (0.0, -1.6),
    };
    [x, 0.0, dir * (lim - dz)]
}

fn spawn_yaw(team: u8) -> f32 {
    // `math::forward(yaw, 0)` = (-sin(yaw), 0, -cos(yaw)).
    //   yaw = 0   -> nhin ve -Z
    //   yaw = PI  -> nhin ve +Z
    // doi CT o +Z nen nhin -Z (yaw 0); doi T o -Z nen nhin +Z (yaw PI).
    if team == 0 { 0.0 } else { std::f32::consts::PI }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m() -> Match {
        Match::new(1, 12345)
    }

    #[test]
    fn new_has_full_roster_and_balanced_teams() {
        let g = m();
        assert_eq!(g.actors.len(), ROSTER_SIZE);
        let (c, t) = g.team_sizes();
        assert_eq!(c, TEAM_SIZE);
        assert_eq!(t, TEAM_SIZE);
        assert_eq!(g.actors[0].is_player, true);
        assert_eq!(g.actors.iter().filter(|a| !a.is_player).count(), 9);
    }

    #[test]
    fn spawn_points_are_all_distinct_and_spread_out() {
        let g = m();
        let mut pts: Vec<(i32, i32)> = g
            .actors
            .iter()
            .map(|a| ((a.pos[0] * 10.0) as i32, (a.pos[2] * 10.0) as i32))
            .collect();
        let n = pts.len();
        pts.sort();
        pts.dedup();
        assert_eq!(pts.len(), n, "khong duoc co 2 nguoi cung mot diem spawn");
        // moi nguoi phai cach nhau it nhat 2m
        for i in 0..g.actors.len() {
            for j in (i + 1)..g.actors.len() {
                let dx = g.actors[i].pos[0] - g.actors[j].pos[0];
                let dz = g.actors[i].pos[2] - g.actors[j].pos[2];
                let d = (dx * dx + dz * dz).sqrt();
                assert!(d >= 2.0, "spawn {} va {} qua gan: {:.2}m", i, j, d);
            }
        }
    }

    #[test]
    fn no_teammate_spawns_directly_in_front_of_player() {
        let g = m();
        let p = &g.actors[g.player_id];
        // forward = (-sin(yaw), -cos(yaw)) — xem core::math::forward
        let fx = -p.yaw.sin();
        let fz = -p.yaw.cos();
        for a in g.actors.iter() {
            if a.id == p.id {
                continue;
            }
            let dx = a.pos[0] - p.pos[0];
            let dz = a.pos[2] - p.pos[2];
            let d = (dx * dx + dz * dz).sqrt();
            let dot = (dx * fx + dz * fz) / d.max(1e-4);
            // khong ai duoc nam trong 40 do phia truoc trong 4m
            assert!(
                !(dot > 0.76 && d < 4.0),
                "{} o ngay phia truoc nguoi choi ({:.1}m) - che man hinh",
                a.id,
                d
            );
        }
    }

    #[test]
    fn both_teams_face_each_other() {        let g = m();
        let ct = g.actors.iter().find(|a| a.team == 0).unwrap();
        let tm = g.actors.iter().find(|a| a.team == 1).unwrap();
        // CT o +Z nhin ve -Z, T o -Z nhin ve +Z
        assert!(ct.pos[2] > 0.0);
        assert!(tm.pos[2] < 0.0);
        let cf = ct.forward();
        let tf = tm.forward();
        assert!(cf[2] < 0.0, "CT phai nhin ve -Z (vao giua map)");
        assert!(tf[2] > 0.0, "T phai nhin ve +Z (vao giua map)");
    }

    #[test]
    fn freeze_then_live() {
        let mut g = m();
        assert_eq!(g.state, MatchState::Freeze);
        g.tick_round(FREEZE_TIME + 0.1);
        assert_eq!(g.state, MatchState::Live);
    }

    #[test]
    fn round_ends_when_timer_expires() {
        let mut g = m();
        g.tick_round(FREEZE_TIME + 0.1);
        let ended = g.tick_round(ROUND_TIME + 1.0);
        assert!(ended, "het gio phai ket thuc vong");
        // sau khi ket thuc, vong moi da bat dau (freeze, dong ho reset)
        assert_eq!(g.state, MatchState::Freeze);
        assert_eq!(g.round, 2);
        assert_eq!(g.time_left, ROUND_TIME);
    }

    #[test]
    fn team_with_more_kills_wins_round() {
        let mut g = m();
        g.tick_round(FREEZE_TIME + 0.1);
        // team 0 ghi 2 kill
        for a in g.actors.iter_mut() {
            if a.team == 0 { a.kills = 2; }
        }
        g.tick_round(ROUND_TIME + 1.0);
        assert_eq!(g.score[0], 1);
        assert_eq!(g.score[1], 0);
    }

    #[test]
    fn draw_gives_no_point() {
        let mut g = m();
        g.tick_round(FREEZE_TIME + 0.1);
        g.tick_round(ROUND_TIME + 1.0); // ca hai deu 0 kill
        assert_eq!(g.score[0], 0);
        assert_eq!(g.score[1], 0);
    }

    #[test]
    fn first_to_five_wins_match() {
        let mut g = m();
        // cho phep toi da vong (5-4 la tran dai nhat). `tick_round` tra ve
        // "vong vua ket thuc" chu khong phai "match xong", nen phai check
        // `is_over()` sau moi vong.
        for _ in 0..12 {
            g.tick_round(FREEZE_TIME + 0.1);
            if g.is_over() {
                break;
            }
            let t = g.player_team();
            for a in g.actors.iter_mut() {
                if a.team == t {
                    a.kills = 5;
                }
            }
            g.tick_round(ROUND_TIME + 1.0);
        }
        assert!(g.is_over(), "phai co doi thang 5 vong");
        assert_eq!(g.score.iter().max().unwrap(), &ROUNDS_TO_WIN);
    }

    #[test]
    fn halftime_swaps_teams_after_four_rounds() {
        let mut g = m();
        // vong 1..4: doi team 0 ghi 1 kill moi vong de tranh hoa
        for r in 1..=4 {
            g.tick_round(FREEZE_TIME + 0.1);
            for a in g.actors.iter_mut() {
                if a.team == 0 { a.kills = 1; }
            }
            g.tick_round(ROUND_TIME + 1.0);
            let _ = r;
        }
        assert!(g.halftime_done, "phai doi doi o giua vong 4");
        // player doi sang doi 1
        assert_eq!(g.player_team(), 1);
    }

    #[test]
    fn manual_switch_swaps_player_and_one_bot() {
        let mut g = m();
        let before_ct = g.team_sizes().0;
        let ok = g.manual_switch();
        assert!(ok);
        assert_eq!(g.player_team(), 1);
        // van 5/5
        let (c, t) = g.team_sizes();
        assert_eq!(c, before_ct);
        assert_eq!(c, TEAM_SIZE);
        assert_eq!(t, TEAM_SIZE);
    }

    #[test]
    fn manual_switch_only_once_per_match() {
        let mut g = m();
        assert!(g.manual_switch());
        // doi lai: khong duoc
        assert!(!g.manual_switch(), "chi doi doi 1 lan");
    }

    #[test]
    fn manual_switch_preserves_team_counts_exactly() {
        let mut g = m();
        for _ in 0..3 {
            g.manual_switch();
        }
        // chi 1 lan nen van la 5/5
        let (c, t) = g.team_sizes();
        assert_eq!((c, t), (TEAM_SIZE, TEAM_SIZE));
    }

    #[test]
    fn bots_get_random_weapons_from_table() {
        let g = m();
        for a in g.actors.iter() {
            if !a.is_player {
                assert!((a.weapon as usize) < WEAPON_COUNT);
            }
        }
    }

    #[test]
    fn result_reports_player_stats_and_win() {
        let mut g = m();
        let pid = g.player_id;
        g.actors[pid].kills = 10;
        g.actors[pid].deaths = 4;
        g.actors[pid].headshots = 3;
        g.score = [5, 2];
        let r = g.result();
        assert_eq!(r.player.kills, 10);
        assert_eq!(r.player.deaths, 4);
        assert_eq!(r.player.headshots, 3);
        assert_eq!(r.winner, Some(0));
        // player team 0, thang -> won
        assert_eq!(r.player.team, 0);
        assert!(r.player.won);
    }

    #[test]
    fn kd_handles_zero_deaths() {
        let mut g = m();
        let pid = g.player_id;
        g.actors[pid].kills = 3;
        g.actors[pid].deaths = 0;
        let r = g.result();
        assert_eq!(r.player.kd(), 3.0);
    }

    #[test]
    fn start_round_resets_actors_but_keeps_match_score() {
        let mut g = m();
        g.tick_round(FREEZE_TIME + 0.1);
        g.actors[0].kills = 7;
        g.score[0] = 1;
        g.start_round();
        assert_eq!(g.actors[0].kills, 0, "reset kill moi vong");
        assert_eq!(g.score[0], 1, "khong reset diem tran");
        assert_eq!(g.actors[0].hp, super::super::actor::HP_MAX);
    }
}
