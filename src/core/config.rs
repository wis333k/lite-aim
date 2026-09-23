// config: Cfg + Data + Stats (PORT NGUYEN tu stats.rs) + quality preset

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn d_fov() -> f32 { 103.0 }
fn d_vol() -> f32 { 0.6 }
fn d_xsize() -> f32 { 1.0 }
fn d_xcolor() -> usize { 0 }
fn d_bhp() -> f32 { 100.0 }
fn d_true() -> bool { true }
fn d_quality() -> u8 { 1 }
fn d_gun_ovr() -> i32 { -1 }
fn d_map() -> usize { 0 }

#[derive(Serialize, Deserialize, Clone)]
pub struct Cfg {
    pub game_id: usize,
    pub sens: [f32; 4],
    pub dpi: [f32; 4],
    pub duration: f32,
    pub difficulty: usize,
    pub crosshair: usize,
    pub lang: usize,
    #[serde(default = "d_fov")] pub fov: f32,
    #[serde(default)] pub invert_y: bool,
    #[serde(default = "d_xcolor")] pub xhair_color: usize,
    #[serde(default = "d_xsize")] pub xhair_size: f32,
    #[serde(default = "d_true")] pub xhair_dot: bool,
    #[serde(default = "d_vol")] pub volume: f32,
    #[serde(default)] pub mute: bool,
    #[serde(default = "d_true")] pub particles: bool,
    #[serde(default = "d_true")] pub dmgnum: bool,
    #[serde(default = "d_true")] pub hitmark_on: bool,
    #[serde(default = "d_true")] pub shake_on: bool,
    #[serde(default = "d_bhp")] pub bot_hp: f32,
    #[serde(default)] pub fullscreen: bool,
    #[serde(default = "d_quality")] pub quality: u8,
    #[serde(default = "d_gun_ovr")] pub gun_override: i32,
    #[serde(default = "d_map")] pub map_id: usize,
}

impl Default for Cfg {
    fn default() -> Self {
        Cfg {
            game_id: 0,
            sens: [0.4, 1.1, 1.6, 6.0],
            dpi: [800.0, 800.0, 1600.0, 1600.0],
            duration: 60.0,
            difficulty: 1,
            crosshair: 2,
            lang: 0,
            fov: d_fov(),
            invert_y: false,
            xhair_color: d_xcolor(),
            xhair_size: d_xsize(),
            xhair_dot: true,
            volume: d_vol(),
            mute: false,
            particles: true,
            dmgnum: true,
            hitmark_on: true,
            shake_on: true,
            bot_hp: d_bhp(),
            fullscreen: false,
            quality: d_quality(),
            gun_override: d_gun_ovr(),
            map_id: d_map(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Results {
    pub mode_id: usize,
    pub mode: String,
    pub score: String,
    pub score_num: f32,
    pub acc: u32,
    pub ttk: f32,
    pub deaths: u32,
    pub hp_left: u32,
    pub shots: u32,
}

impl Results {
    pub fn new(mode_id: usize, mode: &str, score: String, score_num: f32, acc: u32, ttk: f32) -> Self {
        Results { mode_id, mode: mode.to_owned(), score, score_num, acc, ttk, deaths: 0, hp_left: 0, shots: 0 }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Data {
    pub cfg: Cfg,
    pub best: [f32; 5],
    #[serde(default)]
    pub board: Vec<ScoreEntry>,
}

impl Default for Data {
    fn default() -> Self {
        Data { cfg: Cfg::default(), best: [-1.0; 5], board: Vec::new() }
    }
}

// 1 lan choi: luu vao bang xep hang
#[derive(Serialize, Deserialize, Clone)]
pub struct ScoreEntry {
    pub mode_id: usize,
    pub mode: String,
    pub game: String,
    pub score: String,
    pub score_num: f32,
    pub acc: u32,
    #[serde(default)] pub ttk: f32,
    #[serde(default)] pub difficulty: usize,
    #[serde(default)] pub deaths: u32,
    #[serde(default)] pub ts: u64,
}

#[derive(Clone)]
pub struct Stats {
    pub cfg: Cfg,
    pub best: [f32; 5],
    pub board: Vec<ScoreEntry>,
    path: PathBuf,
    }

impl Stats {
    pub fn load() -> Self {
        let path = file_path();
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<Data>(&s).ok())
            .unwrap_or_default();
        Stats { cfg: data.cfg, best: data.best, board: data.board, path }
    }

    pub fn save(&mut self) {
        let data = Data { cfg: self.cfg.clone(), best: self.best, board: self.board.clone() };
        if let Ok(json) = serde_json::to_string_pretty(&data) {
            let _ = fs::write(&self.path, json);
        }
    }

    pub fn submit(&mut self, mode_id: usize, score_num: f32) -> bool {
        if mode_id >= 5 { return false; }
        let cur = self.best[mode_id];
        let is_new = cur < 0.0 || score_num > cur;
        if is_new {
            self.best[mode_id] = score_num;
            self.save();
        }
        is_new
    }

    // ghi 1 lan choi vao bxh, sap giam dan theo diem, giu top 100
    pub fn record(&mut self, e: ScoreEntry) {
        self.board.push(e);
        self.board.sort_by(|a, b| b.score_num.partial_cmp(&a.score_num).unwrap_or(std::cmp::Ordering::Equal));
        self.board.truncate(100);
        self.save();
    }
}

fn file_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("aimcoach_data.json");
        }
    }
    PathBuf::from("aimcoach_data.json")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_quality_high() {
        assert_eq!(Cfg::default().quality, 1);
    }

    #[test]
    fn old_save_missing_new_field_ok() {
        let json = r#"{"cfg":{"game_id":1,"sens":[1,2,3,4],"dpi":[1,2,3,4],"duration":60.0,"difficulty":1,"crosshair":2,"lang":0},"best":[-1,-1,-1,-1,-1]}"#;
        let d: Data = serde_json::from_str(json).unwrap();
        assert_eq!(d.cfg.quality, 1);
        assert_eq!(d.cfg.fov, 103.0);
    }
}
