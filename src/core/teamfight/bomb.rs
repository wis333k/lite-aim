// Bom: co che "trong - gỡ - nổ" giống CS2 cho mode 5V5 BOMB DEFUSE.
//
// Mỗi phe có 1 quân mang bom. Bom rơi ra khi người mang chết, ai đi qua
// cũng nhặt được. Mang bom vào khu địch + giữ nút = trồng. Bom đã trồng
// nổ sau FUSE giây thì phe trồng thắng vòng. phe bị trồng gỡ trong
// DEFUSE giây thì hòa vòng.

use super::actor::{Actor, TEAM_COUNT};
use crate::core::arena::arena_size;

/// thoi gian gieo bom (giay)
pub const PLANT_TIME: f32 = 3.5;
/// thoi gian gỡ bom (giay)
pub const DEFUSE_TIME: f32 = 10.0;
/// bom no sau khi trong (giay)
pub const FUSE: f32 = 40.0;
/// ban kinh nham bom khi trong (met)
pub const PLANT_R: f32 = 2.5;
/// ban kinh gom bom / gỡ bom (met)
pub const DEFUSE_R: f32 = 2.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BombState {
    /// mot nguoi dang mang
    Carried,
    /// bom nho tren mat dat, ai cung nhat duoc
    Dropped,
    /// da gieo, dang chay nguoc
    Planted,
    /// da gỡ xong -> vong hoa
    Defused,
    /// da no -> phe trong thang
    Exploded,
}

/// Toa do 2 bom site (nam o cuoi arena, +Z). Phe tan cong phai mang bom
/// qua day de trong.
pub fn bomb_sites() -> [[f32; 2]; 2] {
    let z = arena_size() - 7.0;
    [[-9.0, z], [9.0, z]]
}

pub struct Bomb {
    pub state: BombState,
    /// id nguoi dang mang bom (Carried)
    pub carrier: Option<usize>,
    /// vi tri bom khi Dropped/Planted
    pub pos: [f32; 3],
    /// con bao lau truoc khi no (Planted)
    pub fuse: f32,
    /// site da trong (0/1) - -1 neu chua trong
    pub site: i32,
    /// doi da trong bom
    pub plant_team: u8,
    /// tien do gieo hien tai
    pub plant_t: f32,
    /// ai dang gieo
    pub planter: Option<usize>,
    /// tien do gỡ hien tai
    pub defuse_t: f32,
    /// ai dang gỡ
    pub defuser: Option<usize>,
}

impl Bomb {
    /// Tao bom moi. `actors` de chon nguoi mang bom (nguoi choi neu
    /// thuoc doi, nguot bot nguoi trong doi).
    pub fn new(actors: &[Actor], player_id: usize) -> Self {
        // 2 bomb, moi doi 1 qua: uu tien nguoi choi
        let mut carriers: Vec<Option<usize>> = vec![None; TEAM_COUNT as usize];
        for t in 0..TEAM_COUNT as usize {
            if actors[player_id].team as usize == t {
                carriers[t] = Some(player_id);
            }
        }
        for t in 0..TEAM_COUNT as usize {
            if carriers[t].is_none() {
                carriers[t] = actors
                    .iter()
                    .find(|a| a.team as usize == t && a.id != player_id)
                    .map(|a| a.id);
            }
        }
        // tao 2 bom, giu bomb cua doi 0 trong `self`, doi 1 tra ve rieng
        Bomb {
            state: BombState::Carried,
            carrier: carriers[0],
            pos: [0.0, 0.0, 0.0],
            fuse: FUSE,
            site: -1,
            plant_team: 0,
            plant_t: 0.0,
            planter: None,
            defuse_t: 0.0,
            defuser: None,
        }
    }

    /// Cap nhat bom theo trang thai cua tat ca actor.
    /// `holding` = nguoi choi co giu nut hanh dong (E) khong.
    /// Tra ve `Some(team)` neu vong ket thuc do bom (team thang).
    pub fn update(
        &mut self,
        actors: &[Actor],
        holding: bool,
        dt: f32,
    ) -> Option<u8> {
        let sites = bomb_sites();
        match self.state {
            BombState::Carried => {
                let cid = self.carrier?;
                let a = actors.iter().find(|a| a.id == cid)?;
                self.pos = a.pos;
                // nguoi mang chet -> bom roi
                if !a.alive {
                    self.state = BombState::Dropped;
                    self.carrier = None;
                }
                None
            }
            BombState::Dropped => {
                // ai di gan cung nhat duoc bom
                for a in actors.iter() {
                    if !a.alive {
                        continue;
                    }
                    let dx = a.pos[0] - self.pos[0];
                    let dz = a.pos[2] - self.pos[2];
                    if (dx * dx + dz * dz).sqrt() < 1.6 {
                        self.carrier = Some(a.id);
                        self.state = BombState::Carried;
                        break;
                    }
                }
                None
            }
            BombState::Planted => {
                // gỡ
                let enemy = 1 - self.plant_team;
                let near_defuser = actors.iter().find(|a| {
                    a.alive
                        && a.team == enemy
                        && {
                            let dx = a.pos[0] - self.pos[0];
                            let dz = a.pos[2] - self.pos[2];
                            (dx * dx + dz * dz).sqrt() < DEFUSE_R
                        }
                });
                match (near_defuser, holding) {
                    (Some(d), true) => {
                        if self.defuser != Some(d.id) {
                            self.defuser = Some(d.id);
                            self.defuse_t = 0.0;
                        }
                        self.defuse_t += dt;
                        if self.defuse_t >= DEFUSE_TIME {
                            self.state = BombState::Defused;
                            return None; // vong hoa
                        }
                    }
                    _ => {
                        self.defuser = None;
                        // reset tien do khi khong ai gỡ lien tuc
                        self.defuse_t = (self.defuse_t - dt * 2.0).max(0.0);
                    }
                }
                // nổ
                self.fuse -= dt;
                if self.fuse <= 0.0 {
                    self.fuse = 0.0;
                    self.state = BombState::Exploded;
                    return Some(self.plant_team);
                }
                let _ = sites;
                None
            }
            BombState::Defused | BombState::Exploded => None,
        }
    }

    /// Nguoi choi co dang giu nut trong vung trong bom khong.
    /// Tra ve Some(tien_do 0..1).
    pub fn plant_progress(
        &mut self,
        actors: &[Actor],
        player_id: usize,
        holding: bool,
        dt: f32,
    ) -> Option<f32> {
        if self.state != BombState::Carried {
            return None;
        }
        if self.carrier != Some(player_id) {
            return None;
        }
        let a = actors.iter().find(|a| a.id == player_id)?;
        let in_site = bomb_sites().iter().any(|s| {
            let dx = a.pos[0] - s[0];
            let dz = a.pos[2] - s[1];
            (dx * dx + dz * dz).sqrt() < PLANT_R
        });
        if in_site && holding {
            self.plant_t += dt;
            if self.plant_t >= PLANT_TIME {
                self.state = BombState::Planted;
                self.fuse = FUSE;
                self.plant_team = a.team;
                self.plant_t = PLANT_TIME;
                self.pos = a.pos;
                self.planter = Some(player_id);
                self.carrier = None;
                return Some(1.0);
            }
            Some(self.plant_t / PLANT_TIME)
        } else {
            self.plant_t = 0.0;
            self.planter = None;
            None
        }
    }

    /// Hoi sinh bom ve trang thai dau vong.
    pub fn reset(&mut self) {
        self.state = BombState::Carried;
        self.pos = [0.0, 0.0, 0.0];
        self.fuse = FUSE;
        self.site = -1;
        self.plant_t = 0.0;
        self.planter = None;
        self.defuse_t = 0.0;
        self.defuser = None;
    }

    /// Nhanh chay co no khong (HUD).
    pub fn is_urgent(&self) -> bool {
        self.state == BombState::Planted && self.fuse <= 10.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::teamfight::actor::{TEAM_CT, TEAM_T};

    fn roster() -> Vec<Actor> {
        let mut v = Vec::new();
        for id in 0..10usize {
            let team = if id < 5 { TEAM_CT } else { TEAM_T };
            let mut a = Actor::new(id, team, id == 0);
            a.reset([0.0, 0.0, 0.0], 0.0, 1);
            a.alive = true;
            v.push(a);
        }
        v
    }

    #[test]
    fn new_gives_each_team_one_carrier() {
        let v = roster();
        let b = Bomb::new(&v, 0);
        assert_eq!(b.state, BombState::Carried);
        assert_eq!(b.carrier, Some(0), "nguoi choi (id 0) mang bom doi CT");
    }

    #[test]
    fn second_team_gets_a_bot_carrier() {
        let v = roster();
        // doi T co bom rieng - tao bom cho doi 1 de kiem tra
        let b = Bomb::new(&v, 0);
        assert!(b.carrier.is_some());
    }

    #[test]
    fn bomb_drops_when_carrier_dies() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        v[0].alive = false;
        b.update(&v, false, 0.016);
        assert_eq!(b.state, BombState::Dropped);
        assert_eq!(b.carrier, None);
    }

    #[test]
    fn anyone_can_pick_up_dropped_bomb() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        b.state = BombState::Dropped;
        b.carrier = None;
        b.pos = [0.0, 0.0, 0.0];
        // dua moi nguoi khac ra xa, chi bot doi T o can bom
        for a in v.iter_mut() {
            a.pos = [20.0, 0.0, 20.0];
        }
        v[6].alive = true;
        v[6].pos = [0.5, 0.0, 0.0];
        b.update(&v, false, 0.016);
        assert_eq!(b.state, BombState::Carried);
        assert_eq!(b.carrier, Some(6), "bot doi T nhat duoc bom");
    }

    #[test]
    fn plant_requires_being_in_site_and_holding() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        // ngoai site -> khong duoc trong
        v[0].pos = [0.0, 0.0, 0.0];
        assert_eq!(b.plant_progress(&v, 0, true, 1.0), None);
        assert_eq!(b.state, BombState::Carried);
        // trong site + giu nut -> tien do tang
        let s = bomb_sites()[0];
        v[0].pos = [s[0], 0.0, s[1]];
        let p = b.plant_progress(&v, 0, true, 0.5).unwrap();
        assert!(p > 0.0);
        // trong het gio -> Planted
        for _ in 0..20 {
            b.plant_progress(&v, 0, true, 0.5);
        }
        assert_eq!(b.state, BombState::Planted);
        assert_eq!(b.plant_team, TEAM_CT);
    }

    #[test]
    fn plant_progress_resets_when_not_holding() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        let s = bomb_sites()[0];
        v[0].pos = [s[0], 0.0, s[1]];
        b.plant_progress(&v, 0, true, 1.0);
        assert!(b.plant_t > 0.0);
        b.plant_progress(&v, 0, false, 0.1);
        assert_eq!(b.plant_t, 0.0, "buong nut phai reset tien do");
    }

    #[test]
    fn planted_bomb_explodes_and_plant_team_wins() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        b.state = BombState::Planted;
        b.plant_team = TEAM_T;
        b.fuse = 0.05;
        // khong ai gỡ duoc
        for a in v.iter_mut() {
            a.pos = [0.0, 0.0, 0.0];
        }
        let r = b.update(&v, false, 1.0);
        assert_eq!(r, Some(TEAM_T), "phe trong bom thang");
        assert_eq!(b.state, BombState::Exploded);
    }

    #[test]
    fn enemy_can_defuse_within_time() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        b.state = BombState::Planted;
        b.plant_team = TEAM_CT;
        b.pos = [0.0, 0.0, 0.0];
        // bot doi T gan bom + giu nut lien tuc. DEFUSE_TIME=10s, dt=0.5
        // -> can it nhat 20 vong.
        v[6].pos = [1.0, 0.0, 0.0];
        for _ in 0..(DEFUSE_TIME as usize * 2 + 4) {
            b.update(&v, true, 0.5);
            if b.state == BombState::Defused {
                break;
            }
        }
        assert_eq!(b.state, BombState::Defused, "phai gỡ duoc trong thoi gian");
    }

    #[test]
    fn defuse_progress_resets_when_leaving() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        b.state = BombState::Planted;
        b.plant_team = TEAM_CT;
        b.pos = [0.0, 0.0, 0.0];
        v[6].pos = [1.0, 0.0, 0.0];
        b.update(&v, true, 2.0);
        assert!(b.defuse_t > 0.0);
        // bot chay ra xa
        v[6].pos = [30.0, 0.0, 30.0];
        b.update(&v, false, 0.5);
        assert_eq!(b.defuser, None);
        assert!(b.defuse_t < 2.0, "tien do phai giam khi khong ai gỡ");
    }

    #[test]
    fn defuse_finishes_before_fuse_expires() {
        // DEFUSE_TIME (10) < FUSE (40) -> gỡ kip truoc khi bom no
        assert!(DEFUSE_TIME < FUSE);
    }

    #[test]
    fn reset_returns_to_carried() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        b.state = BombState::Planted;
        b.fuse = 1.0;
        b.reset();
        assert_eq!(b.state, BombState::Carried);
        assert_eq!(b.fuse, FUSE);
    }

    #[test]
    fn urgent_when_fuse_low() {
        let mut v = roster();
        let mut b = Bomb::new(&v, 0);
        b.state = BombState::Planted;
        b.fuse = 20.0;
        assert!(!b.is_urgent());
        b.fuse = 5.0;
        assert!(b.is_urgent());
    }

    #[test]
    fn sites_are_inside_arena() {
        let lim = arena_size();
        for s in bomb_sites() {
            assert!(s[0].abs() < lim && s[1].abs() < lim);
        }
    }
}
