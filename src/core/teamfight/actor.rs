// Actor: mot nguoi/doi cua mode 5v5. Giu toan bo trang thai chien dau cua
// mot bot (hoac nguoi choi) trong mot mode, tach khoi `core::Target` de khong
// pha mode cu (Gridshot/Flick/Tracking/Recoil/Duel van dung Target nhu cu).

use crate::core::math::ground_height;

pub const TEAM_CT: u8 = 0;
pub const TEAM_T: u8 = 1;
pub const TEAM_COUNT: u8 = 2;
/// so nguoi moi doi trong mode 5v5
pub const TEAM_SIZE: usize = 5;
/// tong so nguoi (ke ca nguoi choi)
pub const ROSTER_SIZE: usize = TEAM_SIZE * 2;

pub const HP_MAX: f32 = 100.0;
pub const ARMOR_MAX: f32 = 100.0;
/// giap chi hoi duoc khi khong trung dan sau mot khoang
pub const ARMOR_REGEN_DELAY: f32 = 4.0;
pub const ARMOR_REGEN_RATE: f32 = 25.0;
/// sau khi chet, xuat hien lai sau mot khoang
pub const RESPAWN_TIME: f32 = 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiState {
    /// chua thay dich, di truc tiep giua map
    Patrol,
    /// thay dich, di den gan
    Hunt,
    /// ton tai tia ban, dung ban
    Engage,
    /// lui ve sau vat can
    Cover,
    /// vua chet, chua the hoi sinh
    Dead,
}

#[derive(Clone, Debug)]
pub struct Actor {
    pub id: usize,
    /// TEAM_CT hoac TEAM_T
    pub team: u8,
    /// vi tri chan (feet), dung cho collision va nav
    pub pos: [f32; 3],
    /// huong nhin, radian. yaw = 0 nhin ve +Z
    pub yaw: f32,
    /// huong ban, radian (pitch)
    pub pitch: f32,
    pub hp: f32,
    pub armor: f32,
    /// thoi gian con lai truoc khi giap hoi lai
    pub armor_wait: f32,
    pub alive: bool,
    /// con bao lau moi duoc hoi sinh
    pub respawn_t: f32,
    /// index vao WEAPONS
    pub weapon: u8,
    pub ammo: u32,
    /// con bao lau moi ban duoc
    pub fire_t: f32,
    /// con bao lau nap dan
    pub reload_t: f32,
    /// dem dan trong loat hien tai (AI)
    pub burst: u32,
    /// thoi gian ngung giua cac loat (AI)
    pub burst_wait: f32,
    /// thoi gian phan ung truoc khi ban (AI) — cang lon cang kho
    pub react_t: f32,
    /// lech ngam hien tai (rad), "do lech" luu trong Actor::aim_jitter
    pub aim_err: [f32; 2],
    pub state: AiState,
    /// duong di toi diem cuoi (ke ca diem bat dau)
    pub path: Vec<[f32; 3]>,
    /// index trong `path` dang tro toi
    pub path_i: usize,
    /// thoi gian con lai cua trang thai hien tai
    pub state_t: f32,
    /// khoang cach toi muc tieu gan nhat (capped)
    pub sight: f32,
    /// muc tieu dang nham tinh hien tai
    pub target: Option<usize>,
    /// thoi gian da bi ban trung lan cuoi (dung cho radar)
    pub seen_t: f32,
    /// dang di chuyen (AI set trong navigate) — dung cho animation + HUD
    pub moving: bool,
    /// huong di ngang khi ban: 1 = phai, -1 = trai, 0 = dung yên
    pub strafe_dir: f32,
    /// dem nguoc doi huong di ngang (tranh kẹt 1 huong)
    pub strafe_t: f32,

    // thanh cong
    pub kills: u32,
    pub deaths: u32,
    pub headshots: u32,
    pub dmg_dealt: f32,
    /// true neu day la nguoi choi
    pub is_player: bool,
}

impl Actor {
    pub fn new(id: usize, team: u8, is_player: bool) -> Self {
        Actor {
            id,
            team,
            pos: [0.0, 0.0, 0.0],
            yaw: 0.0,
            pitch: 0.0,
            hp: HP_MAX,
            armor: 0.0,
            armor_wait: 0.0,
            alive: true,
            respawn_t: 0.0,
            weapon: 0,
            ammo: 12,
            fire_t: 0.0,
            reload_t: 0.0,
            burst: 0,
            burst_wait: 0.0,
            react_t: 0.0,
            aim_err: [0.0, 0.0],
            state: AiState::Patrol,
            path: Vec::new(),
            path_i: 0,
            state_t: 0.0,
            sight: 0.0,
            target: None,
            seen_t: 0.0,
            moving: false,
            strafe_dir: 0.0,
            strafe_t: 0.0,
            kills: 0,
            deaths: 0,
            headshots: 0,
            dmg_dealt: 0.0,
            is_player,
        }
    }

    /// dat lai sau moi vong / khi hoi sinh
    pub fn reset(&mut self, pos: [f32; 3], yaw: f32, weapon: u8) {
        let w = crate::core::weapon::weapon(weapon);
        self.pos = pos;
        self.yaw = yaw;
        self.pitch = 0.0;
        self.hp = HP_MAX;
        self.armor = 0.0;
        self.armor_wait = 0.0;
        self.alive = true;
        self.respawn_t = 0.0;
        self.weapon = weapon;
        self.ammo = w.mag;
        self.fire_t = 0.0;
        self.reload_t = 0.0;
        self.burst = 0;
        self.burst_wait = 0.0;
        self.react_t = 0.0;
        self.aim_err = [0.0, 0.0];
        self.state = AiState::Patrol;
        self.path.clear();
        self.path_i = 0;
        self.state_t = 0.0;
        self.sight = 0.0;
        self.target = None;
        self.seen_t = 0.0;
        self.moving = false;
        self.strafe_dir = 0.0;
        self.strafe_t = 0.0;
    }

    /// mat mau: giap an truoc, sau do mau. tra ve (da chet?, so sat thuong thuc te)
    pub fn damage(&mut self, amount: f32) -> (bool, f32) {
        if !self.alive {
            return (false, 0.0);
        }
        let mut left = amount;
        // giap an mot phan truoc
        if self.armor > 0.0 {
            let absorbed = self.armor.min(left * 0.5);
            self.armor -= absorbed;
            left -= absorbed;
        }
        // reset timer hoi giap
        self.armor_wait = ARMOR_REGEN_DELAY;
        let before = left;
        self.hp -= left;
        if self.hp <= 0.0 {
            self.hp = 0.0;
            self.alive = false;
            self.deaths += 1;
            self.respawn_t = RESPAWN_TIME;
            self.state = AiState::Dead;
            return (true, before);
        }
        (false, before)
    }

    /// hoi giap sau khi khong bi ban trong mot khoang
    pub fn tick_armor(&mut self, dt: f32) {
        if self.armor_wait > 0.0 {
            self.armor_wait -= dt;
        } else if self.armor < ARMOR_MAX {
            self.armor = (self.armor + ARMOR_REGEN_RATE * dt).min(ARMOR_MAX);
        }
    }

    /// dem het thoi gian hoi sinh -> song lai
    pub fn tick_respawn(&mut self, dt: f32) {
        if self.alive {
            return;
        }
        self.respawn_t -= dt;
    }

    /// khu vuc mat dat tai vi tri chan
    pub fn ground(&self) -> f32 {
        ground_height(self.pos[0], self.pos[2], self.pos[1])
    }

    /// huong nhin dang don vi vec3 (cho AI / ban).
    /// PHẢI khớp `core::math::forward` de player va bot dung mot quy uoc:
    /// yaw = 0 -> nhin ve -Z.
    pub fn forward(&self) -> [f32; 3] {
        let (sy, cy) = self.yaw.sin_cos();
        let (sp, cp) = self.pitch.sin_cos();
        [-cp * sy, sp, -cp * cy]
    }

    pub fn eye(&self) -> [f32; 3] {
        [self.pos[0], self.pos[1] + 1.6, self.pos[2]]
    }
}

/// team cua nguoi choi doi lai
pub fn other_team(t: u8) -> u8 {
    1 - t
}

/// ban kinh coi nhat giua 2 nguoi. Hai nguoi duoc day ra xa nhau khi
/// cham vao nhau (bot khong lot qua camera nguoi choi).
pub const ACTOR_R: f32 = 0.55;
/// khoang cach toi da ngan giữa bot và nguoi chơi.
/// FOV 103° rat rong: bot dung o 2m se che gan het man hinh, nên can
/// it nhat ~5m de nhin thay duoc phan con lai cua tran.
pub const ACTOR_STANDOFF: f32 = 5.0;

/// Day cac actor ra xa nhau trong 1 vi tri de khong chong/che nhau.
/// `skip` la id duoc bo qua (nguoi choi khong bi day boi ham nay).
/// Tra ve so cap bi tach ra.
pub fn separate(actors: &mut [Actor], skip: usize) -> u32 {
    let n = actors.len();
    let mut pushed = 0u32;
    // 2 vong: du de tach cac cum lon
    for _ in 0..2 {
        for i in 0..n {
            if actors[i].id == skip || !actors[i].alive {
                continue;
            }
            for j in (i + 1)..n {
                if actors[j].id == skip || !actors[j].alive {
                    continue;
                }
                let dx = actors[j].pos[0] - actors[i].pos[0];
                let dz = actors[j].pos[2] - actors[i].pos[2];
                let d2 = dx * dx + dz * dz;
                if d2 >= ACTOR_R * 2.0 * ACTOR_R * 2.0 {
                    continue;
                }
                let (dx, dz, d) = if d2 < 1e-6 {
                    // trung xac: day theo truc X
                    (1.0f32, 0.0f32, 0.0f32)
                } else {
                    let d = d2.sqrt();
                    (dx / d, dz / d, d)
                };
                let overlap = (ACTOR_R * 2.0 - d) * 0.5 + 0.001;
                actors[i].pos[0] -= dx * overlap;
                actors[i].pos[2] -= dz * overlap;
                actors[j].pos[0] += dx * overlap;
                actors[j].pos[2] += dz * overlap;
                pushed += 1;
            }
        }
    }
    pushed
}

/// Day 1 bot ra khoi nguoi choi + lui them theo `standoff` (bot khong
/// dic sat vao ban, va khong lot qua camera).
/// Tra ve true neu da day.
pub fn keep_off_player(me: &mut Actor, px: f32, pz: f32) -> bool {
    if me.id == 0 {
        return false;
    }
    let dx = me.pos[0] - px;
    let dz = me.pos[2] - pz;
    let d2 = dx * dx + dz * dz;
    let min = ACTOR_STANDOFF + ACTOR_R;
    if d2 >= min * min {
        return false;
    }
    let (nx, nz) = if d2 < 1e-6 {
        (1.0f32, 0.0f32)
    } else {
        let d = d2.sqrt();
        (dx / d, dz / d)
    };
    me.pos[0] = px + nx * min;
    me.pos[2] = pz + nz * min;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_restores_full_loadout() {
        let mut a = Actor::new(0, TEAM_CT, false);
        a.hp = 3.0;
        a.armor = 0.0;
        a.weapon = 2;
        a.ammo = 1;
        a.kills = 7;
        a.reset([1.0, 0.0, -2.0], 0.5, 1);
        assert_eq!(a.hp, HP_MAX);
        assert_eq!(a.alive, true);
        assert_eq!(a.weapon, 1);
        // reset nap bang day
        assert_eq!(a.ammo, crate::core::weapon::weapon(1).mag);
        // reset KHONG xoa thanh cong
        assert_eq!(a.kills, 7);
    }

    #[test]
    fn armor_absorbs_half_then_hp_takes_rest() {
        let mut a = Actor::new(0, TEAM_CT, false);
        a.armor = 50.0;
        let (died, dealt) = a.damage(60.0);
        // 50 giap -> an 30 (60*0.5) -> con 30 mau
        assert!((a.armor - 20.0).abs() < 0.01, "armor={}", a.armor);
        assert!((a.hp - 70.0).abs() < 0.01, "hp={}", a.hp);
        assert!(!died);
        assert!((dealt - 30.0).abs() < 0.01);
    }

    #[test]
    fn death_sets_flag_and_increments_deaths() {
        let mut a = Actor::new(3, TEAM_T, false);
        let (died, _) = a.damage(999.0);
        assert!(died);
        assert!(!a.alive);
        assert_eq!(a.deaths, 1);
        assert_eq!(a.hp, 0.0);
        assert_eq!(a.respawn_t, RESPAWN_TIME);
    }

    #[test]
    fn damage_on_dead_actor_is_ignored() {
        let mut a = Actor::new(0, TEAM_CT, false);
        a.damage(999.0);
        let hp_before = a.hp;
        let (died, dealt) = a.damage(50.0);
        assert!(!died);
        assert_eq!(dealt, 0.0);
        assert_eq!(a.hp, hp_before);
        // khong dem chet 2 lan
        assert_eq!(a.deaths, 1);
    }

    #[test]
    fn armor_regen_waits_for_delay_then_regens_to_cap() {
        let mut a = Actor::new(0, TEAM_CT, false);
        // bi ban -> reset timer hoi giap len 4s (damage tieu mot phan giap)
        a.armor = 100.0;
        a.damage(10.0);
        let after_hit = a.armor;
        a.tick_armor(0.1);
        assert_eq!(a.armor, after_hit, "khong hoi ngay khi vua bi ban");
        // con trong thoi gian cho (3.8s < 4s)
        a.tick_armor(3.8);
        assert_eq!(a.armor, after_hit, "khong hoi khi con trong thoi gian cho");
        // het cho -> hoi dan (tick them cho khi timer am)
        a.tick_armor(0.5);
        a.tick_armor(0.5);
        assert!(a.armor > after_hit, "phai bat dau hoi giap");
        // khong vuot tran
        a.tick_armor(1000.0);
        assert_eq!(a.armor, ARMOR_MAX);
    }

    #[test]
    fn armor_regen_blocked_while_recently_hit() {
        let mut a = Actor::new(0, TEAM_CT, false);
        a.armor = 0.0;
        a.damage(10.0); // reset wait len 4s
        a.tick_armor(3.9);
        assert_eq!(a.armor, 0.0, "khong duoc hoi khi moi bi ban");
    }

    #[test]
    fn respawn_timer_counts_down_only_when_dead() {
        let mut a = Actor::new(0, TEAM_CT, false);
        a.tick_respawn(5.0);
        assert!(a.alive, "nguoi song khong bi reset");
        a.damage(999.0);
        a.tick_respawn(1.0);
        assert!((a.respawn_t - 1.0).abs() < 0.01);
        a.tick_respawn(5.0);
        assert!(a.respawn_t < 0.0);
    }

    #[test]
    fn forward_is_unit_length() {
        let mut a = Actor::new(0, TEAM_CT, false);
        a.yaw = 0.7;
        a.pitch = -0.3;
        let f = a.forward();
        let len = (f[0] * f[0] + f[1] * f[1] + f[2] * f[2]).sqrt();
        assert!((len - 1.0).abs() < 0.0001);
    }

    #[test]
    fn other_team_flips() {
        assert_eq!(other_team(TEAM_CT), TEAM_T);
        assert_eq!(other_team(TEAM_T), TEAM_CT);
    }

    fn mk(id: usize, team: u8, x: f32, z: f32) -> Actor {
        let mut a = Actor::new(id, team, false);
        a.pos = [x, 0.0, z];
        a.alive = true;
        a
    }

    #[test]
    fn separate_pushes_overlapping_actors_apart() {
        let mut v = vec![mk(0, TEAM_CT, 0.0, 0.0), mk(1, TEAM_T, 0.1, 0.0)];
        let p = separate(&mut v, usize::MAX);
        assert!(p > 0, "phai day 2 nguoi dang de nhau");
        let d = (v[0].pos[0] - v[1].pos[0]).abs();
        assert!(d >= ACTOR_R * 2.0 - 0.01, "phai tach ra >= 2R, hien {:.2}", d);
    }

    #[test]
    fn separate_handles_exact_overlap() {
        let mut v = vec![mk(0, TEAM_CT, 3.0, 3.0), mk(1, TEAM_T, 3.0, 3.0)];
        separate(&mut v, usize::MAX);
        let dx = v[0].pos[0] - v[1].pos[0];
        let dz = v[0].pos[2] - v[1].pos[2];
        assert!((dx * dx + dz * dz).sqrt() >= ACTOR_R * 2.0 - 0.01);
    }

    #[test]
    fn separate_skips_player_and_dead() {
        // `separate` bo qua player (id 0) -> cap khong dung ham nay de
        // day bot khoi nguoi choi; ham do la `keep_off_player`.
        let mut v = vec![mk(0, TEAM_CT, 0.0, 0.0), mk(1, TEAM_T, 0.05, 0.0)];
        separate(&mut v, 0);
        assert!((v[0].pos[0] - 0.0).abs() < 0.0001, "player doi chong");
        assert!((v[1].pos[0] - 0.05).abs() < 0.0001, "cap khong doi bot");
    }

    #[test]
    fn separate_ignores_dead_actors() {
        let mut v = vec![mk(0, TEAM_CT, 0.0, 0.0), mk(1, TEAM_T, 0.1, 0.0)];
        v[1].alive = false;
        separate(&mut v, usize::MAX);
        assert!((v[0].pos[0] - 0.0).abs() < 0.0001);
    }

    #[test]
    fn keep_off_player_keeps_standoff() {
        let mut bot = mk(1, TEAM_T, 0.2, 0.0);
        let pushed = keep_off_player(&mut bot, 0.0, 0.0);
        assert!(pushed);
        let d = (bot.pos[0].powi(2) + bot.pos[2].powi(2)).sqrt();
        assert!(d >= ACTOR_STANDOFF + ACTOR_R - 0.01, "phaigiu ky: {}", d);
    }

    #[test]
    fn keep_off_player_noop_when_far() {
        let mut bot = mk(1, TEAM_T, 10.0, 0.0);
        assert!(!keep_off_player(&mut bot, 0.0, 0.0));
        assert_eq!(bot.pos[0], 10.0);
    }

    #[test]
    fn keep_off_player_never_moves_the_player() {
        // id 0 la player -> ham phai bo qua
        let mut p = mk(0, TEAM_CT, 0.1, 0.1);
        assert!(!keep_off_player(&mut p, 0.0, 0.0));
        assert_eq!(p.pos[0], 0.1);
    }
}
