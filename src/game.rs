// game: settings mirror + drill hien tai + states (PORT tu App macroquad)
use crate::core::config::{Cfg, Stats};
use crate::core::drills::Drill;
use crate::core::presets::{GunKind, PRESETS};
use bevy::prelude::*;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Screen {
    #[default]
    Menu,
    Playing,
    Paused,
    Results,
    Board,
    Editor,
    Settings,
}

// dang cho nguoi dung bam phim moi cho keybind (index), None = khong cho
#[derive(Resource, Default)]
pub struct KeyWait(pub Option<usize>);
// trang thai benchmark
#[derive(Resource, Default)]
pub struct Bench {
    pub active: bool,
    pub t: f32,
    pub frames: u32,
    pub sum_dt: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub result: Option<String>,
}

#[derive(Resource)]
pub struct Game {
    pub game_id: usize,
    pub sens: [f32; 4],
    pub dpi: [f32; 4],
    pub duration: f32,
    pub difficulty: usize,
    pub crosshair: usize,
    pub lang: usize,
    pub fov: f32,
    pub invert_y: bool,
    pub xhair_color: usize,
    pub xhair_size: f32,
    pub xhair_dot: bool,
    pub volume: f32,
    pub mute: bool,
    pub particles: bool,
    pub dmgnum: bool,
    pub hitmark_on: bool,
    pub shake_on: bool,
    pub bot_hp: f32,
    pub fullscreen: bool,
    pub quality: u8,
    pub gun_override: i32,
    pub map_id: usize,
    pub fps_limit: u32,
    pub keys: [String; 10],
    pub name: String,
    pub bot_skin: usize,
    pub board_filter: usize,
    pub drill: Option<Drill>,
    pub result: Option<crate::core::config::Results>,
    pub best_flash: bool,
    pub stats: Stats,
    // mouse grab
    pub grab_active: bool,
    pub grab_hwnd: usize,
    pub cursor_hidden: bool,
    pub center: (i32, i32),
    pub pending_shot: bool,
    // ui
    pub fps_show: f32,
    pub fps_acc: f32,
    pub fps_n: u32,
    pub t_prev: f32,
    pub mode_id: usize,
}

impl Game {
    pub fn new() -> Self {
        let stats = Stats::load();
        let cfg = stats.cfg.clone();
        Game {
            game_id: cfg.game_id,
            sens: cfg.sens,
            dpi: cfg.dpi,
            duration: cfg.duration,
            difficulty: cfg.difficulty,
            crosshair: cfg.crosshair,
            lang: cfg.lang,
            fov: cfg.fov,
            invert_y: cfg.invert_y,
            xhair_color: cfg.xhair_color,
            xhair_size: cfg.xhair_size,
            xhair_dot: cfg.xhair_dot,
            volume: cfg.volume,
            mute: cfg.mute,
            particles: cfg.particles,
            dmgnum: cfg.dmgnum,
            hitmark_on: cfg.hitmark_on,
            shake_on: cfg.shake_on,
            bot_hp: cfg.bot_hp,
            fullscreen: cfg.fullscreen,
            quality: cfg.quality,
            gun_override: cfg.gun_override,
            map_id: cfg.map_id,
            fps_limit: cfg.fps_limit,
            keys: cfg.keys,
            name: cfg.name,
            bot_skin: cfg.bot_skin,
            board_filter: 5,
            drill: None,
            result: None,
            best_flash: false,
            stats,
            grab_active: false,
            grab_hwnd: 0,
            cursor_hidden: false,
            center: (0, 0),
            pending_shot: false,
            fps_show: 0.0,
            fps_acc: 0.0,
            fps_n: 0,
            t_prev: 0.0,
            mode_id: 0,
        }
    }

    pub fn save_cfg(&mut self) {
        let q = self.quality;
        self.stats.cfg = Cfg {
            game_id: self.game_id,
            sens: self.sens,
            dpi: self.dpi,
            duration: self.duration,
            difficulty: self.difficulty,
            crosshair: self.crosshair,
            lang: self.lang,
            fov: self.fov,
            invert_y: self.invert_y,
            xhair_color: self.xhair_color,
            xhair_size: self.xhair_size,
            xhair_dot: self.xhair_dot,
            volume: self.volume,
            mute: self.mute,
            particles: self.particles,
            dmgnum: self.dmgnum,
            hitmark_on: self.hitmark_on,
            shake_on: self.shake_on,
            bot_hp: self.bot_hp,
            fullscreen: self.fullscreen,
            quality: q,
            gun_override: self.gun_override,
            map_id: self.map_id,
            fps_limit: self.fps_limit,
            keys: self.keys.clone(),
            name: self.name.clone(),
            bot_skin: self.bot_skin,
        };
        self.stats.save();
    }

    pub fn deg_per_count(&self) -> f32 {
        PRESETS[self.game_id].yaw * self.sens[self.game_id]
    }

    pub fn current_gun(&self) -> GunKind {
        match self.gun_override {
            0 => GunKind::Pistol,
            1 => GunKind::Rifle,
            2 => GunKind::Sniper,
            3 => GunKind::Smg,
            _ => PRESETS[self.game_id].gun,
        }
    }

    // nhan hien thi cho nut chon sung: -1 = theo game
    pub fn gun_label(&self) -> &'static str {
        match self.gun_override {
            0 => "PISTOL",
            1 => "RIFLE",
            2 => "SNIPER",
            3 => "SMG",
            _ => "AUTO",
        }
    }

    // xoay vong: auto -> pistol -> rifle -> sniper -> smg -> auto
    pub fn cycle_gun(&mut self) {
        self.gun_override = match self.gun_override {
            -1 => 0,
            0 => 1,
            1 => 2,
            2 => 3,
            _ => -1,
        };
        self.save_cfg();
    }

    // ten map hien tai
    pub fn map_label(&self) -> &'static str {
        crate::core::arena::map_label_at(self.map_id)
    }

    // xoay vong qua tat ca map (5 preset + custom)
    pub fn cycle_map(&mut self) {
        let total = crate::core::arena::total_maps().max(1);
        self.map_id = (self.map_id + 1) % total;
        crate::core::arena::set_active_map(self.map_id);
        self.save_cfg();
    }

    // ghi ket qua vao bang xep hang lich su
    pub fn record_result(&mut self) {
        let Some(res) = self.result.clone() else { return };
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let e = crate::core::config::ScoreEntry {
            mode_id: res.mode_id,
            mode: res.mode.clone(),
            game: PRESETS[self.game_id].name.to_owned(),
            score: res.score.clone(),
            score_num: res.score_num,
            acc: res.acc,
            ttk: res.ttk,
            difficulty: self.difficulty,
            deaths: res.deaths,
            ts,
            name: self.name.clone(),
        };
        self.stats.record(e);
    }

    pub fn cm360(&self) -> f32 {
        let p = &PRESETS[self.game_id];
        crate::core::presets::cm360(p.yaw, self.sens[self.game_id], self.dpi[self.game_id])
    }

    pub fn eff_volume(&self) -> f32 {
        if self.mute { 0.0 } else { self.volume }
    }

    // fps limit -> PresentMode (Fifo = vsync cap; AutoNoVsync = khong gioi han)
    pub fn fps_label(&self) -> String {
        match self.fps_limit {
            0 => "UNLIMITED".into(),
            1 => "60".into(),
            2 => "120".into(),
            3 => "144".into(),
            4 => "240".into(),
            _ => format!("{}", self.fps_limit),
        }
    }

    pub fn cycle_fps(&mut self) {
        self.fps_limit = match self.fps_limit {
            0 => 1,
            1 => 2,
            2 => 3,
            3 => 4,
            4 => 0,
            _ => 0,
        };
        self.save_cfg();
    }

    // index keybind -> ten hien thi
    pub fn key_at(&self, i: usize) -> &str {
        self.keys.get(i).map(|s| s.as_str()).unwrap_or("")
    }

    // map ten phim (hien thi) -> KeyCode; None = khong doi
    pub fn keycode_at(&self, i: usize) -> Option<KeyCode> {
        parse_key(self.key_at(i))
    }
}

// ten phim <-> bevy KeyCode (luu dang chuoi de hien thi + serialize)
pub fn key_name(k: KeyCode) -> &'static str {
    match k {
        KeyCode::KeyA => "A", KeyCode::KeyB => "B", KeyCode::KeyC => "C",
        KeyCode::KeyD => "D", KeyCode::KeyE => "E", KeyCode::KeyF => "F",
        KeyCode::KeyG => "G", KeyCode::KeyH => "H", KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J", KeyCode::KeyK => "K", KeyCode::KeyL => "L",
        KeyCode::KeyM => "M", KeyCode::KeyN => "N", KeyCode::KeyO => "O",
        KeyCode::KeyP => "P", KeyCode::KeyQ => "Q", KeyCode::KeyR => "R",
        KeyCode::KeyS => "S", KeyCode::KeyT => "T", KeyCode::KeyU => "U",
        KeyCode::KeyV => "V", KeyCode::KeyW => "W", KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y", KeyCode::KeyZ => "Z",
        KeyCode::Digit0 => "0", KeyCode::Digit1 => "1", KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3", KeyCode::Digit4 => "4", KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6", KeyCode::Digit7 => "7", KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::ShiftLeft => "Shift", KeyCode::ControlLeft => "Ctrl",
        KeyCode::Space => "Space", KeyCode::Tab => "Tab",
        KeyCode::Escape => "Esc", KeyCode::Enter => "Enter",
        KeyCode::F1 => "F1", KeyCode::F2 => "F2", KeyCode::F3 => "F3",
        KeyCode::F4 => "F4", KeyCode::F5 => "F5", KeyCode::F6 => "F6",
        _ => "?",
    }
}

pub fn parse_key(s: &str) -> Option<KeyCode> {
    Some(match s {
        "A" => KeyCode::KeyA, "B" => KeyCode::KeyB, "C" => KeyCode::KeyC,
        "D" => KeyCode::KeyD, "E" => KeyCode::KeyE, "F" => KeyCode::KeyF,
        "G" => KeyCode::KeyG, "H" => KeyCode::KeyH, "I" => KeyCode::KeyI,
        "J" => KeyCode::KeyJ, "K" => KeyCode::KeyK, "L" => KeyCode::KeyL,
        "M" => KeyCode::KeyM, "N" => KeyCode::KeyN, "O" => KeyCode::KeyO,
        "P" => KeyCode::KeyP, "Q" => KeyCode::KeyQ, "R" => KeyCode::KeyR,
        "S" => KeyCode::KeyS, "T" => KeyCode::KeyT, "U" => KeyCode::KeyU,
        "V" => KeyCode::KeyV, "W" => KeyCode::KeyW, "X" => KeyCode::KeyX,
        "Y" => KeyCode::KeyY, "Z" => KeyCode::KeyZ,
        "0" => KeyCode::Digit0, "1" => KeyCode::Digit1, "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3, "4" => KeyCode::Digit4, "5" => KeyCode::Digit5,
        "6" => KeyCode::Digit6, "7" => KeyCode::Digit7, "8" => KeyCode::Digit8,
        "9" => KeyCode::Digit9,
        "Shift" => KeyCode::ShiftLeft, "Ctrl" => KeyCode::ControlLeft,
        "Space" => KeyCode::Space, "Tab" => KeyCode::Tab,
        "Esc" => KeyCode::Escape, "Enter" => KeyCode::Enter,
        "F1" => KeyCode::F1, "F2" => KeyCode::F2, "F3" => KeyCode::F3,
        "F4" => KeyCode::F4, "F5" => KeyCode::F5, "F6" => KeyCode::F6,
        _ => return None,
    })
}

impl Game {
    // tao drill theo mode_id + cau hinh hien tai
    pub fn start_mode(&mut self) {
        // mode 5v5 (mode_id >= 5) luon dung map 5v5: 5 map cu chi co 1 khu
        // spawn nen khong choi duoc. Nguoi choi van doi map trong menu duoc.
        if self.mode_id >= 5 {
            // map 5v5 cuoi cung la index 5..10 -> giu nguyen
            self.map_id = if self.map_id >= 5 {
                self.map_id
            } else {
                crate::core::arena::MAP_DF_PORT + (self.map_id % 5)
            };
        }
        crate::core::arena::set_active_map(self.map_id);
        let p = &PRESETS[self.game_id];
        let dur = self.duration;
        let diff = self.difficulty;
        let hp = self.bot_hp;
        self.drill = Some(match self.mode_id {
            0 => Drill::Gridshot(crate::core::drills::Gridshot::new(dur)),
            1 => Drill::Flick(crate::core::drills::Flick::new(dur)),
            2 => Drill::Tracking(crate::core::drills::Tracking::new(dur)),
            3 => Drill::Recoil(crate::core::drills::Recoil::new(dur, p.recoil_mag, p.recoil_spread)),
            4 => Drill::Duel(crate::core::drills::Duel::new(dur, diff, hp)),
            5 => Drill::TeamFight(crate::core::teamfight::fight::TeamFight::new(
                dur,
                diff,
                self.current_gun() as u8,
                crate::core::arena::current_map().name,
            )),
            _ => Drill::TeamFight(crate::core::teamfight::fight::TeamFight::new_bomb(
                diff,
                self.current_gun() as u8,
                crate::core::arena::current_map().name,
            )),
        });
        self.best_flash = false;
        self.result = None;
    }

    pub fn release_grab(&mut self) {
        if self.grab_active {
            self.grab_active = false;
            self.cursor_hidden = false;
        }
    }

    pub fn set_cursor_visible(&mut self, vis: bool) {
        self.cursor_hidden = !vis;
    }
}
