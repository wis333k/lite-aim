// core math: projection + raycast (PORT NGUYEN tu world.rs, da verify)

use super::arena::active_blocks;

// huong nhin tu yaw/pitch (do) — khop project()
pub fn forward(yaw: f32, pitch: f32) -> (f32, f32, f32) {
    let cp = pitch.to_radians().cos();
    let sp = pitch.to_radians().sin();
    let cy = yaw.to_radians().cos();
    let sy = yaw.to_radians().sin();
    (-cp * sy, sp, -cp * cy)
}

// giao ray voi AABB -> t nho nhat (>=0)
pub fn ray_aabb(
    fx: f32, fy: f32, fz: f32,
    dx: f32, dy: f32, dz: f32,
    minx: f32, miny: f32, minz: f32,
    maxx: f32, maxy: f32, maxz: f32,
) -> Option<f32> {
    let mut t0 = 0.0f32;
    let mut t1 = f32::INFINITY;
    let axes = [(fx, dx, minx, maxx), (fy, dy, miny, maxy), (fz, dz, minz, maxz)];
    for (p, d, lo, hi) in axes {
        if d.abs() < 1e-9 {
            if p < lo || p > hi {
                return None;
            }
        } else {
            let mut ta = (lo - p) / d;
            let mut tb = (hi - p) / d;
            if ta > tb {
                std::mem::swap(&mut ta, &mut tb);
            }
            if ta > t0 {
                t0 = ta;
            }
            if tb < t1 {
                t1 = tb;
            }
            if t0 > t1 {
                return None;
            }
        }
    }
    Some(t0)
}

// giao ray voi hinh cau -> t nho nhat (>=0)
pub fn ray_sphere(
    fx: f32, fy: f32, fz: f32,
    dx: f32, dy: f32, dz: f32,
    cx: f32, cy: f32, cz: f32, r: f32,
) -> Option<f32> {
    let ox = fx - cx;
    let oy = fy - cy;
    let oz = fz - cz;
    let b = ox * dx + oy * dy + oz * dz;
    let c = ox * ox + oy * oy + oz * oz - r * r;
    let disc = b * b - c;
    if disc < 0.0 {
        return None;
    }
    let t = -b - disc.sqrt();
    if t < 0.0 {
        return None;
    }
    Some(t)
}

pub fn block_forward_dist(fx: f32, fy: f32, fz: f32, dx: f32, dy: f32, dz: f32, max_d: f32) -> f32 {
    let mut best = max_d;
    for b in active_blocks().iter() {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        if let Some(t) = ray_aabb(fx, fy, fz, dx, dy, dz,
            bx - hw, by - hh, bz - hd, bx + hw, by + hh, bz + hd) {
            if t < best {
                best = t;
            }
        }
    }
    best
}

pub fn segment_blocked(fx: f32, fy: f32, fz: f32, tx: f32, ty: f32, tz: f32) -> bool {
    let dx = tx - fx;
    let dy = ty - fy;
    let dz = tz - fz;
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    if dist < 1e-6 {
        return false;
    }
    let (dx, dy, dz) = (dx / dist, dy / dist, dz / dist);
    for b in active_blocks().iter() {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        if let Some(t) = ray_aabb(fx, fy, fz, dx, dy, dz,
            bx - hw, by - hh, bz - hd, bx + hw, by + hh, bz + hd) {
            if t < dist - 1e-3 {
                return true;
            }
        }
    }
    false
}

pub fn ground_height(px: f32, pz: f32, feet: f32) -> f32 {
    let mut g = 0.0f32;
    for b in active_blocks().iter() {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        let top = by + hh;
        if top <= feet + 0.45
            && px > bx - hw - 0.25 && px < bx + hw + 0.25
            && pz > bz - hd - 0.25 && pz < bz + hd + 0.25
            && top > g
        {
            g = top;
        }
    }
    g
}

pub fn collide(px: f32, pz: f32, feet: f32) -> (f32, f32) {
    use super::arena::{arena_size, P_R};
    let lim = arena_size() - 0.5;
    let mut x = px.clamp(-lim, lim);
    let mut z = pz.clamp(-lim, lim);
    for b in active_blocks().iter() {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        if by + hh < feet + 0.35 {
            continue;
        }
        let cx = x.clamp(bx - hw, bx + hw);
        let cz = z.clamp(bz - hd, bz + hd);
        let dx = x - cx;
        let dz = z - cz;
        let d2 = dx * dx + dz * dz;
        if d2 < P_R * P_R {
            if d2 > 1e-8 {
                let d = d2.sqrt();
                x = cx + dx / d * P_R;
                z = cz + dz / d * P_R;
            } else {
                let pl = x - (bx - hw);
                let pr = (bx + hw) - x;
                let pn = z - (bz - hd);
                let pf = (bz + hd) - z;
                if pl <= pr && pl <= pn && pl <= pf {
                    x = bx - hw - P_R;
                } else if pr <= pn && pr <= pf {
                    x = bx + hw + P_R;
                } else if pn <= pf {
                    z = bz - hd - P_R;
                } else {
                    z = bz + hd + P_R;
                }
            }
        }
    }
    (x, z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_yaw0_is_minus_z() {
        let (x, y, z) = forward(0.0, 0.0);
        assert!(x.abs() < 1e-6 && y.abs() < 1e-6 && (z + 1.0).abs() < 1e-6);
    }

    #[test]
    fn ray_sphere_hit_and_miss() {
        // trai cau tam (0,0,-5) r=0.5, ban tu goc theo -Z
        assert!(ray_sphere(0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, -5.0, 0.5).is_some());
        // lech 2m -> truot
        assert!(ray_sphere(2.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, -5.0, 0.5).is_none());
        // cau sau lung -> None
        assert!(ray_sphere(0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 5.0, 0.5).is_none());
    }

    #[test]
    fn ray_aabb_hits_wall() {
        // hop x[1..2] y[-1..1] z[-11..-9]; tia tu goc theo -Z nhung x=1.5 moi trung
        assert!(ray_aabb(1.5, 0.0, 0.0, 0.0, 0.0, -1.0, 1.0, -1.0, -11.0, 2.0, 1.0, -9.0).is_some());
        // x=0 ngoai hop -> truot
        assert!(ray_aabb(0.0, 0.0, 0.0, 0.0, 0.0, -1.0, 1.0, -1.0, -11.0, 2.0, 1.0, -9.0).is_none());
        // x=7 ngoai hop [5..6] -> truot
        assert!(ray_aabb(7.0, 0.0, 0.0, 0.0, 0.0, -1.0, 5.0, -1.0, -11.0, 6.0, 1.0, -9.0).is_none());
    }

    #[test]
    fn ground_on_platform() {
        // platform [-8,0.225,-6] hw=2 hh=0.225 -> top 0.45
        let g = ground_height(-8.0, -6.0, 0.5);
        assert!((g - 0.45).abs() < 1e-4);
        // ngoai platform -> 0
        assert_eq!(ground_height(0.0, 0.0, 0.5), 0.0);
    }
}
