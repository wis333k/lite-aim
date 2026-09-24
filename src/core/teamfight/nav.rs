// Nav: luoi di chuyen + A* cho bot AI trong mode 5v5.
//
// Luoi duoc dung cho MOT map trong bo nho (rebuild khi doi map).
// O moi cell giu `walk` (di duoc) va `cost` (chi phi de them cho terrain cao).
// A* dung 8 huong (ke ca cheo) cham vao tuong bang collision AABB hien co.

use crate::core::arena::{active_blocks, arena_size};

pub const CELL: f32 = 1.0;
/// ban kinh lam no o moi huong de tranh bot "dinh" vao canh tuong


pub struct NavGrid {
    pub n: usize,
    /// vi tri cell (i, j) -> toa do the gioi
    pub min_x: f32,
    pub min_z: f32,
    pub walk: Vec<bool>,
    /// do cao dat tai cell (dung de bot leo len ban)
    pub height: Vec<f32>,
    /// chi phi buoc them khi cell cao hon cell hien tai
    pub climb: Vec<f32>,
}

impl NavGrid {
    pub fn cell_center(&self, i: usize, j: usize) -> (f32, f32) {
        (
            self.min_x + (i as f32 + 0.5) * CELL,
            self.min_z + (j as f32 + 0.5) * CELL,
        )
    }

    pub fn idx(&self, i: usize, j: usize) -> usize {
        j * self.n + i
    }

    pub fn in_bounds(&self, i: i32, j: i32) -> bool {
        i >= 0 && j >= 0 && (i as usize) < self.n && (j as usize) < self.n
    }

    pub fn walkable(&self, i: i32, j: i32) -> bool {
        if !self.in_bounds(i, j) {
            return false;
        }
        self.walk[self.idx(i as usize, j as usize)]
    }

    /// cell chua vi tri the gioi cho bat ky. Luon trong luoi.
    pub fn cell_of(&self, x: f32, z: f32) -> (i32, i32) {
        let i = ((x - self.min_x) / CELL).floor() as i32;
        let j = ((z - self.min_z) / CELL).floor() as i32;
        let i = i.clamp(0, self.n as i32 - 1);
        let j = j.clamp(0, self.n as i32 - 1);
        (i, j)
    }

    /// diem di chuyen gan nhat ma khong nam trong vat can.
    /// quet vong tron quanh cell, toi da 8 vong.
    pub fn nearest_walkable(&self, x: f32, z: f32) -> (i32, i32) {
        let (ci, cj) = self.cell_of(x, z);
        if self.walkable(ci, cj) {
            return (ci, cj);
        }
        for r in 1..=8i32 {
            for dj in -r..=r {
                for di in -r..=r {
                    if di.abs() != r && dj.abs() != r {
                        continue; // chi vong shell
                    }
                    if self.walkable(ci + di, cj + dj) {
                        return (ci + di, cj + dj);
                    }
                }
            }
        }
        (ci, cj)
    }
}

/// Dung NavGrid cho map dang active.
pub fn build() -> NavGrid {
    let lim = arena_size() - 0.5;
    let n = ((lim * 2.0) / CELL).ceil() as usize;
    let min_x = -lim;
    let min_z = -lim;
    let cells = n * n;
    let mut grid = NavGrid {
        n,
        min_x,
        min_z,
        walk: vec![true; cells],
        height: vec![0.0; cells],
        climb: vec![0.0; cells],
    };
    // chieu cao dat + cham vat: dung 2 lan so phep
    for j in 0..n {
        for i in 0..n {
            let (x, z) = grid.cell_center(i, j);
            let idx = grid.idx(i, j);
            // dat dat: thap nhat de bot khong bi "leo" len khong co
            let g = crate::core::math::ground_height(x, z, 10.0);
            grid.height[idx] = g;
            // vat cham o giua nguoi: dung ham collide cu chinh game de khop hanh vi
            let (cx, cz) = crate::core::math::collide(x, z, g);
            let pushed = (cx - x).abs() > 0.01 || (cz - z).abs() > 0.01;
            grid.walk[idx] = !pushed;
        }
    }
    // tru do cao leo gan ke: cho phép leo <= 0.55m (chan ~0.55m, dung cho crate thap)
    for j in 0..n {
        for i in 0..n {
            let idx = grid.idx(i, j);
            if !grid.walk[idx] {
                continue;
            }
            let h = grid.height[idx];
            let mut step = 0.0f32;
            for (di, dj) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1)] {
                if !grid.walkable(i as i32 + di, j as i32 + dj) {
                    continue;
                }
                let hn = grid.height[grid.idx((i as i32 + di) as usize, (j as i32 + dj) as usize)];
                let d = (hn - h).abs();
                if d > step && d <= 0.55 {
                    step = d;
                }
            }
            grid.climb[idx] = step;
        }
    }
    let _ = active_blocks();
    grid
}

/// A* tra ve duong di (world-space) tu `from` den `to`.
/// rong neu khong tim thay duong.
pub fn find_path(grid: &NavGrid, from: (f32, f32), to: (f32, f32)) -> Vec<[f32; 3]> {
    let (si, sj) = grid.nearest_walkable(from.0, from.1);
    let (gi, gj) = grid.nearest_walkable(to.0, to.1);
    if !grid.walkable(si, sj) || !grid.walkable(gi, gj) {
        return Vec::new();
    }
    if (si, sj) == (gi, gj) {
        let (x, z) = grid.cell_center(si as usize, sj as usize);
        return vec![[x, grid.height[grid.idx(si as usize, sj as usize)], z]];
    }

    let cells = grid.n * grid.n;
    let start = grid.idx(si as usize, sj as usize);
    let goal = grid.idx(gi as usize, gj as usize);

    // g: chi phi tot nhat tim duoc, f: uoc luong, came: da dong
    let mut g = vec![f32::INFINITY; cells];
    let mut f = vec![f32::INFINITY; cells];
    let mut came = vec![usize::MAX; cells];
    let mut closed = vec![false; cells];
    let mut open = std::collections::BinaryHeap::new();

    let heur = |i: usize, j: usize| -> f32 {
        let (gi, gj) = (i as f32 - gi as f32, j as f32 - gj as f32);
        (gi * gi + gj * gj).sqrt() * CELL
    };

    g[start] = 0.0;
    f[start] = heur(si as usize, sj as usize);
    open.push(HeapItem { f: f[start], idx: start });

    let dirs: [(i32, i32, f32); 8] = [
        (1, 0, 1.0),
        (-1, 0, 1.0),
        (0, 1, 1.0),
        (0, -1, 1.0),
        (1, 1, 1.4142),
        (1, -1, 1.4142),
        (-1, 1, 1.4142),
        (-1, -1, 1.4142),
    ];

    while let Some(HeapItem { idx: cur, .. }) = open.pop() {
        if cur == goal {
            break;
        }
        if closed[cur] {
            continue;
        }
        closed[cur] = true;
        let ci = (cur % grid.n) as i32;
        let cj = (cur / grid.n) as i32;
        let ch = grid.height[cur];
        for (di, dj, step) in dirs.iter() {
            let ni = ci + *di;
            let nj = cj + *dj;
            if !grid.walkable(ni, nj) {
                continue;
            }
            // khong cut qua goc (diag) — chan chéo giua 2 o chan
            if *di != 0 && *dj != 0 {
                if !grid.walkable(ci + *di, cj) || !grid.walkable(ci, cj + *dj) {
                    continue;
                }
            }
            let nidx = grid.idx(ni as usize, nj as usize);
            if closed[nidx] {
                continue;
            }
            let nh = grid.height[nidx];
            if (nh - ch).abs() > 0.55 {
                continue; // qua cao -> khong leo
            }
            let cost = g[cur] + step * CELL + grid.climb[nidx];
            if cost < g[nidx] {
                g[nidx] = cost;
                came[nidx] = cur;
                let fnew = cost + heur(ni as usize, nj as usize);
                f[nidx] = fnew;
                open.push(HeapItem { f: fnew, idx: nidx });
            }
        }
    }

    if came[goal] == usize::MAX && goal != start {
        return Vec::new();
    }
    // dung lai
    let mut out = Vec::new();
    let mut cur = goal;
    let mut guard = 0;
    loop {
        let i = cur % grid.n;
        let j = cur / grid.n;
        let (x, z) = grid.cell_center(i, j);
        out.push([x, grid.height[cur], z]);
        if cur == start {
            break;
        }
        let prev = came[cur];
        if prev == usize::MAX || guard > grid.n * grid.n {
            out.clear();
            break;
        }
        cur = prev;
        guard += 1;
    }
    out.reverse();
    out
}

// BinaryHeap la max-heap -> Reverse de lay uu tien f nho nhat
#[derive(Clone, Copy, PartialEq)]
struct HeapItem {
    f: f32,
    idx: usize,
}
impl Eq for HeapItem {}
impl Ord for HeapItem {
    fn cmp(&self, o: &Self) -> std::cmp::Ordering {
        // f nho hon = uu tien cao hon; idx nho lam tie-break de on dinh
        o.f.partial_cmp(&self.f).unwrap_or(std::cmp::Ordering::Equal)
            .then(o.idx.cmp(&self.idx))
    }
}
impl PartialOrd for HeapItem {
    fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(o))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// map la global nen moi test khoa + gan map co dinh de khong race
    fn map0() -> std::sync::MutexGuard<'static, ()> {
        let g = crate::core::arena::map_test_lock();
        crate::core::arena::set_active_map(0);
        g
    }

    #[test]
    fn build_produces_in_bounds_grid() {
        let _g = map0();
        let g = build();
        assert!(g.n > 10, "luoi phai co it nhat 10x10, n={}", g.n);
        assert_eq!(g.walk.len(), g.n * g.n);
        assert!(g.nearest_walkable(0.0, 0.0).0 >= 0);
    }

    #[test]
    fn cell_roundtrip() {
        let _g = map0();
        let g = build();
        let (x, z) = g.cell_center(3, 4);
        let (i, j) = g.cell_of(x, z);
        assert_eq!((i, j), (3, 4));
    }

    #[test]
    fn path_open_map_is_direct() {
        let _g = map0();
        let g = build();
        let p = find_path(&g, (0.0, 0.0), (5.0, 0.0));
        assert!(!p.is_empty(), "phai tim thay duong tren san trong");
        assert!(p.len() <= 12, "duong qua san trong phai ngan, n={}", p.len());
    }

    #[test]
    fn path_same_cell_returns_one_node() {
        let _g = map0();
        let g = build();
        let (i, j) = g.nearest_walkable(0.0, 0.0);
        let (x, z) = g.cell_center(i as usize, j as usize);
        let p = find_path(&g, (x, z), (x + 0.1, z + 0.1));
        assert_eq!(p.len(), 1);
    }

    #[test]
    fn path_ends_at_goal_neighborhood() {
        let _g = map0();
        let g = build();
        let p = find_path(&g, (0.0, 0.0), (-4.0, -3.0));
        assert!(!p.is_empty(), "phai tim thay duong");
        let last = p[p.len() - 1];
        let d = ((last[0] + 4.0).powi(2) + (last[2] + 3.0).powi(2)).sqrt();
        assert!(d < 1.8, "dau phai gan dich, d={}", d);
    }

    #[test]
    fn path_start_point_not_inside_wall() {
        let _g = map0();
        let g = build();
        let a = arena_size();
        let p = find_path(&g, (a - 0.2, 0.0), (0.0, 0.0));
        for node in p.iter() {
            assert!(node[0].abs() <= a && node[2].abs() <= a);
        }
    }
}
