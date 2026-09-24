// AI cho bot trong mode 5v5. 3 do kho: 0=de, 1=vua, 2=kho.
//
// Pipeline moi frame cho 1 bot:
//   1. `think()`      — nhin (co LOS?), chon muc tieu, quyet dinh trang thai
//   2. `aim()`        — xoay yaw/pitch ve muc tieu co lech ngam theo do kho
//   3. `navigate()`   — di theo A* den diem (diem lui khi can, diem dich khi tim)
//   4. `shoot()`      — ton tai bam theo burst, ton tai reaction, ton tai reload
//
// Muc tieu KHONG bao gio giai quyet: muc tieu la mot `Option<usize>` chi ra
// actor id, thay doi chi khi muc tieu cu chet / mat tia ban.

use super::actor::{Actor, AiState};
use super::nav::NavGrid;
use crate::core::math;
use crate::core::weapon;

/// Do kho.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tier {
    /// thoi gian phan ung truoc khi ban (giay) — cang lon cang kho
    pub react: f32,
    /// do lech ngam cung (do) — cang lon cang kho
    pub aim_err: f32,
    /// he so nhanh xoay (rad/s) — cang lon cang phan ung nhanh
    pub turn: f32,
    /// ti le ban trung (0..1) lam tron
    pub accuracy: f32,
    /// banh kinh nhin (met)
    pub sight: f32,
    /// goc nhin toi da (do, 0 = 360)
    pub fov: f32,
    /// ti le di chuyen
    pub speed: f32,
    /// xac nhan vi tri dung ban (giay) — bot dung lai de ban chinh xac hon
    pub settle: f32,
}

pub const TIERS: [Tier; 3] = [
    // DE: cham, lem, lem ngam, nhin hep
    Tier {
        react: 0.55,
        aim_err: 5.5,
        turn: 3.0,
        accuracy: 0.35,
        sight: 22.0,
        fov: 100.0,
        speed: 0.62,
        settle: 0.25,
    },
    // VUA
    Tier {
        react: 0.32,
        aim_err: 3.0,
        turn: 5.5,
        accuracy: 0.55,
        sight: 30.0,
        fov: 120.0,
        speed: 0.85,
        settle: 0.4,
    },
    // KHO: nhanh, chinh xac, nhin xa
    Tier {
        react: 0.16,
        aim_err: 1.4,
        turn: 9.0,
        accuracy: 0.78,
        sight: 42.0,
        fov: 150.0,
        speed: 1.0,
        settle: 0.6,
    },
];

pub fn tier(difficulty: usize) -> Tier {
    TIERS[difficulty.min(TIERS.len() - 1)]
}

/// Ket qua mot phat ban cua bot.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    pub from: usize,
    pub to: Option<usize>,
    /// ban trung dau
    pub head: bool,
    /// ban trung (luon true neu to=Some)
    pub hit: bool,
    /// co bi tuong chan khong
    pub blocked: bool,
}

fn ang_diff(a: f32, b: f32) -> f32 {
    let mut d = (a - b) % (2.0 * std::f32::consts::PI);
    if d > std::f32::consts::PI {
        d -= 2.0 * std::f32::consts::PI;
    }
    if d < -std::f32::consts::PI {
        d += 2.0 * std::f32::consts::PI;
    }
    d
}

fn dist3(a: &[f32; 3], b: &[f32; 3]) -> f32 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Bot co the nhin thay `foe` khong? (cung toi + trong goc nhin + khong bi chan)
pub fn can_see(me: &Actor, foe: &Actor, t: &Tier) -> bool {
    let eye = me.eye();
    let tgt = foe.eye();
    let d = dist3(&eye, &tgt);
    if d > t.sight {
        return false;
    }
    // goc nhin
    let fwd = me.forward();
    let dx = tgt[0] - eye[0];
    let dy = tgt[1] - eye[1];
    let dz = tgt[2] - eye[2];
    let len = d.max(0.0001);
    let cosang = (fwd[0] * dx + fwd[1] * dy + fwd[2] * dz) / len;
    let half = (t.fov * 0.5).to_radians().cos();
    if cosang < half {
        return false;
    }
    // tuong chan
    !math::segment_blocked(eye[0], eye[1], eye[2], tgt[0], tgt[1], tgt[2])
}

/// Chon muc tieu: doi phia, nho nhat, gan nhat, nhin thay duoc.
pub fn pick_target(me: &Actor, foes: &[Actor], t: &Tier, prev: Option<usize>) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
    for f in foes.iter() {
        if f.team == me.team || !f.alive {
            continue;
        }
        if !can_see(me, f, t) {
            continue;
        }
        let d = dist3(&me.pos, &f.pos);
        // uu tien muc tieu da ban: giu banh them cho de hieu
        let bias = if prev == Some(f.id) { -0.75 } else { 0.0 };
        let score = d + bias;
        if best.is_none() || score < best.unwrap().1 {
            best = Some((f.id, score));
        }
    }
    best.map(|(id, _)| id)
}

/// `world` la danh sach tat ca actor, dung de tra cua muc tieu.
pub fn foe_of<'a>(actors: &'a [Actor], _me: &Actor, id: usize) -> Option<&'a Actor> {
    actors.iter().find(|a| a.id == id)
}

/// Lean muc tieu truoc phat ban (nhung AI khong "dan" — chi dung cho cam goc).
pub struct AimOut {
    pub yaw: f32,
    pub pitch: f32,
}

pub fn aim_at(me: &Actor, foe: &Actor, _t: &Tier, want_err: [f32; 2]) -> AimOut {
    let eye = me.eye();
    let tgt = foe.eye();
    let dx = tgt[0] - eye[0];
    let dy = tgt[1] - eye[1];
    let dz = tgt[2] - eye[2];
    let horiz = (dx * dx + dz * dz).sqrt().max(0.0001);
    // forward = (-sin(yaw), pitch, -cos(yaw)) -> yaw = atan2(-dx, -dz)
    let yaw = (-dx).atan2(-dz);
    let pitch = dy.atan2(horiz);
    AimOut {
        yaw: yaw + want_err[0].to_radians(),
        pitch: pitch + want_err[1].to_radians(),
    }
}

/// Quyet dinh trang thai cho bot. Tra ve (trang thai, thoi gian ban dau can).
pub fn think(me: &mut Actor, actors: &[Actor], t: &Tier, rng: &mut dyn FnMut() -> f32) {
    if !me.alive {
        me.state = AiState::Dead;
        me.target = None;
        return;
    }
    // mat muc tieu cu -> tim lai
    let keep = me
        .target
        .map(|id| actors.iter().any(|a| a.id == id && a.alive && a.team != me.team))
        .unwrap_or(false);
    if !keep {
        me.target = None;
    }
    // thay muc tieu moi neu khong co hoac muc tieu cu mat tam nhin
    let visible_target = me
        .target
        .and_then(|id| foe_of(actors, me, id))
        .map(|f| can_see(me, f, t))
        .unwrap_or(false);
    if !visible_target {
        if let Some(id) = pick_target(me, actors, t, me.target) {
            me.target = Some(id);
        }
    }

    match me.target {
        Some(id) => {
            if let Some(f) = foe_of(actors, me, id) {
                let d = dist3(&me.pos, &f.pos);
                me.sight = d;
                if visible_target {
                    if me.state != AiState::Engage {
                        // vua chuyen sang engage -> khoi dong reaction
                        me.react_t = t.react;
                        me.burst = 0;
                        me.state_t = 0.0;
                    }
                    me.state = AiState::Engage;
                    me.state_t = 0.0;
                } else {
                    me.state = AiState::Hunt;
                    me.state_t += 0.0; // thoi gian cua Hunt do navigate tinh
                }
            }
        }
        None => {
            me.sight = 0.0;
            if me.state != AiState::Patrol {
                me.state_t = 0.0;
            }
            me.state = AiState::Patrol;
        }
    }
    // consume 1 rng de tuy chon hoa don (giai phong rng cho AI khac)
    let _ = rng();
}

/// Cap nhat huong nhin: xoay ve muc tieu, them lech, gioi han toc do xoay.
pub fn aim(me: &mut Actor, actors: &[Actor], t: &Tier, dt: f32, rng: &mut dyn FnMut() -> f32) {
    let Some(id) = me.target else {
        return;
    };
    let Some(foe) = foe_of(actors, me, id) else {
        return;
    };
    // lech ngam: chon 1 lan moi loat, giu y nguyen trong loat
    if me.burst == 0 {
        let ey = (rng() * 2.0 - 1.0) * t.aim_err;
        let ep = (rng() * 2.0 - 1.0) * t.aim_err * 0.6;
        me.aim_err = [ey, ep];
    }
    let want = aim_at(me, foe, t, me.aim_err);
    let max_step = t.turn * dt;
    let dy = ang_diff(want.yaw, me.yaw);
    let dp = want.pitch - me.pitch;
    me.yaw += dy.clamp(-max_step, max_step);
    me.pitch += dp.clamp(-max_step, max_step);
    // ghim pitch trong khoang hop le
    me.pitch = me.pitch.clamp(-1.2, 1.2);
}

/// Di chuyen theo A*. Diem dich: tien toi foe. Diem lui: lui ve phia sau
/// theo huong yaw (nếu bi ket) — don gian hoá Cover.
pub fn navigate(me: &mut Actor, actors: &[Actor], grid: &NavGrid, t: &Tier, dt: f32) {
    let speed = 4.2 * t.speed;
    let (goal, arrive) = match me.state {
        AiState::Hunt | AiState::Patrol => match me.target.and_then(|id| foe_of(actors, me, id)) {
            Some(f) => {
                let d = dist3(&me.pos, &f.pos);
                // da gan muc tieu -> dung lai, khong dich toi cham vao
                if d <= super::actor::ACTOR_STANDOFF {
                    ((me.pos[0], me.pos[2]), 0.0)
                } else {
                    ((f.pos[0], f.pos[2]), 1.5)
                }
            }
            None => {
                // Patrol: neu het duong -> chon diem ngau nhien moi
                if me.path.is_empty() || me.path_i >= me.path.len() {
                    let rx = (rng01(me.id, me.path_i, me.seen_t) * 2.0 - 1.0)
                        * (crate::core::arena::arena_size() - 3.0);
                    let rz = (rng01(me.id, me.path_i + 7, me.seen_t) * 2.0 - 1.0)
                        * (crate::core::arena::arena_size() - 3.0);
                    me.path = super::nav::find_path(grid, (me.pos[0], me.pos[2]), (rx, rz));
                    me.path_i = 0;
                }
                (me.path_target(), 0.6)
            }
        },
        AiState::Engage => {
            // giữ khoảng cách: quá gần -> lui 1 chút, nguoi lai de ban chính xac
            if let Some(f) = me.target.and_then(|id| foe_of(actors, me, id)) {
                let d = dist3(&me.pos, &f.pos);
                if d < super::actor::ACTOR_STANDOFF + 1.0 {
                    let away = (me.pos[2] - f.pos[2]).signum();
                    ((me.pos[0], me.pos[2] + away * 2.0), 0.0)
                } else {
                    ((me.pos[0], me.pos[2]), 0.0) // dung ban
                }
            } else {
                ((me.pos[0], me.pos[2]), 0.0)
            }
        }
        AiState::Cover | AiState::Dead => ((me.pos[0], me.pos[2]), 0.0),
    };

    // Neu khong phai engage va goal da doi -> tinh lai duong
    if me.state != AiState::Engage && me.path.is_empty() {
        me.path = super::nav::find_path(grid, (me.pos[0], me.pos[2]), goal);
        me.path_i = 0;
    }

    if me.state == AiState::Dead || me.path.is_empty() {
        me.moving = false;
        return;
    }
    // da den gan goal -> dung lai (tranh dut dap vao muc tieu/diem den)
    let gd = ((goal.0 - me.pos[0]).powi(2) + (goal.1 - me.pos[2]).powi(2)).sqrt();
    if gd <= arrive {
        me.path.clear();
        me.path_i = 0;
        me.moving = false;
        return;
    }
    // tien toi node hien tai
    let target = me.path[me.path_i.min(me.path.len() - 1)];
    let dx = target[0] - me.pos[0];
    let dz = target[2] - me.pos[2];
    let d = (dx * dx + dz * dz).sqrt();
    if d < 0.35 {
        me.path_i += 1;
        if me.path_i >= me.path.len() {
            me.path.clear();
            me.path_i = 0;
        }
        me.moving = false;
        return;
    }
    let nx = dx / d;
    let nz = dz / d;
    let (cx, cz) = math::collide(
        me.pos[0] + nx * speed * dt,
        me.pos[2] + nz * speed * dt,
        me.pos[1],
    );
    me.pos[0] = cx;
    me.pos[2] = cz;
    me.pos[1] = me.ground();
    me.moving = true;
    // xoay nguoi theo huong di (ngoai ra khi dang ban)
    if me.state != AiState::Engage {
        // forward = (-sin(yaw), -cos(yaw)) -> yaw = atan2(-nx, -nz)
        let want_yaw = (-nx).atan2(-nz);
        let max_step = t.turn * dt * 1.5;
        me.yaw += ang_diff(want_yaw, me.yaw).clamp(-max_step, max_step);
    }
}

impl Actor {
    fn path_target(&self) -> (f32, f32) {
        if self.path.is_empty() {
            (self.pos[0], self.pos[2])
        } else {
            let t = self.path[self.path_i.min(self.path.len() - 1)];
            (t[0], t[2])
        }
    }
}

fn rng01(id: usize, salt: usize, extra: f32) -> f32 {
    // dua tren id/salt de da phan, tranh can mot RNG chung
    let x = (id as u32)
        .wrapping_mul(2654435761)
        ^ (salt as u32).wrapping_mul(40503)
        ^ extra.to_bits();
    let mut z = x | 1;
    z ^= z << 13;
    z ^= z >> 17;
    z ^= z << 5;
    ((z >> 8) as f32) / 16_777_216.0
}

/// Ban: tra ve Shot neu co phat ban trong frame nay.
pub fn shoot(me: &mut Actor, actors: &[Actor], t: &Tier, dt: f32, rng: &mut dyn FnMut() -> f32) -> Option<Shot> {
    // dem thoi gian
    if me.fire_t > 0.0 {
        me.fire_t -= dt;
    }
    if me.reload_t > 0.0 {
        me.reload_t -= dt;
        if me.reload_t <= 0.0 {
            me.ammo = weapon::weapon(me.weapon).mag;
        }
        return None;
    }
    // reaction
    if me.react_t > 0.0 {
        me.react_t -= dt;
        return None;
    }
    // phai dang Engage moi ban
    if me.state != AiState::Engage {
        return None;
    }
    // phai co muc tieu con song
    let Some(id) = me.target else { return None };
    let Some(foe) = foe_of(actors, me, id) else { return None };
    if !foe.alive {
        return None;
    }
    // nap dan
    let w = weapon::weapon(me.weapon);
    if me.ammo == 0 {
        me.reload_t = w.reload;
        me.burst = 0;
        return None;
    }
    // giua cac loat
    if me.burst_wait > 0.0 {
        me.burst_wait -= dt;
        return None;
    }
    // ton tai ban
    if me.fire_t > 0.0 {
        return None;
    }
    me.fire_t = 60.0 / w.rpm.max(1.0);
    me.ammo -= 1;
    me.burst += 1;

    // phat ban co ban trung khong
    let eye = me.eye();
    let fwd = me.forward();
    let fwd3 = [fwd[0], fwd[1], fwd[2]];
    // tia ban lech mot chut (spread): tao 2 truc vuot goc tai, lech theo goc
    let s = w.spread * (0.4 + rng() * 0.6);
    let ang_y = (rng() * 2.0 - 1.0) * s.to_radians();
    let ang_p = (rng() * 2.0 - 1.0) * s.to_radians() * 0.5;
    let right = [fwd3[2], 0.0, -fwd3[0]]; // vuot goc nghinh truc Y
    let rl = (right[0] * right[0] + right[2] * right[2]).sqrt().max(0.0001);
    let right = [right[0] / rl, 0.0, right[2] / rl];
    let dir = [
        fwd3[0] + right[0] * ang_y,
        fwd3[1] + ang_p,
        fwd3[2] + right[2] * ang_y,
    ];
    let l = (dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2]).sqrt().max(0.0001);
    let dir = [dir[0] / l, dir[1] / l, dir[2] / l];

    // kiem tra tuong
    let t_block = math::block_forward_dist(eye[0], eye[1], eye[2], dir[0], dir[1], dir[2], 200.0);
    // test body/head foe
    let hb = super::super::arena::BOT_BODY_R;
    let hh = super::super::arena::BOT_HEAD_R;
    let t_body = math::ray_sphere(
        eye[0], eye[1], eye[2], dir[0], dir[1], dir[2],
        foe.pos[0], foe.pos[1] + super::super::arena::BOT_BODY_Y, foe.pos[2], hb,
    );
    let t_head = math::ray_sphere(
        eye[0], eye[1], eye[2], dir[0], dir[1], dir[2],
        foe.pos[0], foe.pos[1] + super::super::arena::BOT_HEAD_Y, foe.pos[2], hh,
    );
    let hit_pt = match (t_body, t_head) {
        (Some(b), Some(h)) => Some(if h < b { (h, true) } else { (b, false) }),
        (Some(b), None) => Some((b, false)),
        (None, Some(h)) => Some((h, true)),
        (None, None) => None,
    };
    let blocked = t_block < t_head.unwrap_or(t_body.unwrap_or(t_block));
    let hit = hit_pt.map(|(th, _)| th < t_block).unwrap_or(false);
    let head = hit && hit_pt.map(|(_, h)| h).unwrap_or(false);

    // xac nhan ban dung: dung mot nua xac suat
    if hit {
        if rng() > t.accuracy {
            // lech -> khong ban trung (van ton tai)
            let (dead, _) = (false, ());
            let _ = dead;
            return Some(Shot { from: me.id, to: None, head: false, hit: false, blocked });
        }
    }
    // het loat -> ngung
    if me.burst >= w.burst_max {
        me.burst = 0;
        me.burst_wait = w.burst_gap;
    }
    Some(Shot {
        from: me.id,
        to: if hit { Some(foe.id) } else { None },
        head,
        hit,
        blocked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::teamfight::actor::{Actor, TEAM_CT, TEAM_T};

    fn mk(id: usize, team: u8, x: f32, z: f32) -> Actor {
        let mut a = Actor::new(id, team, false);
        a.reset([x, 0.0, z], 0.0, 0);
        a
    }

    #[test]
    fn tier_clamps_difficulty() {
        assert_eq!(tier(0).react, 0.55);
        assert_eq!(tier(2).react, 0.16);
        assert_eq!(tier(99).react, tier(2).react);
    }

    #[test]
    fn can_see_requires_facing_and_los() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        let mut foe = mk(1, TEAM_T, 0.0, -10.0);
        let t = tier(2);
        // yaw PI -> nhin ve +Z, foe o -Z -> sau lung -> khong thay
        me.yaw = std::f32::consts::PI;
        assert!(!can_see(&me, &foe, &t));
        // xoay 180 -> yaw 0 nhin -Z, thay foe
        me.yaw = 0.0;
        assert!(can_see(&me, &foe, &t));
        // qua xa -> khong thay
        foe.pos[2] = -100.0;
        assert!(!can_see(&me, &foe, &t));
    }

    #[test]
    fn pick_target_ignores_teammates_and_dead() {
        let me = mk(0, TEAM_CT, 0.0, 0.0);
        let mut allies = vec![mk(2, TEAM_CT, 0.0, -3.0)];
        let mut foes = vec![mk(1, TEAM_T, 0.0, -10.0)];
        let mut dead = mk(3, TEAM_T, 0.0, -5.0);
        dead.alive = false;
        foes.push(dead);
        let t = tier(2);
        let mut all = vec![me.clone()];
        all.extend(allies.clone());
        all.extend(foes.clone());
        // yaw 0 -> nhin ve -Z, noi foe o -Z
        let mut me2 = me.clone();
        me2.yaw = 0.0;
        let picked = pick_target(&me2, &all, &t, None);
        assert_eq!(picked, Some(1), "phai chon foe 1, bo qua dong doi va xac");
    }

    #[test]
    fn pick_target_biases_previous() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        me.yaw = 0.0;
        let f1 = mk(1, TEAM_T, 0.0, -5.0);
        let f2 = mk(3, TEAM_T, 6.0, -5.5);
        let all = vec![me.clone(), f1, f2];
        let t = tier(2);
        // f2 xa hon, nhung bias co the thu -0.75 -> van f1
        let a = pick_target(&me, &all, &t, Some(1));
        assert_eq!(a, Some(1));
    }

    #[test]
    fn aim_points_at_foe_within_tolerance() {
        let me = mk(0, TEAM_CT, 0.0, 0.0);
        let foe = mk(1, TEAM_T, 0.0, -10.0);
        let t = tier(2);
        let out = aim_at(&me, &foe, &t, [0.0, 0.0]);
        // foe o -Z, yaw ~PI
        assert!(ang_diff(out.yaw, 0.0).abs() < 0.01);
        assert!(out.pitch.abs() < 0.01);
    }

    #[test]
    fn aim_respects_turn_rate() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        let foe = mk(1, TEAM_T, 10.0, -10.0);
        let t = tier(0);
        let actors = vec![me.clone(), foe.clone()];
        // yaw 0, foe o phia -Z/+X -> phai quay nhung gioi han buoc/frame
        let mut rng = || 0.5;
        let y0 = me.yaw;
        aim(&mut me, &actors, &t, 1.0, &mut rng);
        let moved = ang_diff(me.yaw, y0).abs();
        assert!(moved <= t.turn * 1.0 + 0.01, "khong duoc vuot qua toc do xoay: {}", moved);
    }

    #[test]
    fn shoot_waits_for_reaction() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        let mut foe = mk(1, TEAM_T, 0.0, -8.0);
        me.yaw = 0.0;
        me.state = AiState::Engage;
        me.target = Some(1);
        me.react_t = 0.5;
        let actors = vec![me.clone(), foe.clone()];
        let t = tier(0);
        let mut rng = || 0.5;
        // trong thoi gian reaction -> khong ban
        assert!(shoot(&mut me, &actors, &t, 0.1, &mut rng).is_none());
        let _ = &mut foe;
    }

    #[test]
    fn shoot_respects_fire_rate() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        let foe = mk(1, TEAM_T, 0.0, -8.0);
        me.yaw = 0.0;
        me.state = AiState::Engage;
        me.target = Some(1);
        me.react_t = 0.0;
        me.ammo = 30;
        let actors = vec![me.clone(), foe.clone()];
        let t = tier(2);
        let mut rng = || 0.5;
        let s1 = shoot(&mut me, &actors, &t, 0.016, &mut rng);
        assert!(s1.is_some(), "phai ban duoc ngay");
        // ban tiep trong 16ms -> fire_t chua het -> khong ban
        let s2 = shoot(&mut me, &actors, &t, 0.016, &mut rng);
        assert!(s2.is_none());
    }

    #[test]
    fn shoot_empty_mag_triggers_reload() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        let foe = mk(1, TEAM_T, 0.0, -8.0);
        me.yaw = 0.0;
        me.state = AiState::Engage;
        me.target = Some(1);
        me.react_t = 0.0;
        me.ammo = 0;
        me.weapon = 1;
        let actors = vec![me.clone(), foe.clone()];
        let t = tier(2);
        let mut rng = || 0.5;
        assert!(shoot(&mut me, &actors, &t, 0.016, &mut rng).is_none());
        assert!(me.reload_t > 0.0, "phai bat dau nap dan");
    }

    #[test]
    fn shoot_burst_pauses_between_groups() {
        let mut me = mk(0, TEAM_CT, 0.0, 0.0);
        let foe = mk(1, TEAM_T, 0.0, -8.0);
        me.yaw = 0.0;
        me.state = AiState::Engage;
        me.target = Some(1);
        me.react_t = 0.0;
        me.weapon = 0; // pistol burst_max 3
        me.ammo = 12;
        let actors = vec![me.clone(), foe.clone()];
        let t = tier(2);
        let mut rng = || 0.5;
        // ban 3 vien lien -> sau do phai ngung
        let mut fired = 0;
        for _ in 0..3 {
            if shoot(&mut me, &actors, &t, 0.14, &mut rng).is_some() {
                fired += 1;
            }
        }
        assert_eq!(fired, 3, "pistol ban 3 vien/loat");
        assert!(me.burst_wait > 0.0, "phai ngung giua loat");
    }

    #[test]
    fn harder_tier_aims_better() {
        // cung mot vi tri, de tier phai ban trung nhieu hon
        let mut hits_easy = 0;
        let mut hits_hard = 0;
        for (tier_i, counter) in [(0usize, &mut hits_easy), (2usize, &mut hits_hard)] {
            for s in 0..40 {
                let mut me = mk(0, TEAM_CT, 0.0, 0.0);
                let foe = mk(1, TEAM_T, 0.0, -8.0);
                me.yaw = 0.0;
                me.state = AiState::Engage;
                me.target = Some(1);
                me.react_t = 0.0;
                me.ammo = 99;
                me.weapon = 2; // sniper spread nho
                let actors = vec![me.clone(), foe.clone()];
                let t = tier(tier_i);
                // rng co dinh de so sanh cong bang
                let mut rng = || ((s * 37 % 100) as f32) / 100.0;
                if let Some(shot) = shoot(&mut me, &actors, &t, 1.0, &mut rng) {
                    if shot.hit {
                        *counter += 1;
                    }
                }
            }
        }
        assert!(hits_hard >= hits_easy, "kho phai ban trung it nhat: {} vs {}", hits_hard, hits_easy);
    }
}
