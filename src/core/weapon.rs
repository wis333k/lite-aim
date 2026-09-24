// Bang thong so sung dung chung cho ca nguoi choi va bot AI.
// 0=Pistol 1=Rifle 2=Sniper 3=Smg
//
// Day la nguon su that nhat cho ca hai ben:
//  - `World::gun_stats()` tra ve (rpm, auto, zoom) cho nguoi choi (xem core/world.rs)
//  - `TeamFight` doc `WEAPONS[kind]` de mo phong bot (xem core/teamfight/ai.rs)

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weapon {
    /// ten hien thi
    pub name: &'static str,
    /// phat ban moi phut
    pub rpm: f32,
    /// true = giu chuot trai lien tuc ban, false = ban tung phat
    pub auto: bool,
    /// foc do zoom ADS (degree). 0 = khong zoom
    pub zoom: f32,
    /// so dan trong mot bang
    pub mag: u32,
    /// thoi gian nap dan (giay)
    pub reload: f32,
    /// sat thuong mot vien vao than
    pub dmg_body: f32,
    /// sat thuong ban trung dau
    pub dmg_head: f32,
    /// do lech ngam goc khi ban (do, 1.0 = 1 do). dung cho AI
    pub spread: f32,
    /// he so hao muc cua sung (khoang cach/100)
    pub falloff: f32,
    /// ban toi dau trong mot loat (AI dung)
    pub burst_max: u32,
    /// gian giua hai loat (giay, AI dung)
    pub burst_gap: f32,
}

pub const WEAPON_COUNT: usize = 4;

pub const WEAPONS: [Weapon; WEAPON_COUNT] = [
    // PISTOL: ban tung phat, chay, sat thuong vua, chinh xac
    Weapon {
        name: "PISTOL",
        rpm: 450.0,
        auto: false,
        zoom: 0.0,
        mag: 12,
        reload: 1.4,
        dmg_body: 28.0,
        dmg_head: 90.0,
        spread: 0.8,
        falloff: 0.6,
        burst_max: 3,
        burst_gap: 0.35,
    },
    // RIFLE: bang dan, ban lien toc, sat thuong cao
    Weapon {
        name: "RIFLE",
        rpm: 750.0,
        auto: true,
        zoom: 0.0,
        mag: 30,
        reload: 2.2,
        dmg_body: 34.0,
        dmg_head: 105.0,
        spread: 0.55,
        falloff: 0.45,
        burst_max: 6,
        burst_gap: 0.5,
    },
    // SNIPER: cham, zoom lon, ban 1 phat 1 nhanh de chet
    Weapon {
        name: "SNIPER",
        rpm: 150.0,
        auto: false,
        zoom: 28.0,
        mag: 5,
        reload: 3.0,
        dmg_body: 100.0,
        dmg_head: 150.0,
        spread: 0.15,
        falloff: 0.15,
        burst_max: 1,
        burst_gap: 1.2,
    },
    // SMG: bang dan nhat, ban nhanh, sat thuong thap, chap dap
    Weapon {
        name: "SMG",
        rpm: 1050.0,
        auto: true,
        zoom: 0.0,
        mag: 32,
        reload: 2.0,
        dmg_body: 24.0,
        dmg_head: 72.0,
        spread: 1.4,
        falloff: 0.75,
        burst_max: 9,
        burst_gap: 0.4,
    },
];

pub fn weapon(kind: u8) -> &'static Weapon {
    &WEAPONS[(kind as usize) % WEAPON_COUNT]
}

/// Giam sat thuong theo khoang cach. `d` la khoang cach ban.
pub fn damage_at(w: &Weapon, head: bool, dist: f32) -> f32 {
    let base = if head { w.dmg_head } else { w.dmg_body };
    // hao muc: sat thuong giam dan tuyen tinh theo khoang cach, toi thieu 55% sat thuong goc
    let k = 1.0 - w.falloff * (dist / 100.0);
    base * k.clamp(0.55, 1.0)
}

/// (rpm, auto, zoom) — dung cho `World::gun_stats()`.
pub fn stats_for(kind: u8) -> (f32, bool, f32) {
    let w = weapon(kind);
    (w.rpm, w.auto, w.zoom)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weapon_wraps_out_of_range_kind() {
        assert_eq!(weapon(0).name, "PISTOL");
        assert_eq!(weapon(3).name, "SMG");
        // 4 -> wraps về 0, 255 -> 3
        assert_eq!(weapon(4).name, "PISTOL");
        assert_eq!(weapon(255).name, "SMG");
    }

    #[test]
    fn stats_for_matches_legacy_gun_stats() {
        // phai khop voi bang cu trong World::gun_stats() truoc khi tach
        assert_eq!(stats_for(0), (450.0, false, 0.0));
        assert_eq!(stats_for(1), (750.0, true, 0.0));
        assert_eq!(stats_for(2), (150.0, false, 28.0));
        assert_eq!(stats_for(3), (1050.0, true, 0.0));
    }

    #[test]
    fn damage_at_head_always_beats_body() {
        let w = weapon(1);
        for d in [0.0f32, 10.0, 30.0, 50.0] {
            assert!(damage_at(w, true, d) > damage_at(w, false, d));
        }
    }

    #[test]
    fn damage_falls_off_with_distance_but_keeps_floor() {
        let w = weapon(1);
        let near = damage_at(w, false, 0.0);
        let mid = damage_at(w, false, 30.0);
        let far = damage_at(w, false, 100.0);
        assert!(near > mid, "damage phai giam o 30m");
        assert!(mid > far, "damage phai giam o 100m");
        assert!(far >= near * 0.55 - 0.001, "khong duoc xuong duoi san 55%");
    }

    #[test]
    fn damage_never_negative() {
        for w in WEAPONS.iter() {
            for d in [0.0f32, 500.0, 10000.0] {
                assert!(damage_at(w, false, d) > 0.0);
            }
        }
    }
}
