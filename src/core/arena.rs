// arena constants + geometry helpers (ported from world.rs, logic verified)

pub const ARENA_SIZE: f32 = 15.0;
/// ban nho (map luyen tap 30x30)
pub const ARENA_SMALL: f32 = 15.0;
/// ban 5v5 (44x44) — cho du khoang cho 10 nguoi
pub const ARENA_BIG: f32 = 22.0;
pub const ARENA_WALL_H: f32 = 6.0;

/// Kich thuoc arena cua map dang active. 5 map 5v5 (id >= 5) dung ban lon.
pub fn arena_size() -> f32 {
    if active_map_index() >= MAP_DF_PORT {
        ARENA_BIG
    } else {
        ARENA_SMALL
    }
}

pub const EYE_STAND: f32 = 1.6;
pub const EYE_CROUCH: f32 = 1.0;
pub const P_R: f32 = 0.4;

pub const BOT_BODY_Y: f32 = 1.05;
pub const BOT_BODY_R: f32 = 0.60;
pub const BOT_HEAD_Y: f32 = 1.92;
pub const BOT_HEAD_R: f32 = 0.30;
pub const DMG_BODY: f32 = 34.0;
pub const DMG_HEAD: f32 = 68.0;

// [x, y, z(center), half_w, half_h, half_d]
pub const DUEL_BLOCKS: [[f32; 6]; 12] = [
    [-3.2, 0.55, -7.5, 0.9, 0.55, 0.35],
    [2.8, 0.55, -9.5, 0.9, 0.55, 0.35],
    [0.0, 0.75, -11.5, 1.4, 0.75, 0.40],
    [-5.6, 0.55, -10.5, 0.7, 0.55, 0.70],
    [5.4, 0.55, -12.5, 0.7, 0.55, 0.70],
    [-8.0, 0.225, -6.0, 2.0, 0.225, 2.0],
    [8.0, 0.225, -6.0, 2.0, 0.225, 2.0],
    [-8.0, 0.675, -9.5, 1.5, 0.225, 1.5],
    [8.0, 0.675, -9.5, 1.5, 0.225, 1.5],
    [0.0, 0.5, -7.0, 2.5, 0.5, 0.40],
    [-6.5, 1.5, -13.5, 0.8, 1.5, 0.80],
    [6.5, 1.5, -13.5, 0.8, 1.5, 0.80],
];

// vat can tinh (duel): [x, y, z, half_w, half_h, half_d]
pub const DUEL_EXTRA: [[f32; 6]; 10] = [
    [-1.8, 0.30, -8.8, 0.55, 0.30, 0.55],
    [1.9, 0.30, -11.8, 0.55, 0.30, 0.55],
    [-4.4, 0.45, -12.6, 0.60, 0.45, 0.60],
    [4.6, 0.45, -8.4, 0.60, 0.45, 0.60],
    [-11.5, 0.90, -8.0, 0.55, 0.90, 0.55],
    [11.5, 0.90, -8.0, 0.55, 0.90, 0.55],
    [-10.5, 0.30, -12.0, 1.20, 0.30, 0.50],
    [10.5, 0.30, -12.0, 1.20, 0.30, 0.50],
    [-3.0, 1.30, -4.5, 0.45, 1.30, 0.45],
    [3.0, 1.30, -4.5, 0.45, 1.30, 0.45],
];

// prop trang tri (luon hien): [x, y, z, kind, scale]
// kind: 0=cot tron, 1=thung, 2=container, 3=cot vuong
pub const ARENA_PROPS: [[f32; 5]; 14] = [
    [-13.0, 1.6, 9.0, 0.0, 1.6],
    [13.0, 1.6, 9.0, 0.0, 1.6],
    [-13.0, 1.6, -9.0, 0.0, 1.6],
    [13.0, 1.6, -9.0, 0.0, 1.6],
    [-13.5, 1.0, 0.0, 1.0, 1.0],
    [13.5, 1.0, 0.0, 1.0, 1.0],
    [-13.5, 0.9, 4.5, 2.0, 0.9],
    [13.5, 0.9, 4.5, 2.0, 0.9],
    [-13.5, 0.9, -4.5, 2.0, 0.9],
    [13.5, 0.9, -4.5, 2.0, 0.9],
    [-9.0, 0.75, 12.5, 3.0, 0.75],
    [9.0, 0.75, 12.5, 3.0, 0.75],
    [-11.0, 1.2, -13.0, 1.0, 1.2],
    [11.0, 1.2, -13.0, 1.0, 1.2],
];

// ===================== 5 MAP =====================
// Moi map gom: blocks (vat can, co va cham + dung de leo) va props (trang tri, chi hien).
pub const MAP_DUEL: usize = 0;
pub const MAP_WAREHOUSE: usize = 1;
pub const MAP_LANES: usize = 2;
pub const MAP_PILLARS: usize = 3;
pub const MAP_RANGE: usize = 4;

pub struct MapDef {
    pub name: &'static str,
    pub name_vi: &'static str,
    pub blocks: &'static [[f32; 6]],
    pub extras: &'static [[f32; 6]],
    pub props: &'static [[f32; 5]],
    pub theme: usize,
}

// --- Map 1: WAREHOUSE: thung + container xep tang ---
const WH_BLOCKS: [[f32; 6]; 14] = [
    [-6.0, 0.50, -6.0, 1.5, 0.50, 1.5],
    [6.0, 0.50, -6.0, 1.5, 0.50, 1.5],
    [-6.0, 0.50, 6.0, 1.5, 0.50, 1.5],
    [6.0, 0.50, 6.0, 1.5, 0.50, 1.5],
    [0.0, 0.40, 0.0, 2.5, 0.40, 1.0],
    [-10.5, 0.50, 0.0, 1.0, 0.50, 3.0],
    [10.5, 0.50, 0.0, 1.0, 0.50, 3.0],
    [-3.0, 1.50, -11.0, 2.0, 1.50, 0.5],
    [3.0, 1.50, -11.0, 2.0, 1.50, 0.5],
    [-2.0, 0.30, -4.0, 0.8, 0.30, 0.8],
    [2.0, 0.30, -4.0, 0.8, 0.30, 0.8],
    [0.0, 2.20, -6.0, 4.0, 0.20, 1.2],
    [-9.0, 1.60, -8.0, 0.8, 1.60, 0.8],
    [9.0, 1.60, -8.0, 0.8, 1.60, 0.8],
];
const WH_EXTRAS: [[f32; 6]; 6] = [
    [-4.0, 0.25, -8.5, 1.0, 0.25, 1.0],
    [4.0, 0.25, -8.5, 1.0, 0.25, 1.0],
    [0.0, 0.75, -9.5, 1.5, 0.75, 0.5],
    [-7.5, 0.40, -3.0, 0.6, 0.40, 0.6],
    [7.5, 0.40, -3.0, 0.6, 0.40, 0.6],
    [0.0, 0.20, -3.0, 3.0, 0.20, 0.6],
];
const WH_PROPS: [[f32; 5]; 16] = [
    [-13.0, 1.6, 9.0, 0.0, 1.6],
    [13.0, 1.6, 9.0, 0.0, 1.6],
    [-13.0, 1.6, -9.0, 0.0, 1.6],
    [13.0, 1.6, -9.0, 0.0, 1.6],
    [-12.5, 1.0, 0.0, 1.0, 1.0],
    [12.5, 1.0, 0.0, 1.0, 1.0],
    [-13.0, 0.9, 5.0, 2.0, 0.9],
    [13.0, 0.9, 5.0, 2.0, 0.9],
    [-13.0, 0.9, -5.0, 2.0, 0.9],
    [13.0, 0.9, -5.0, 2.0, 0.9],
    [-8.0, 0.75, 12.5, 3.0, 0.75],
    [8.0, 0.75, 12.5, 3.0, 0.75],
    [-11.0, 1.2, -13.5, 1.0, 1.2],
    [11.0, 1.2, -13.5, 1.0, 1.2],
    [-2.0, 1.2, 12.5, 1.0, 1.2],
    [2.0, 1.2, 12.5, 1.0, 1.2],
];

// --- Map 2: LANES: 3 lang ban doc ---
const LN_BLOCKS: [[f32; 6]; 12] = [
    // vach ngan 3 lang
    [-5.2, 1.50, 0.0, 0.3, 1.50, 12.0],
    [5.2, 1.50, 0.0, 0.3, 1.50, 12.0],
    // bien moi lang
    [-11.0, 1.50, 0.0, 0.3, 1.50, 12.0],
    [11.0, 1.50, 0.0, 0.3, 1.50, 12.0],
    // cover giua lang
    [-8.0, 0.55, -4.0, 1.2, 0.55, 0.6],
    [-8.0, 0.55, 4.0, 1.2, 0.55, 0.6],
    [0.0, 0.55, -4.0, 1.2, 0.55, 0.6],
    [0.0, 0.55, 4.0, 1.2, 0.55, 0.6],
    [8.0, 0.55, -4.0, 1.2, 0.55, 0.6],
    [8.0, 0.55, 4.0, 1.2, 0.55, 0.6],
    [0.0, 1.50, -11.5, 5.0, 1.50, 0.5],
    [0.0, 1.50, 11.5, 5.0, 1.50, 0.5],
];
const LN_EXTRAS: [[f32; 6]; 8] = [
    [-8.0, 0.25, -9.0, 1.0, 0.25, 1.0],
    [0.0, 0.25, -9.0, 1.0, 0.25, 1.0],
    [8.0, 0.25, -9.0, 1.0, 0.25, 1.0],
    [-8.0, 0.25, 9.0, 1.0, 0.25, 1.0],
    [0.0, 0.25, 9.0, 1.0, 0.25, 1.0],
    [8.0, 0.25, 9.0, 1.0, 0.25, 1.0],
    [-8.0, 0.70, 0.0, 0.8, 0.70, 0.8],
    [8.0, 0.70, 0.0, 0.8, 0.70, 0.8],
];
const LN_PROPS: [[f32; 5]; 12] = [
    [-13.5, 1.6, -8.0, 0.0, 1.6],
    [-13.5, 1.6, 8.0, 0.0, 1.6],
    [13.5, 1.6, -8.0, 0.0, 1.6],
    [13.5, 1.6, 8.0, 0.0, 1.6],
    [-13.5, 1.0, 0.0, 1.0, 1.0],
    [13.5, 1.0, 0.0, 1.0, 1.0],
    [-13.5, 0.9, 4.0, 2.0, 0.9],
    [13.5, 0.9, 4.0, 2.0, 0.9],
    [-13.5, 0.9, -4.0, 2.0, 0.9],
    [13.5, 0.9, -4.0, 2.0, 0.9],
    [-6.0, 1.2, -13.5, 1.0, 1.2],
    [6.0, 1.2, -13.5, 1.0, 1.2],
];

// --- Map 3: PILLARS: nhieu cot + platform cao ---
const PL_BLOCKS: [[f32; 6]; 16] = [
    [-4.0, 1.50, -4.0, 0.6, 1.50, 0.6],
    [4.0, 1.50, -4.0, 0.6, 1.50, 0.6],
    [-4.0, 1.50, 4.0, 0.6, 1.50, 0.6],
    [4.0, 1.50, 4.0, 0.6, 1.50, 0.6],
    [0.0, 1.50, -8.0, 0.6, 1.50, 0.6],
    [0.0, 1.50, 8.0, 0.6, 1.50, 0.6],
    [-8.0, 1.50, 0.0, 0.6, 1.50, 0.6],
    [8.0, 1.50, 0.0, 0.6, 1.50, 0.6],
    [-8.0, 0.30, -8.0, 2.0, 0.30, 2.0],
    [8.0, 0.30, -8.0, 2.0, 0.30, 2.0],
    [-8.0, 0.30, 8.0, 2.0, 0.30, 2.0],
    [8.0, 0.30, 8.0, 2.0, 2.00, 2.0],
    [0.0, 0.90, -11.0, 3.0, 0.90, 0.5],
    [0.0, 0.50, 0.0, 2.0, 0.50, 2.0],
    [-11.0, 0.50, -11.0, 0.8, 0.50, 0.8],
    [11.0, 0.50, -11.0, 0.8, 0.50, 0.8],
];
const PL_EXTRAS: [[f32; 6]; 6] = [
    [-6.0, 0.25, -10.0, 1.0, 0.25, 1.0],
    [6.0, 0.25, -10.0, 1.0, 0.25, 1.0],
    [-10.0, 0.25, -2.0, 1.0, 0.25, 1.0],
    [10.0, 0.25, -2.0, 1.0, 0.25, 1.0],
    [-2.0, 0.60, -6.0, 0.6, 0.60, 0.6],
    [2.0, 0.60, -6.0, 0.6, 0.60, 0.6],
];
const PL_PROPS: [[f32; 5]; 14] = [
    [-13.0, 1.6, 9.0, 0.0, 1.6],
    [13.0, 1.6, 9.0, 0.0, 1.6],
    [-13.0, 1.6, -9.0, 0.0, 1.6],
    [13.0, 1.6, -9.0, 0.0, 1.6],
    [-13.5, 1.0, 0.0, 1.0, 1.0],
    [13.5, 1.0, 0.0, 1.0, 1.0],
    [-13.5, 1.6, 4.5, 0.0, 1.6],
    [13.5, 1.6, 4.5, 0.0, 1.6],
    [-13.5, 1.6, -4.5, 0.0, 1.6],
    [13.5, 1.6, -4.5, 0.0, 1.6],
    [-8.0, 0.9, 12.5, 2.0, 0.9],
    [8.0, 0.9, 12.5, 2.0, 0.9],
    [-2.0, 1.2, 13.0, 1.0, 1.2],
    [2.0, 1.2, 13.0, 1.0, 1.2],
];

// --- Map 4: OPEN RANGE: trong, chi vai cover thap (tot cho tracking/flick) ---
const RG_BLOCKS: [[f32; 6]; 8] = [
    [-6.0, 0.40, -6.0, 1.0, 0.40, 1.0],
    [6.0, 0.40, -6.0, 1.0, 0.40, 1.0],
    [-6.0, 0.40, 6.0, 1.0, 0.40, 1.0],
    [6.0, 0.40, 6.0, 1.0, 0.40, 1.0],
    [0.0, 0.40, -10.0, 2.0, 0.40, 0.6],
    [0.0, 0.40, 10.0, 2.0, 0.40, 0.6],
    [-12.0, 1.20, 0.0, 0.6, 1.20, 3.0],
    [12.0, 1.20, 0.0, 0.6, 1.20, 3.0],
];
const RG_EXTRAS: [[f32; 6]; 4] = [
    [-9.0, 0.25, -9.0, 1.0, 0.25, 1.0],
    [9.0, 0.25, -9.0, 1.0, 0.25, 1.0],
    [-9.0, 0.25, 9.0, 1.0, 0.25, 1.0],
    [9.0, 0.25, 9.0, 1.0, 0.25, 1.0],
];
const RG_PROPS: [[f32; 5]; 10] = [
    [-13.5, 1.6, -10.0, 0.0, 1.6],
    [13.5, 1.6, -10.0, 0.0, 1.6],
    [-13.5, 1.6, 10.0, 0.0, 1.6],
    [13.5, 1.6, 10.0, 0.0, 1.6],
    [-13.5, 0.9, 0.0, 2.0, 0.9],
    [13.5, 0.9, 0.0, 2.0, 0.9],
    [-13.5, 1.0, 5.0, 1.0, 1.0],
    [13.5, 1.0, 5.0, 1.0, 1.0],
    [-13.5, 1.0, -5.0, 1.0, 1.0],
    [13.5, 1.0, -5.0, 1.0, 1.0],
];

// ===================== 5 MAP 5V5 =====================
// Phong bi 2 khu spawn doi nhau tren truc Z, giua la vat che tam nhin.
// LUU Y: vat che phai nam trong `blocks` (khong phai `extras`) de no
// cham, chan dan va chan duong nhin. `extras` chi de trang tri.
pub const MAP_DF_PORT: usize = 5;
pub const MAP_DF_FOUNDRY: usize = 6;
pub const MAP_DF_CARGO: usize = 7;
pub const MAP_DF_YARD: usize = 8;
pub const MAP_DF_SILO: usize = 9;
/// tong so map (5 cu + 5 map 5v5)
pub const MAP_COUNT: usize = 10;

// --- DF-PORT: 2 khoang cua ben, giua la khoang mo ---
pub const PT_BLOCKS: [[f32; 6]; 16] = [
    // tuong ben A (z ~ -7) cat nguon nhin sang B, de xa khu spawn
    [-9.00, 1.5, -10.50, 3.0, 1.5, 0.5],
    [9.00, 1.5, -10.50, 3.0, 1.5, 0.5],
    // cong giua 2 ben
    [-2.40, 1.2, 0.00, 0.5, 1.2, 3.0],
    [2.40, 1.2, 0.00, 0.5, 1.2, 3.0],
    // hop cao o 2 ben
    [-14.25, 0.75, -7.50, 1.2, 0.75, 1.2],
    [14.25, 0.75, -7.50, 1.2, 0.75, 1.2],
    [-14.25, 0.75, 7.50, 1.2, 0.75, 1.2],
    [14.25, 0.75, 7.50, 1.2, 0.75, 1.2],
    // bong container chan duong giua
    [0.00, 0.9, -9.75, 2.0, 0.9, 0.9],
    [0.00, 0.9, 9.75, 2.0, 0.9, 0.9],
    // bac thang de leo len bo
    [-6.00, 0.225, 0.00, 1.2, 0.225, 1.2],
    [6.00, 0.225, 0.00, 1.2, 0.225, 1.2],
    [-6.00, 0.675, 0.00, 1.0, 0.225, 1.0],
    [6.00, 0.675, 0.00, 1.0, 0.225, 1.0],
    // tuong lui an cap
    [-16.50, 1.0, 0.00, 0.5, 1.0, 2.5],
    [16.50, 1.0, 0.00, 0.5, 1.0, 2.5],
];
pub const PT_EXTRAS: [[f32; 6]; 8] = [
    [-4.50, 0.30, -12.00, 0.55, 0.30, 0.55],
    [4.80, 0.30, 12.00, 0.55, 0.30, 0.55],
    [-10.50, 0.45, 12.75, 0.60, 0.45, 0.60],
    [10.80, 0.45, -12.75, 0.60, 0.45, 0.60],
    [0.00, 0.30, 14.25, 1.20, 0.30, 0.50],
    [0.00, 0.30, -14.25, 1.20, 0.30, 0.50],
    [-18.00, 0.90, 12.00, 0.55, 0.90, 0.55],
    [18.00, 0.90, -12.00, 0.55, 0.90, 0.55],
];
pub const PT_PROPS: [[f32; 5]; 10] = [
    [-19.50, 1.6, 16.50, 0.0, 1.6],
    [19.50, 1.6, 16.50, 0.0, 1.6],
    [-19.50, 1.6, -16.50, 0.0, 1.6],
    [19.50, 1.6, -16.50, 0.0, 1.6],
    [-20.25, 1.0, 0.00, 1.0, 1.0],
    [20.25, 1.0, 0.00, 1.0, 1.0],
    [-20.25, 0.9, 6.75, 2.0, 0.9],
    [20.25, 0.9, -6.75, 2.0, 0.9],
    [-9.00, 0.75, 18.75, 3.0, 0.75],
    [9.00, 0.75, -18.75, 3.0, 0.75],
];

// --- DF-FOUNDRY: nhieu cot, khu gac cao ---
pub const FD_BLOCKS: [[f32; 6]; 18] = [
    [-12.00, 1.5, -12.00, 0.6, 1.5, 0.6],
    [12.00, 1.5, -12.00, 0.6, 1.5, 0.6],
    [-12.00, 1.5, 12.00, 0.6, 1.5, 0.6],
    [12.00, 1.5, 12.00, 0.6, 1.5, 0.6],
    [0.00, 1.5, -12.00, 0.6, 1.5, 0.6],
    [0.00, 1.5, 12.00, 0.6, 1.5, 0.6],
    // san cai che goc ban
    [-7.50, 1.0, -3.00, 0.5, 1.0, 3.0],
    [7.50, 1.0, 3.00, 0.5, 1.0, 3.0],
    [-7.50, 1.0, 3.00, 0.5, 1.0, 3.0],
    [7.50, 1.0, -3.00, 0.5, 1.0, 3.0],
    // bong lon chan tim
    [0.00, 1.2, 0.00, 2.0, 1.2, 2.0],
    // khu gac cao 2 ben
    [-15.75, 0.45, 0.00, 1.5, 0.45, 1.5],
    [15.75, 0.45, 0.00, 1.5, 0.45, 1.5],
    [-15.75, 1.125, 0.00, 1.2, 0.225, 1.2],
    [15.75, 1.125, 0.00, 1.2, 0.225, 1.2],
    // hop thap
    [-4.50, 0.30, -9.00, 0.7, 0.30, 0.7],
    [4.50, 0.30, 9.00, 0.7, 0.30, 0.7],
    [0.00, 0.30, 8.25, 1.4, 0.30, 0.5],
];
pub const FD_EXTRAS: [[f32; 6]; 8] = [
    [-9.75, 0.30, 7.50, 0.55, 0.30, 0.55],
    [9.75, 0.30, -7.50, 0.55, 0.30, 0.55],
    [-3.00, 0.45, 14.25, 0.60, 0.45, 0.60],
    [3.30, 0.45, -14.25, 0.60, 0.45, 0.60],
    [-16.50, 0.90, 9.00, 0.55, 0.90, 0.55],
    [16.50, 0.90, -9.00, 0.55, 0.90, 0.55],
    [0.00, 0.30, -17.25, 1.20, 0.30, 0.50],
    [0.00, 0.30, 17.25, 1.20, 0.30, 0.50],
];
pub const FD_PROPS: [[f32; 5]; 10] = [
    [-19.50, 1.6, 16.50, 0.0, 1.6],
    [19.50, 1.6, 16.50, 0.0, 1.6],
    [-19.50, 1.6, -16.50, 0.0, 1.6],
    [19.50, 1.6, -16.50, 0.0, 1.6],
    [-20.25, 1.0, 0.00, 1.0, 1.0],
    [20.25, 1.0, 0.00, 1.0, 1.0],
    [-20.25, 0.9, 6.75, 2.0, 0.9],
    [20.25, 0.9, -6.75, 2.0, 0.9],
    [-10.50, 0.75, 18.75, 3.0, 0.75],
    [10.50, 0.75, -18.75, 3.0, 0.75],
];

// --- DF-CARGO: hang container, duong hanh hep ---
pub const CG_BLOCKS: [[f32; 6]; 19] = [
    // 3 hang container doc X, cat giua
    [-9.00, 1.1, -9.00, 2.5, 1.1, 0.9],
    [9.00, 1.1, -9.00, 2.5, 1.1, 0.9],
    [0.00, 1.1, 0.00, 2.5, 1.1, 0.9],
    [-9.00, 1.1, 9.00, 2.5, 1.1, 0.9],
    [9.00, 1.1, 9.00, 2.5, 1.1, 0.9],
    // hai khe hep giua cac hang
    [-4.50, 0.5, -4.50, 0.4, 0.5, 2.5],
    [4.50, 0.5, 4.50, 0.4, 0.5, 2.5],
    // bac chong container 2 tang
    [0.00, 1.9, -4.50, 1.2, 0.8, 1.2],
    [0.00, 1.9, 4.50, 1.2, 0.8, 1.2],
    // hop keo
    [-14.25, 0.60, -13.50, 0.7, 0.60, 0.7],
    [14.25, 0.60, -13.50, 0.7, 0.60, 0.7],
    [-14.25, 0.60, 13.50, 0.7, 0.60, 0.7],
    [14.25, 0.60, 13.50, 0.7, 0.60, 0.7],
    // tuong chan bien
    [-18.00, 1.2, -4.50, 0.5, 1.2, 3.0],
    [18.00, 1.2, -4.50, 0.5, 1.2, 3.0],
    [-18.00, 1.2, 4.50, 0.5, 1.2, 3.0],
    [18.00, 1.2, 4.50, 0.5, 1.2, 3.0],
    // san thap giua
    [0.00, 0.45, 0.00, 1.0, 0.45, 1.0],
    [0.00, 0.225, -12.00, 2.0, 0.225, 1.5],
];
pub const CG_EXTRAS: [[f32; 6]; 8] = [
    [-6.00, 0.30, 14.25, 0.55, 0.30, 0.55],
    [6.30, 0.30, -14.25, 0.55, 0.30, 0.55],
    [-11.25, 0.45, 1.50, 0.60, 0.45, 0.60],
    [11.25, 0.45, -1.50, 0.60, 0.45, 0.60],
    [0.00, 0.30, 18.00, 1.20, 0.30, 0.50],
    [0.00, 0.30, -18.00, 1.20, 0.30, 0.50],
    [-16.50, 0.90, 9.75, 0.55, 0.90, 0.55],
    [16.50, 0.90, -9.75, 0.55, 0.90, 0.55],
];
pub const CG_PROPS: [[f32; 5]; 10] = [
    [-20.25, 1.6, 17.25, 0.0, 1.6],
    [20.25, 1.6, 17.25, 0.0, 1.6],
    [-20.25, 1.6, -17.25, 0.0, 1.6],
    [20.25, 1.6, -17.25, 0.0, 1.6],
    [-20.25, 1.0, 0.00, 1.0, 1.0],
    [20.25, 1.0, 0.00, 1.0, 1.0],
    [-20.25, 0.9, 6.75, 2.0, 0.9],
    [20.25, 0.9, -6.75, 2.0, 0.9],
    [-12.00, 0.75, 18.75, 3.0, 0.75],
    [12.00, 0.75, -18.75, 3.0, 0.75],
];

// --- DF-YARD: san mo, it vat che, nhieu duong lap rung ---
pub const YD_BLOCKS: [[f32; 6]; 14] = [
    // vai cuc nho rai rac (che nhin nhung khong chan hoan toan)
    [-10.50, 0.8, -6.00, 0.8, 0.8, 0.8],
    [10.50, 0.8, 6.00, 0.8, 0.8, 0.8],
    [-6.00, 0.6, 10.50, 0.7, 0.6, 0.7],
    [6.00, 0.6, -10.50, 0.7, 0.6, 0.7],
    [0.00, 0.9, -12.00, 1.2, 0.9, 0.8],
    [0.00, 0.9, 12.00, 1.2, 0.9, 0.8],
    // goc cao 2 ben
    [-16.50, 0.675, -12.00, 1.5, 0.675, 1.5],
    [16.50, 0.675, -12.00, 1.5, 0.675, 1.5],
    [-16.50, 0.675, 12.00, 1.5, 0.675, 1.5],
    [16.50, 0.675, 12.00, 1.5, 0.675, 1.5],
    // cau cao
    [-4.50, 0.30, 0.00, 0.7, 0.30, 0.7],
    [4.50, 0.30, 0.00, 0.7, 0.30, 0.7],
    [-9.00, 0.45, 3.75, 0.7, 0.45, 0.7],
    [9.00, 0.45, -3.75, 0.7, 0.45, 0.7],
];
pub const YD_EXTRAS: [[f32; 6]; 8] = [
    [-3.00, 0.30, 9.00, 0.55, 0.30, 0.55],
    [3.30, 0.30, -9.00, 0.55, 0.30, 0.55],
    [-12.00, 0.45, 0.00, 0.60, 0.45, 0.60],
    [12.00, 0.45, 0.00, 0.60, 0.45, 0.60],
    [0.00, 0.30, 16.50, 1.20, 0.30, 0.50],
    [0.00, 0.30, -16.50, 1.20, 0.30, 0.50],
    [-18.00, 0.90, 6.00, 0.55, 0.90, 0.55],
    [18.00, 0.90, -6.00, 0.55, 0.90, 0.55],
];
pub const YD_PROPS: [[f32; 5]; 10] = [
    [-19.50, 1.6, 18.00, 0.0, 1.6],
    [19.50, 1.6, 18.00, 0.0, 1.6],
    [-19.50, 1.6, -18.00, 0.0, 1.6],
    [19.50, 1.6, -18.00, 0.0, 1.6],
    [-20.25, 1.0, 0.00, 1.0, 1.0],
    [20.25, 1.0, 0.00, 1.0, 1.0],
    [-20.25, 0.9, 6.75, 2.0, 0.9],
    [20.25, 0.9, -6.75, 2.0, 0.9],
    [-9.00, 0.75, 19.50, 3.0, 0.75],
    [9.00, 0.75, -19.50, 3.0, 0.75],
];

// --- DF-SILO: khoang tron, bo thap cao, ham thu ---
pub const SL_BLOCKS: [[f32; 6]; 18] = [
    // vòng bo silo (cho xuyen qua khe), keo vao giua de khong chan spawn
    [0.00, 2.0, -12.75, 3.0, 2.0, 0.6],
    [0.00, 2.0, 12.75, 3.0, 2.0, 0.6],
    [-4.50, 2.0, 0.00, 0.6, 2.0, 3.0],
    [4.50, 2.0, 0.00, 0.6, 2.0, 3.0],
    // khe giua vong cho 2 duong
    [0.00, 2.0, -7.50, 3.0, 2.0, 0.6],
    [0.00, 2.0, 7.50, 3.0, 2.0, 0.6],
    // cot giua
    [0.00, 1.2, 0.00, 1.0, 1.2, 1.0],
    // be thap de lui an
    [-12.00, 0.45, -7.50, 1.0, 0.45, 1.0],
    [12.00, 0.45, -7.50, 1.0, 0.45, 1.0],
    [-12.00, 0.45, 7.50, 1.0, 0.45, 1.0],
    [12.00, 0.45, 7.50, 1.0, 0.45, 1.0],
    // thap cao
    [-8.25, 0.90, 0.00, 0.7, 0.90, 0.7],
    [8.25, 0.90, 0.00, 0.7, 0.90, 0.7],
    [-16.50, 1.5, 0.00, 0.6, 1.5, 2.5],
    [16.50, 1.5, 0.00, 0.6, 1.5, 2.5],
    // hop chan
    [-3.75, 0.55, -11.25, 0.7, 0.55, 0.7],
    [3.75, 0.55, 11.25, 0.7, 0.55, 0.7],
    [0.00, 0.225, 0.00, 2.0, 0.225, 2.0],
];
pub const SL_EXTRAS: [[f32; 6]; 8] = [
    [-9.00, 0.30, 12.00, 0.55, 0.30, 0.55],
    [9.00, 0.30, -12.00, 0.55, 0.30, 0.55],
    [-13.50, 0.45, 12.00, 0.60, 0.45, 0.60],
    [13.50, 0.45, -12.00, 0.60, 0.45, 0.60],
    [0.00, 0.30, 18.75, 1.20, 0.30, 0.50],
    [0.00, 0.30, -18.75, 1.20, 0.30, 0.50],
    [-18.00, 0.90, 6.00, 0.55, 0.90, 0.55],
    [18.00, 0.90, -6.00, 0.55, 0.90, 0.55],
];
pub const SL_PROPS: [[f32; 5]; 10] = [
    [-20.25, 1.6, 18.00, 0.0, 1.6],
    [20.25, 1.6, 18.00, 0.0, 1.6],
    [-20.25, 1.6, -18.00, 0.0, 1.6],
    [20.25, 1.6, -18.00, 0.0, 1.6],
    [-20.25, 1.0, 0.00, 1.0, 1.0],
    [20.25, 1.0, 0.00, 1.0, 1.0],
    [-20.25, 0.9, 6.75, 2.0, 0.9],
    [20.25, 0.9, -6.75, 2.0, 0.9],
    [-10.50, 0.75, 19.50, 3.0, 0.75],
    [10.50, 0.75, -19.50, 3.0, 0.75],
];

pub const MAPS: [MapDef; MAP_COUNT] = [
    MapDef { name: "DUEL ARENA", name_vi: "DAU TRUONG", blocks: &DUEL_BLOCKS, extras: &DUEL_EXTRA, props: &ARENA_PROPS, theme: 0 },
    MapDef { name: "WAREHOUSE",  name_vi: "KHO HANG",   blocks: &WH_BLOCKS,   extras: &WH_EXTRAS,   props: &WH_PROPS,   theme: 1 },
    MapDef { name: "LANES",      name_vi: "BA LAN",     blocks: &LN_BLOCKS,   extras: &LN_EXTRAS,   props: &LN_PROPS,   theme: 2 },
    MapDef { name: "PILLARS",    name_vi: "COT TRU",    blocks: &PL_BLOCKS,   extras: &PL_EXTRAS,   props: &PL_PROPS,   theme: 3 },
    MapDef { name: "OPEN RANGE", name_vi: "SAN TRONG",  blocks: &RG_BLOCKS,   extras: &RG_EXTRAS,   props: &RG_PROPS,   theme: 4 },
    // --- 5 map rieng cho 5v5 ---
    MapDef { name: "DF-PORT",    name_vi: "CAI PORT",   blocks: &PT_BLOCKS,   extras: &PT_EXTRAS,   props: &PT_PROPS,   theme: 0 },
    MapDef { name: "DF-FOUNDRY", name_vi: "NHOM THOI",  blocks: &FD_BLOCKS,   extras: &FD_EXTRAS,   props: &FD_PROPS,   theme: 1 },
    MapDef { name: "DF-CARGO",   name_vi: "KHO HANG 2", blocks: &CG_BLOCKS,   extras: &CG_EXTRAS,   props: &CG_PROPS,   theme: 2 },
    MapDef { name: "DF-YARD",    name_vi: "SAN NGOAI",  blocks: &YD_BLOCKS,   extras: &YD_EXTRAS,   props: &YD_PROPS,   theme: 3 },
    MapDef { name: "DF-SILO",    name_vi: "KHO SILO",   blocks: &SL_BLOCKS,   extras: &SL_EXTRAS,   props: &SL_PROPS,   theme: 4 },
];

// theme mau block dang dung (0..N)
static ACTIVE_THEME: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn set_active_theme(i: usize) {
    ACTIVE_THEME.store(i, std::sync::atomic::Ordering::Relaxed);
}

pub fn active_theme() -> usize {
    ACTIVE_THEME.load(std::sync::atomic::Ordering::Relaxed)
}

// map dang chon, de toan bo module core (khong co Resource) doc duoc.
static ACTIVE_MAP: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

// danh sach map custom nguoi dung tu tao (nap tu maps.json), rong luc dau.
static CUSTOM_MAPS: std::sync::OnceLock<Vec<MapDef>> = std::sync::OnceLock::new();

pub fn set_active_map(i: usize) {
    ACTIVE_MAP.store(i, std::sync::atomic::Ordering::Relaxed);
}

/// Blocks cua map dang active (dung cho test + AI).
pub fn current_map_blocks() -> &'static [[f32; 6]] {
    current_map().blocks
}

/// id map dang active (doc tuong ung, dung cho chon skybox).
pub fn active_map_index() -> usize {
    ACTIVE_MAP.load(std::sync::atomic::Ordering::Relaxed)
}

/// Khoá cho test: `ACTIVE_MAP` la global nen cac test doi map phai chay
/// tuần tự, khong duoc chay song song.
#[cfg(test)]
pub fn map_test_lock() -> std::sync::MutexGuard<'static, ()> {
    use std::sync::OnceLock;
    static L: OnceLock<std::sync::Mutex<()>> = OnceLock::new();
    L.get_or_init(|| std::sync::Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

pub fn active_map() -> usize {
    ACTIVE_MAP.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn custom_maps() -> &'static Vec<MapDef> {
    CUSTOM_MAPS.get_or_init(Vec::new)
}

pub fn total_maps() -> usize {
    MAP_COUNT + custom_maps().len()
}

// map dang chon (preset hoac custom)
pub fn current_map() -> &'static MapDef {
    let i = active_map();
    if i < MAP_COUNT {
        &MAPS[i]
    } else {
        let ci = (i - MAP_COUNT).min(custom_maps().len().saturating_sub(1));
        &custom_maps()[ci]
    }
}

pub fn active_blocks() -> &'static [[f32; 6]] {
    current_map().blocks
}

pub fn active_props() -> &'static [[f32; 5]] {
    current_map().props
}

pub fn active_map_theme() -> usize {
    current_map().theme
}

pub fn map_label_at(i: usize) -> &'static str {
    if i < MAP_COUNT {
        MAPS[i].name
    } else {
        let ci = i - MAP_COUNT;
        custom_maps().get(ci).map(|m| m.name).unwrap_or("CUSTOM")
    }
}

// ---- luu / nap map custom (maps.json) ----
fn maps_path() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("maps.json");
        }
    }
    std::path::PathBuf::from("maps.json")
}

// nap maps.json vao CUSTOM_MAPS (goi 1 lan luc startup)
pub fn load_custom_maps() {
    let mut list: Vec<MapDef> = Vec::new();
    if let Ok(s) = std::fs::read_to_string(maps_path()) {
        if let Ok(arr) = serde_json::from_str::<Vec<CustomMap>>(&s) {
            for m in arr {
                let blocks: &'static [[f32; 6]] = Box::leak(m.blocks.into_boxed_slice());
                let extras: &'static [[f32; 6]] = Box::leak(m.extras.into_boxed_slice());
                let props: &'static [[f32; 5]] = Box::leak(m.props.into_boxed_slice());
                let name: &'static str = Box::leak(m.name.into_boxed_str());
                list.push(MapDef {
                    name,
                    name_vi: name,
                    blocks,
                    extras,
                    props,
                    theme: m.theme,
                });
            }
        }
    }
    let _ = CUSTOM_MAPS.set(list);
}

// ghi them 1 map custom vao maps.json (append, giu map cu)
pub fn save_custom_map(name: &str, blocks: &[[f32; 6]], theme: usize) -> bool {
    let mut arr: Vec<CustomMap> = std::fs::read_to_string(maps_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    arr.push(CustomMap {
        name: name.to_owned(),
        blocks: blocks.to_vec(),
        extras: Vec::new(),
        props: Vec::new(),
        theme,
    });
    if let Ok(json) = serde_json::to_string_pretty(&arr) {
        return std::fs::write(maps_path(), json).is_ok();
    }
    false
}

#[derive(serde::Serialize, serde::Deserialize)]
struct CustomMap {
    name: String,
    blocks: Vec<[f32; 6]>,
    #[serde(default)] extras: Vec<[f32; 6]>,
    #[serde(default)] props: Vec<[f32; 5]>,
    #[serde(default)] theme: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 5 map 5v5 phai co du so vat che (khong phai san trong) va 2 khu
    /// spawn phai ket noi duoc bang duong di.
    const TF_MAPS: [usize; 5] = [MAP_DF_PORT, MAP_DF_FOUNDRY, MAP_DF_CARGO, MAP_DF_YARD, MAP_DF_SILO];

    #[test]
    fn total_maps_is_ten() {
        assert_eq!(MAP_COUNT, 10);
        assert_eq!(MAPS.len(), 10);
    }

    #[test]
    fn each_tf_map_has_cover() {
        for id in TF_MAPS {
            let m = &MAPS[id];
            assert!(m.blocks.len() >= 12, "map {} it vat che: {}", id, m.blocks.len());
        }
    }

    #[test]
    fn each_tf_map_has_blocks_inside_arena() {
        let _lock = map_test_lock();
        for id in TF_MAPS {
            set_active_map(id);
            let lim = arena_size();
            for b in MAPS[id].blocks.iter() {
                let (x, _, z, hw, _, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
                assert!(
                    (x - hw).abs() < lim && (x + hw).abs() < lim,
                    "map {} block x={} vuot arena {}", id, x, lim
                );
                assert!(
                    (z - hd).abs() < lim && (z + hd).abs() < lim,
                    "map {} block z={} vuot arena {}", id, z, lim
                );
            }
        }
        set_active_map(0);
    }

    #[test]
    fn spawn_zones_are_free_of_cover() {
        let _lock = map_test_lock();
        // 2 khu spawn nam o 2 dau arena: |x| <= 6, gan 2 ben tuong
        for id in TF_MAPS {
            set_active_map(id);
            let lim = arena_size();
            let zone_z = lim - 3.0; // spawn nam o day
            for b in MAPS[id].blocks.iter() {
                let (x, _, z, hw, _, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
                // vat che nao cham vao vung spawn (|x|<=6.5 va |z|>=zone_z-1.5) -> loi
                let in_spawn = x.abs() <= 6.5 && (z.abs() as f32) >= zone_z - 1.5;
                assert!(
                    !in_spawn,
                    "map {} co vat che chan spawn tai ({}, {})", id, x, z
                );
                let _ = (hw, hd);
            }
        }
        set_active_map(0);
    }

    #[test]
    fn tf_maps_have_sightline_breakers_across_mid() {
        // phai co it nhat 1 vat che o |z| < 8 de cat duong nhin giua 2 khu
        for id in TF_MAPS {
            let mid = MAPS[id]
                .blocks
                .iter()
                .filter(|b| b[2].abs() < 8.0 && b[4] >= 0.8)
                .count();
            assert!(mid >= 2, "map {} thieu vat che cat nhin giua: {}", id, mid);
        }
    }
}
