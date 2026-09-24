// ui: menu / HUD / pause / results — Bevy UI (thay immediate-mode macroquad)
use crate::game::{Game, Screen};
use crate::render::crosshair_color;
use bevy::prelude::*;

// Aim Lab style: nen gan den, accent tim + xanh cyan, panel xam tim
pub const COL_ACC: Color = Color::srgb(0.66, 0.33, 0.97);       // purple #a855f7
pub const COL_ACC2: Color = Color::srgb(0.13, 0.83, 0.93);      // cyan #22d3ee
pub const COL_PANEL: Color = Color::srgba(0.071, 0.071, 0.102, 0.98);
pub const COL_PANEL_HOVER: Color = Color::srgba(0.12, 0.12, 0.18, 0.98);
pub const COL_ACTIVE: Color = Color::srgba(0.36, 0.18, 0.58, 1.0);   // purple dim
pub const COL_TXT: Color = Color::srgb(0.96, 0.96, 0.98);
pub const COL_DIM: Color = Color::srgb(0.52, 0.52, 0.62);
pub const COL_BG: Color = Color::srgb(0.039, 0.039, 0.059);     // #0a0a0f
pub const COL_LINE: Color = Color::srgb(0.14, 0.14, 0.20);

#[derive(Component)]
pub struct UiRoot;

#[derive(Component)]
pub struct Crosshair;

#[derive(Component)]
pub struct CrosshairPart;

#[derive(Component)]
pub struct ScopeOverlay;

#[derive(Component)]
pub struct HurtVignette;

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct HudScore;

#[derive(Component)]
pub struct HudMode;

#[derive(Component)]
pub struct HudHp;

#[derive(Component)]
pub struct DeathMsg;

#[derive(Component)]
pub struct HudRoot;

#[derive(Component)]
pub struct ModeButton(pub usize);
#[derive(Component)]
pub struct GameButton(pub usize);
#[derive(Component)]
pub struct StartButton;
#[derive(Component)]
pub struct QuitButton;
#[derive(Component)]
pub struct LangButton;
#[derive(Component)]
pub struct QualityButton;
#[derive(Component)]
pub struct ResumeButton;
#[derive(Component)]
pub struct MenuButton;
#[derive(Component)]
pub struct RetryButton;
#[derive(Component)]
pub struct BoardButton;
#[derive(Component)]
pub struct GunButton;
#[derive(Component)]
pub struct MapButton;
#[derive(Component)]
pub struct EditorButton;
#[derive(Component)]
pub struct SettingsButton;
#[derive(Component)]
pub struct AboutButton;
#[derive(Component)]
pub struct BoardFilterButton(pub usize);
// settings +/- chinh tri so
#[derive(Component)]
pub struct SetMinus(pub usize);
#[derive(Component)]
pub struct SetPlus(pub usize);
// keybind button (index)
#[derive(Component)]
pub struct KeyBindButton(pub usize);
// benchmark
#[derive(Component)]
pub struct BenchButton;
#[derive(Component)]
pub struct BenchText;

// nav item o sidebar doc (trang tri / mo rong sau)
#[derive(Component)]
pub struct NavigationButton(pub usize);
pub const SET_FOV: usize = 0;
pub const SET_SENS: usize = 1;
pub const SET_DPI: usize = 2;
pub const SET_VOL: usize = 3;
pub const SET_XCOL: usize = 4;
pub const SET_XSIZE: usize = 5;
pub const SET_INVERT: usize = 6;
pub const SET_XDOT: usize = 7;
pub const SET_FPS: usize = 8;
pub const SET_QUALITY: usize = 9;
pub const SET_GUN: usize = 10;
pub const SET_SKIN: usize = 11;
pub const SET_COUNT: usize = 12;

pub const KEY_NAMES: [&str; 10] = [
    "Forward", "Back", "Left", "Right", "Sprint", "Jump", "Crouch", "Reload", "SwapGun", "Pause",
];
pub const KEY_VI: [&str; 10] = [
    "Tien", "Lui", "Trai", "Phai", "Chay", "Nhay", "Ngoi", "Nap", "Doi sung", "Tam dung",
];

#[derive(Resource, Default)]
pub struct UiRes {
    pub dirty: bool,
    pub last: Option<Screen>,
}

// mo url tren Windows
pub fn open_url(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/c", "start", "", url]).spawn();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

// ap dung +/- cho tung setting (id = SET_*)
pub fn apply_setting(game: &mut Game, id: usize, plus: bool) {
    let s = if plus { 1.0 } else { -1.0 };
    match id {
        SET_FOV => game.fov = (game.fov + s * 2.0).clamp(60.0, 120.0),
        SET_SENS => {
            let g = game.game_id;
            game.sens[g] = (game.sens[g] + s * 0.05).clamp(0.02, 20.0);
        }
        SET_DPI => {
            let g = game.game_id;
            game.dpi[g] = (game.dpi[g] + s * 50.0).clamp(100.0, 12800.0);
        }
        SET_VOL => game.volume = (game.volume + s * 0.05).clamp(0.0, 1.0),
        SET_XCOL => {
            let n = 6i32;
            game.xhair_color = (((game.xhair_color as i32 + if plus { 1 } else { -1 }) % n + n) % n) as usize;
        }
        SET_XSIZE => game.xhair_size = (game.xhair_size + s * 0.2).clamp(0.4, 3.0),
        SET_INVERT => game.invert_y = !game.invert_y,
        SET_XDOT => game.xhair_dot = !game.xhair_dot,
        SET_FPS => game.cycle_fps(),
        SET_QUALITY => game.quality = 1 - game.quality,
        SET_GUN => {
            game.gun_override = match game.gun_override {
                -1 => 0, 0 => 1, 1 => 2, 2 => 3, _ => -1,
            };
        }
        SET_SKIN => {
            let n = crate::render::BOT_SKINS.len();
            game.bot_skin = if plus { (game.bot_skin + 1) % n } else { (game.bot_skin + n - 1) % n };
        }
        _ => {}
    }
    game.save_cfg();
}

// system: bat phim moi cho keybind khi dang cho
pub fn keybind_capture(
    mut key_wait: ResMut<crate::game::KeyWait>,
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut ui_res: ResMut<UiRes>,
) {
    let Some(idx) = key_wait.0 else { return };

    // 99 = dang sua ten nguoi choi
    if idx == 99 {
        let mut name = game.name.clone();
        for k in keys.get_just_pressed() {
            match k {
                KeyCode::Escape | KeyCode::Enter => {
                    if !name.is_empty() {
                        game.name = name.clone();
                        game.save_cfg();
                    }
                    key_wait.0 = None;
                    ui_res.dirty = true;
                    return;
                }
                KeyCode::Backspace => { name.pop(); }
                _ => {}
            }
        }
        // ky tu chu: lay qua ten phim A-Z / 0-9
        for k in keys.get_just_pressed() {
            let nm = crate::game::key_name(*k);
            if nm.len() == 1 {
                let c = nm.chars().next().unwrap();
                if c.is_ascii_alphanumeric() && name.len() < 12 {
                    name.push(c);
                }
            }
        }
        if name.is_empty() { name = "P".into(); }
        game.name = name;
        ui_res.dirty = true;
        return;
    }

    for k in keys.get_just_pressed() {
        if *k == KeyCode::Escape { key_wait.0 = None; ui_res.dirty = true; return; }
        let nm = crate::game::key_name(*k);
        if nm != "?" {
            game.keys[idx] = nm.to_owned();
            game.save_cfg();
            key_wait.0 = None;
            ui_res.dirty = true;
            return;
        }
    }
}

fn text_node(s: &str, size: f32, col: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont { font_size: size, ..default() },
        TextColor(col),
    )
}

fn panel(px: f32, py: f32, active: bool) -> impl Bundle {
    (
        Button,
        BtnIdle(if active { COL_ACTIVE } else { COL_PANEL }),
        Node {
            padding: UiRect::axes(Val::Px(px), Val::Px(py)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor::all(if active { COL_ACC } else { COL_LINE }),
        BackgroundColor(if active { COL_ACTIVE } else { COL_PANEL }),
    )
}

#[derive(Component)]
pub struct BtnIdle(pub Color);

// nav item sidebar: cot doc, size co dinh
fn nav_panel(w: f32, h: f32, active: bool) -> impl Bundle {
    (
        Button,
        BtnIdle(if active { COL_ACTIVE } else { COL_PANEL }),
        Node {
            width: Val::Px(w),
            height: Val::Px(h),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            ..default()
        },
        BorderColor::all(if active { COL_ACC } else { COL_LINE }),
        BackgroundColor(if active { COL_ACTIVE } else { COL_PANEL }),
    )
}
// doi mau nut khi hover (giu mau nen goc qua BtnIdle)
pub fn hover_buttons(
    mut q: Query<
        (&Interaction, &BtnIdle, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (inter, idle, mut bg, mut border) in q.iter_mut() {
        match *inter {
            Interaction::Hovered => {
                bg.0 = COL_PANEL_HOVER;
                *border = BorderColor::all(COL_ACC);
            }
            Interaction::Pressed => {
                bg.0 = COL_ACC;
                *border = BorderColor::all(COL_ACC);
            }
            Interaction::None => {
                bg.0 = idle.0;
                *border = BorderColor::all(COL_LINE);
            }
        }
    }
}

fn panel_acc(px: f32, py: f32) -> impl Bundle {
    (
        Button,
        BtnIdle(COL_ACC),
        Node {
            padding: UiRect::axes(Val::Px(px), Val::Px(py)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BackgroundColor(COL_ACC),
    )
}

// system: rebuild UI khi state doi hoac ui dirty
pub fn sync_ui(
    mut commands: Commands,
    game: Res<Game>,
    state: Res<State<Screen>>,
    mut res: ResMut<UiRes>,
    q: Query<Entity, With<UiRoot>>,
    assets: Res<AssetServer>,
) {
    let cur = *state.get();
    let changed = res.last != Some(cur);
    if !changed && !res.dirty {
        return;
    }
    res.dirty = false;
    res.last = Some(cur);

    let logo = assets.load("embedded://assets/logo.png");
    for e in q.iter() {
        commands.entity(e).despawn();
    }
    match cur {
        Screen::Menu => build_menu(&mut commands, &game, logo),
        Screen::Playing => build_hud(&mut commands),
        Screen::Paused => build_pause(&mut commands, &game),
        Screen::Results => build_results(&mut commands, &game),
        Screen::Board => build_board(&mut commands, &game),
        Screen::Editor => build_editor(&mut commands, &game),
        Screen::Settings => build_settings(&mut commands, &game),
    }
}

// HUD huong dan trong editor + thong bao luu
pub fn build_editor(p: &mut Commands, game: &Game) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            top: Val::Px(20.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BorderColor::all(COL_ACC),
        BackgroundColor(Color::srgba(0.03, 0.045, 0.075, 0.92)),
        UiRoot,
    ))
    .with_children(|p| {
        p.spawn(text_node(vi("MAP EDITOR", "MAP EDITOR"), 24.0, COL_ACC));
        p.spawn(text_node(&format!("{} blocks", game.mode_id), 12.0, COL_DIM));
        let lines = [
            vi("WASD bay | Space/E len, Q xuong | Shift nhanh", "WASD fly | Space/E up, Q down | Shift fast"),
            vi("Chuot trai: dat khoi | Chuot phai: xoa khoi", "LMB place | RMB delete"),
            vi("[ / ] : doi kich thuoc khoi", "[ / ] : change block size"),
            vi("T: doi mau dia hinh (theme)", "T: change terrain theme"),
            vi("F: luu map moi vao maps.json", "F: save as new map to maps.json"),
            vi("ESC: thoat editor", "ESC: exit editor"),
        ];
        for l in lines {
            p.spawn(text_node(l, 14.0, COL_DIM));
        }
    });
}

// ---- SETTINGS ----
pub fn build_settings(p: &mut Commands, game: &Game) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };
    let rows: [(usize, &'static str, String); SET_COUNT] = [
        (SET_FOV, vi("FOV", "FOV"), format!("{:.0}", game.fov)),
        (SET_SENS, vi("DO NHẠY", "SENS"), format!("{:.3}", game.sens[game.game_id])),
        (SET_DPI, "DPI", format!("{:.0}", game.dpi[game.game_id])),
        (SET_VOL, vi("AM LUONG", "VOLUME"), format!("{:.0}%", game.volume * 100.0)),
        (SET_XCOL, vi("MAU TAM", "XHAIR COLOR"), format!("{}", game.xhair_color + 1)),
        (SET_XSIZE, vi("CO TAM", "XHAIR SIZE"), format!("{:.1}", game.xhair_size)),
        (SET_INVERT, vi("DAO TRUC Y", "INVERT Y"), if game.invert_y { vi("BAT", "ON").into() } else { vi("TAT", "OFF").into() }),
        (SET_XDOT, vi("CHAM GIUA", "XHAIR DOT"), if game.xhair_dot { vi("BAT", "ON").into() } else { vi("TAT", "OFF").into() }),
        (SET_FPS, vi("GIOI HAN FPS", "FPS LIMIT"), game.fps_label()),
        (SET_QUALITY, vi("CHAT LUONG", "QUALITY"), if game.quality == 1 { "HIGH".into() } else { "LOW".into() }),
        (SET_GUN, vi("SUNG", "WEAPON"), game.gun_label().to_owned()),
        (SET_SKIN, vi("SKIN BOT", "BOT SKIN"), format!("{}", game.bot_skin + 1)),
    ];

    p.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(22.0)),
            ..default()
        },
        BackgroundColor(COL_BG),
        UiRoot,
    ))
    .with_children(|p| {
        p.spawn(text_node(vi("CAI DAT", "SETTINGS"), 34.0, COL_ACC));
        p.spawn(Node { height: Val::Px(6.0), ..default() });

        // ten nguoi choi (hien tren BXH)
        p.spawn((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BorderColor::all(COL_ACC),
            BackgroundColor(COL_PANEL),
        ))
        .with_children(|r| {
            r.spawn(text_node(vi("TEN NGUOI CHOI", "PLAYER NAME"), 14.0, COL_DIM));
            r.spawn(text_node(&game.name, 16.0, COL_ACC));
            r.spawn((panel(14.0, 5.0, false), KeyBindButton(99)))
                .with_children(|b| { b.spawn(text_node(vi("DOI TEN", "EDIT"), 13.0, COL_TXT)); });
        });

        // 2 cot cho gon
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), row_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, justify_content: JustifyContent::Center, ..default() })
            .with_children(|grid| {
                for (id, label, val) in rows {
                    grid.spawn((
                        Node {
                            width: Val::Px(300.0),
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(8.0),
                            padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            border_radius: BorderRadius::all(Val::Px(6.0)),
                            ..default()
                        },
                        BorderColor::all(COL_LINE),
                        BackgroundColor(COL_PANEL),
                    ))
                    .with_children(|r| {
                        r.spawn((Node { width: Val::Px(120.0), ..default() }, text_node(label, 14.0, COL_DIM)));
                        r.spawn((panel(14.0, 3.0, false), SetMinus(id)))
                            .with_children(|b| { b.spawn(text_node("-", 18.0, COL_TXT)); });
                        r.spawn((Node { width: Val::Px(84.0), justify_content: JustifyContent::Center, ..default() }, text_node(&val, 15.0, COL_ACC)));
                        r.spawn((panel(14.0, 3.0, false), SetPlus(id)))
                            .with_children(|b| { b.spawn(text_node("+", 18.0, COL_TXT)); });
                    });
                }
            });

        p.spawn(Node { height: Val::Px(4.0), ..default() });
        p.spawn(text_node(vi("PHIM TAT (bam de doi, roi bam phim moi)", "KEYBINDS (click, then press new key)"), 13.0, COL_DIM));
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), row_gap: Val::Px(6.0), flex_wrap: FlexWrap::Wrap, width: Val::Percent(90.0), justify_content: JustifyContent::Center, ..default() })
            .with_children(|r| {
                for i in 0..10 {
                    let nm = if lang == 0 { KEY_VI[i] } else { KEY_NAMES[i] };
                    r.spawn((panel(10.0, 5.0, false), KeyBindButton(i)))
                        .with_children(|b| {
                            b.spawn(text_node(&format!("{nm}: {}", game.key_at(i)), 13.0, COL_TXT));
                        });
                }
            });

        p.spawn(Node { height: Val::Px(4.0), ..default() });
        p.spawn((panel_acc(22.0, 9.0), BenchButton))
            .with_children(|b| { b.spawn(text_node(vi("BENCHMARK", "BENCHMARK"), 16.0, Color::srgb(0.03, 0.05, 0.08))); });
        p.spawn((panel(20.0, 10.0, false), MenuButton))
            .with_children(|b| { b.spawn(text_node(vi("VE MENU", "MAIN MENU"), 16.0, COL_DIM)); });
    });
}

pub fn build_menu(p: &mut Commands, game: &Game, logo_handle: Handle<Image>) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };

    // ---- root: hang ngang = sidebar (trai) + vung noi dung (phai) ----
    p.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            ..default()
        },
        BackgroundColor(COL_BG),
        UiRoot,
    ))
    .with_children(|root| {
        // ================= SIDEBAR DOC =================
        root.spawn((
            Node {
                width: Val::Px(96.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(16.0)),
                border: UiRect::right(Val::Px(1.0)),
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgb(0.055, 0.055, 0.082)),
        ))
        .with_children(|sb| {
            // logo badge
            sb.spawn((
                Node {
                    width: Val::Px(52.0),
                    height: Val::Px(52.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Px(14.0)),
                    overflow: Overflow::clip(),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(COL_ACC),
                BackgroundColor(Color::srgb(0.09, 0.06, 0.14)),
            ))
            .with_children(|b| {
                b.spawn((
                    Node { width: Val::Px(48.0), height: Val::Px(48.0), ..default() },
                    ImageNode::new(logo_handle),
                ));
            });
            sb.spawn(Node { height: Val::Px(6.0), ..default() });

            // nav doc: icon + nhan ngan
            let nav: [(&'static str, &'static str); 6] = [
                ("PLAY", "PLAY"),
                ("MAP", "MAP"),
                ("GUN", "GUN"),
                ("SKILL", "SKILL"),
                ("RANKS", "RANKS"),
                ("STATS", "STATS"),
            ];
            for (i, (vi_t, en_t)) in nav.iter().enumerate() {
                sb.spawn((nav_panel(72.0, 56.0, i == 0), NavigationButton(i)))
                .with_children(|b| {
                    // gach accent tren dinh
                    b.spawn((Node {
                        width: Val::Px(20.0),
                        height: Val::Px(2.0),
                        border_radius: BorderRadius::all(Val::Px(1.0)),
                        ..default()
                    }, BackgroundColor(if i == 0 { COL_ACC } else { COL_LINE })));
                    b.spawn(Node { height: Val::Px(6.0), ..default() });
                    b.spawn(text_node(vi(vi_t, en_t), 13.0, if i == 0 { COL_TXT } else { COL_DIM }));
                });
            }

            sb.spawn(Node { flex_grow: 1.0, ..default() });

            // nut thoat cuoi sidebar
            sb.spawn((panel(12.0, 10.0, false), QuitButton))
                .with_children(|b| { b.spawn(text_node(vi("THOAT", "QUIT"), 13.0, COL_DIM)); });
        });

        // ================= VUNG NOI DUNG =================
        root.spawn(Node {
            flex_grow: 1.0,
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(24.0)),
            ..default()
        })
        .with_children(|main| {
            // topbar
            main.spawn(Node {
                width: Val::Percent(100.0),
                height: Val::Px(52.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                ..default()
            })
            .with_children(|t| {
                t.spawn(text_node("WLITE", 30.0, COL_TXT));
                t.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(COL_ACC),
                    BackgroundColor(Color::srgb(0.09, 0.06, 0.14)),
                ))
                .with_children(|b| { b.spawn(text_node("AIM LAB", 12.0, COL_ACC)); });
                t.spawn(Node { flex_grow: 1.0, ..default() });
                t.spawn(text_node(vi("TRUNG TAM LUYEN AIM FPS", "FPS AIM TRAINING CENTER"), 12.0, COL_DIM));
                t.spawn((
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(COL_ACTIVE),
                ))
                .with_children(|b| {
                    let p0 = &crate::core::presets::PRESETS[game.game_id];
                    let cm = crate::core::presets::cm360(p0.yaw, game.sens[game.game_id], game.dpi[game.game_id]);
                    b.spawn(text_node(&format!("{:.1} cm/360", cm), 15.0, COL_ACC2));
                });
            });

            // ---- than: 2 cot (mode/game | start) ----
            main.spawn(Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(16.0),
                ..default()
            })
            .with_children(|cols| {
                // cot trai: card chon che do + game
                cols.spawn((
                    Node {
                        flex_grow: 1.0,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(18.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BorderColor::all(COL_LINE),
                    BackgroundColor(COL_PANEL),
                ))
                .with_children(|card| {
                    card.spawn(text_node(vi("CHE DO LUYEN", "TRAINING MODE"), 13.0, COL_ACC2));
                    let modes = ["GRIDSHOT", "FLICK", "TRACKING", "RECOIL", "BOT DUEL", "5V5 DEATHMATCH", "5V5 BOMB"];
                    card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, ..default() })
                        .with_children(|r| {
                            for (i, m) in modes.iter().enumerate() {
                                r.spawn((panel(18.0, 9.0, i == game.mode_id), ModeButton(i)))
                                    .with_children(|b| { b.spawn(text_node(m, 15.0, COL_TXT)); });
                            }
                        });

                    card.spawn(Node { height: Val::Px(4.0), ..default() });
                    card.spawn(text_node(vi("GAME", "GAME"), 13.0, COL_ACC2));
                    card.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), flex_wrap: FlexWrap::Wrap, ..default() })
                        .with_children(|r| {
                            for (i, pr) in crate::core::presets::PRESETS.iter().enumerate() {
                                r.spawn((panel(15.0, 8.0, i == game.game_id), GameButton(i)))
                                    .with_children(|b| { b.spawn(text_node(pr.name, 14.0, COL_TXT)); });
                            }
                        });

                    card.spawn(Node { height: Val::Px(4.0), ..default() });
                    card.spawn(text_node(
                        &format!(
                            "{}: {:.0}s    {}: {}    {}: {}",
                            vi("THOI GIAN", "DURATION"), game.duration,
                            vi("DO KHO", "DIFFICULTY"), game.difficulty,
                            vi("CHAT LUONG", "QUALITY"), if game.quality == 1 { "HIGH" } else { "LOW" },
                        ),
                        13.0, COL_DIM,
                    ));
                });

                // cot phai: card bat dau + tuy chon
                cols.spawn((
                    Node {
                        width: Val::Px(320.0),
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(18.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        border_radius: BorderRadius::all(Val::Px(12.0)),
                        ..default()
                    },
                    BorderColor::all(COL_LINE),
                    BackgroundColor(COL_PANEL),
                ))
                .with_children(|card| {
                    card.spawn((panel_acc(30.0, 14.0), StartButton))
                        .with_children(|b| { b.spawn(text_node(vi("BAT DAU", "START"), 20.0, Color::srgb(0.06, 0.03, 0.10))); });
                    card.spawn(Node { height: Val::Px(2.0), ..default() });
                    card.spawn((panel(18.0, 11.0, false), GunButton))
                        .with_children(|b| { b.spawn(text_node(&format!("{}: {}", vi("SUNG", "GUN"), game.gun_label()), 15.0, COL_ACC2)); });
                    card.spawn((panel(18.0, 11.0, false), MapButton))
                        .with_children(|b| { b.spawn(text_node(&format!("{}: {}", vi("MAP", "MAP"), game.map_label()), 15.0, COL_ACC2)); });
                    card.spawn(Node { flex_grow: 1.0, ..default() });
                    card.spawn((panel(16.0, 10.0, false), SettingsButton))
                        .with_children(|b| { b.spawn(text_node(vi("CAI DAT", "SETTINGS"), 15.0, COL_ACC2)); });
                    card.spawn((panel(16.0, 10.0, false), EditorButton))
                        .with_children(|b| { b.spawn(text_node(vi("EDITOR", "EDITOR"), 15.0, COL_DIM)); });
                    card.spawn((panel(16.0, 10.0, false), BoardButton))
                        .with_children(|b| { b.spawn(text_node(vi("BXH", "RANKS"), 15.0, COL_DIM)); });
                    card.spawn((panel(16.0, 10.0, false), QualityButton))
                        .with_children(|b| { b.spawn(text_node(vi("CHAT LUONG", "QUALITY"), 14.0, COL_DIM)); });
                    card.spawn((panel(16.0, 10.0, false), LangButton))
                        .with_children(|b| { b.spawn(text_node(vi("EN", "VI"), 15.0, COL_DIM)); });
                });
            });
        });
    });

    // about: goc duoi phai -> mo link
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(20.0),
            bottom: Val::Px(16.0),
            ..default()
        },
        UiRoot,
    ))
    .with_children(|a| {
        a.spawn((panel(14.0, 7.0, false), AboutButton))
            .with_children(|b| { b.spawn(text_node(vi("ABOUT", "ABOUT"), 13.0, COL_DIM)); });
    });
}

pub fn build_hud(p: &mut Commands) {
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        UiRoot,
        HudRoot,
    ))
    .with_children(|p| {
        p.spawn(Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Px(12.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            margin: UiRect::left(Val::Px(-200.0)),
            ..default()
        })
        .with_children(|r| {
            r.spawn((Node { padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BorderColor::all(COL_LINE), BackgroundColor(Color::srgba(0.03, 0.045, 0.07, 0.82))))
                .with_children(|b| { b.spawn((text_node("MODE", 15.0, COL_DIM), HudMode)); });
            r.spawn((Node { padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BorderColor::all(COL_ACC), BackgroundColor(Color::srgba(0.03, 0.045, 0.07, 0.86))))
                .with_children(|b| { b.spawn((text_node("0:00", 21.0, COL_TXT), HudText)); });
            r.spawn((Node { padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)), border: UiRect::all(Val::Px(1.0)), border_radius: BorderRadius::all(Val::Px(6.0)), ..default() }, BorderColor::all(COL_LINE), BackgroundColor(Color::srgba(0.03, 0.045, 0.07, 0.82))))
                .with_children(|b| { b.spawn((text_node("0", 21.0, COL_ACC), HudScore)); });
        });

        p.spawn((
            Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), bottom: Val::Px(30.0), margin: UiRect::left(Val::Px(-40.0)), ..default() },
            HudHp,
            Visibility::Hidden,
        ))
        .with_children(|b| { b.spawn(text_node("100", 15.0, COL_TXT)); });

        p.spawn((
            Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Percent(45.0), margin: UiRect::left(Val::Px(-140.0)), ..default() },
            DeathMsg,
            Visibility::Hidden,
        ))
        .with_children(|b| { b.spawn(text_node("YOU DIED", 58.0, Color::srgb(0.90, 0.34, 0.34))); });

        p.spawn((
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::srgba(0.75, 0.08, 0.10, 0.0)),
            HurtVignette,
        ));

        // scope overlay (sniper ads): vien den 4 canh + chu thap manh
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            ScopeOverlay,
            Visibility::Hidden,
        ))
        .with_children(|s| {
            // 4 tam den che ria, chua khoang giua hinh vuong
            let mask = Color::srgba(0.0, 0.0, 0.0, 0.92);
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(38.0), ..default() }, BackgroundColor(mask)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), bottom: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(38.0), ..default() }, BackgroundColor(mask)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Px(0.0), top: Val::Percent(38.0), width: Val::Percent(38.0), height: Val::Percent(24.0), ..default() }, BackgroundColor(mask)));
            s.spawn((Node { position_type: PositionType::Absolute, right: Val::Px(0.0), top: Val::Percent(38.0), width: Val::Percent(38.0), height: Val::Percent(24.0), ..default() }, BackgroundColor(mask)));
            // vien xanh quanh vung ngam
            let edge = Color::srgba(0.02, 0.03, 0.04, 0.95);
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(38.0), top: Val::Percent(38.0), width: Val::Percent(24.0), height: Val::Px(2.0), ..default() }, BackgroundColor(edge)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(38.0), bottom: Val::Percent(38.0), width: Val::Percent(24.0), height: Val::Px(2.0), ..default() }, BackgroundColor(edge)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(38.0), top: Val::Percent(38.0), width: Val::Px(2.0), height: Val::Percent(24.0), ..default() }, BackgroundColor(edge)));
            s.spawn((Node { position_type: PositionType::Absolute, right: Val::Percent(38.0), top: Val::Percent(38.0), width: Val::Px(2.0), height: Val::Percent(24.0), ..default() }, BackgroundColor(edge)));
            // chu thap giua
            let line = Color::srgba(0.85, 0.92, 1.0, 0.85);
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Percent(38.0), margin: UiRect::left(Val::Px(-1.0)), width: Val::Px(2.0), height: Val::Percent(12.0), ..default() }, BackgroundColor(line)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), bottom: Val::Percent(38.0), margin: UiRect::left(Val::Px(-1.0)), width: Val::Px(2.0), height: Val::Percent(12.0), ..default() }, BackgroundColor(line)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(38.0), top: Val::Percent(50.0), margin: UiRect::top(Val::Px(-1.0)), width: Val::Percent(12.0), height: Val::Px(2.0), ..default() }, BackgroundColor(line)));
            s.spawn((Node { position_type: PositionType::Absolute, right: Val::Percent(38.0), top: Val::Percent(50.0), margin: UiRect::top(Val::Px(-1.0)), width: Val::Percent(12.0), height: Val::Px(2.0), ..default() }, BackgroundColor(line)));
            s.spawn((Node { position_type: PositionType::Absolute, left: Val::Percent(50.0), top: Val::Percent(50.0), margin: UiRect::new(Val::Px(-2.0), Val::Px(0.0), Val::Px(-2.0), Val::Px(0.0)), width: Val::Px(4.0), height: Val::Px(4.0), ..default() }, BackgroundColor(line)));
        });

        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect::new(Val::Px(-12.0), Val::Px(0.0), Val::Px(-12.0), Val::Px(0.0)),
                width: Val::Px(24.0),
                height: Val::Px(24.0),
                ..default()
            },
            Crosshair,
        ))
        .with_children(|c| {
            for (w, h, mx, my) in [
                (2.0f32, 8.0f32, 11.0f32, 0.0f32),
                (2.0, 8.0, 11.0, 16.0),
                (8.0, 2.0, 0.0, 11.0),
                (8.0, 2.0, 16.0, 11.0),
            ] {
                c.spawn((
                    Node { position_type: PositionType::Absolute, left: Val::Px(mx), top: Val::Px(my), width: Val::Px(w), height: Val::Px(h), ..default() },
                    BackgroundColor(COL_ACC),
                    CrosshairPart,
                ));
            }
            c.spawn((
                Node { position_type: PositionType::Absolute, left: Val::Px(11.0), top: Val::Px(11.0), width: Val::Px(2.0), height: Val::Px(2.0), ..default() },
                BackgroundColor(COL_ACC),
                CrosshairPart,
            ));
        });

        // HUD rieng cho mode 5v5 (mac dinh an)
        crate::ui_tf::build_tf(p);
    });
}

pub fn build_pause(p: &mut Commands, game: &Game) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(18.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.85)),
        UiRoot,
    ))
    .with_children(|p| {
        p.spawn(text_node(vi("TAM DUNG", "PAUSED"), 46.0, COL_TXT));
        p.spawn((panel_acc(26.0, 11.0), ResumeButton))
            .with_children(|b| { b.spawn(text_node(vi("TIEP TUC", "RESUME"), 18.0, Color::srgb(0.03, 0.05, 0.08))); });
        p.spawn((panel(20.0, 10.0, false), MenuButton))
            .with_children(|b| { b.spawn(text_node(vi("VE MENU", "MAIN MENU"), 16.0, COL_DIM)); });
    });
}

pub fn build_results(p: &mut Commands, game: &Game) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };
    let r = game.result.clone();
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(14.0),
            ..default()
        },
        BackgroundColor(COL_BG),
        UiRoot,
    ))
    .with_children(|p| {
        p.spawn(text_node(
            if game.best_flash { vi("KY LUC MOI!", "NEW RECORD!") } else { vi("KET THUC", "RESULTS") },
            42.0,
            if game.best_flash { COL_ACC } else { COL_TXT },
        ));
        if let Some(r) = &r {
            p.spawn(text_node(&format!("{}: {}", vi("DIEM", "SCORE"), r.score), 30.0, COL_TXT));
            p.spawn(text_node(&format!("{}: {}%", vi("CHINH XAC", "ACCURACY"), r.acc), 18.0, COL_DIM));
            if r.mode_id == 4 {
                p.spawn(text_node(&format!("{}: {}", vi("MANG", "DEATHS"), r.deaths), 18.0, COL_DIM));
            }
        }
        p.spawn((panel_acc(26.0, 11.0), RetryButton))
            .with_children(|b| { b.spawn(text_node(vi("CHOI LAI", "RETRY"), 18.0, Color::srgb(0.03, 0.05, 0.08))); });
        p.spawn((panel(20.0, 10.0, false), MenuButton))
            .with_children(|b| { b.spawn(text_node(vi("VE MENU", "MAIN MENU"), 16.0, COL_DIM)); });
    });
}

// ==== bang xep hang (lich su diem cao) ====
pub fn build_board(p: &mut Commands, game: &Game) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };
    let board_all = game.stats.board.clone();
    let total = board_all.len();
    let filt = game.board_filter;
    let board: Vec<_> = if filt < 5 {
        board_all.iter().filter(|e| e.mode_id == filt).cloned().collect()
    } else {
        board_all.clone()
    };

    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(24.0)),
            row_gap: Val::Px(10.0),
            ..default()
        },
        BackgroundColor(COL_BG),
        UiRoot,
    ))
    .with_children(|p| {
        p.spawn(text_node(vi("BANG XEP HANG", "LEADERBOARD"), 38.0, COL_ACC));
        p.spawn(text_node(
            &format!("{} {}", total, vi("ket qua duoc luu", "results saved")),
            13.0,
            COL_DIM,
        ));

        // filter theo che do
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(6.0), ..default() })
            .with_children(|f| {
                let labels = [vi("TAT CA", "ALL"), "GRIDSHOT", "FLICK", "TRACKING", "RECOIL", "DUEL"];
                for (i, lb) in labels.iter().enumerate() {
                    let on = if i == 0 { filt >= 5 } else { filt == i - 1 };
                    f.spawn((panel(14.0, 5.0, on), BoardFilterButton(i)))
                        .with_children(|b| { b.spawn(text_node(lb, 13.0, if on { COL_TXT } else { COL_DIM })); });
                }
            });

        // header
        p.spawn(Node {
            width: Val::Percent(90.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
            ..default()
        })
        .with_children(|h| {
            let cols = [
                (vi("#", "#"), Val::Px(40.0)),
                (vi("TEN", "PLAYER"), Val::Px(110.0)),
                (vi("CHE DO", "MODE"), Val::Percent(15.0)),
                (vi("GAME", "GAME"), Val::Percent(13.0)),
                (vi("DIEM", "SCORE"), Val::Percent(12.0)),
                (vi("CHINH XAC", "ACC"), Val::Px(78.0)),
                (vi("DO KHO", "DIFF"), Val::Px(58.0)),
                (vi("NGAY", "DATE"), Val::Px(120.0)),
            ];
            for (name, w) in cols {
                h.spawn((
                    Node { width: w, ..default() },
                    text_node(name, 13.0, COL_ACC),
                ));
            }
        });

        // rows (top 20)
        if board.is_empty() {
            p.spawn(text_node(vi("chua co ket qua nao", "no results yet"), 16.0, COL_DIM));
        } else {
            for (i, e) in board.iter().take(20).enumerate() {
                let rank = i + 1;
                let rank_col = match rank {
                    1 => Color::srgb(1.0, 0.84, 0.30),
                    2 => Color::srgb(0.80, 0.83, 0.88),
                    3 => Color::srgb(0.85, 0.55, 0.30),
                    _ => COL_DIM,
                };
                let bg = if i % 2 == 0 { COL_PANEL } else { Color::srgba(0.05, 0.075, 0.12, 0.55) };
                p.spawn((
                    Node {
                        width: Val::Percent(90.0),
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                ))
                .with_children(|row| {
                    row.spawn((Node { width: Val::Px(40.0), ..default() }, text_node(&format!("{rank}"), 15.0, rank_col)));
                    let nm = if e.name.is_empty() { "-".to_owned() } else { e.name.clone() };
                    row.spawn((Node { width: Val::Px(110.0), ..default() }, text_node(&nm, 14.0, if rank <= 3 { rank_col } else { COL_TXT })));
                    row.spawn((Node { width: Val::Percent(15.0), ..default() }, text_node(&e.mode, 14.0, COL_TXT)));
                    row.spawn((Node { width: Val::Percent(13.0), ..default() }, text_node(&e.game, 14.0, COL_DIM)));
                    row.spawn((Node { width: Val::Percent(12.0), ..default() }, text_node(&e.score, 14.0, COL_ACC)));
                    row.spawn((Node { width: Val::Px(78.0), ..default() }, text_node(&format!("{}%", e.acc), 14.0, COL_DIM)));
                    row.spawn((Node { width: Val::Px(58.0), ..default() }, text_node(&format!("{}", e.difficulty + 1), 14.0, COL_DIM)));
                    row.spawn((Node { width: Val::Px(120.0), ..default() }, text_node(&fmt_date(e.ts), 13.0, COL_DIM)));
                });
            }
        }

        p.spawn((panel(20.0, 10.0, false), MenuButton))
            .with_children(|b| { b.spawn(text_node(vi("VE MENU", "MAIN MENU"), 16.0, COL_DIM)); });
    });
}

// timestamp -> "YYYY-MM-DD HH:MM" (khong can chinh xac mui gio)
fn fmt_date(ts: u64) -> String {
    if ts == 0 { return "-".into(); }
    let days = ts / 86400;
    let secs = ts % 86400;
    let (h, m) = (secs / 3600, (secs % 3600) / 60);
    // doi ngay tu 1970-01-01
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{d:02} {h:02}:{m:02}")
}

// thuat toan chuyen so ngay tu epoch -> (nam, thang, ngay), ko dung lib
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ==== per-frame updates ====
pub fn update_hud(
    game: Res<Game>,
    mut qt: Query<&mut Text, (With<HudText>, Without<HudScore>, Without<HudMode>, Without<HudHp>, Without<DeathMsg>)>,
    mut qs: Query<&mut Text, (With<HudScore>, Without<HudText>, Without<HudMode>, Without<HudHp>, Without<DeathMsg>)>,
    mut qm: Query<&mut Text, (With<HudMode>, Without<HudText>, Without<HudScore>, Without<HudHp>, Without<DeathMsg>)>,
    mut qhp: Query<(&mut Text, &mut Visibility), (With<HudHp>, Without<HudText>, Without<HudScore>, Without<HudMode>, Without<DeathMsg>)>,
    mut qdeath: Query<&mut Visibility, (With<DeathMsg>, Without<HudText>, Without<HudScore>, Without<HudMode>, Without<HudHp>)>,
) {
    let Some(d) = game.drill.as_ref() else { return };
    let (left, _) = d.timer();
    let secs = left.max(0.0).ceil() as u32;
    if let Ok(mut t) = qt.single_mut() {
        t.0 = format!("{}:{:02}", secs / 60, secs % 60);
    }
    if let Ok(mut t) = qs.single_mut() {
        t.0 = d.score();
    }
    if let Ok(mut t) = qm.single_mut() {
        t.0 = d.mode_name().to_owned();
    }
    let hp = d.hp();
    if let Ok((mut t, mut vis)) = qhp.single_mut() {
        if hp < 99.5 {
            t.0 = format!("{:.0}", hp.clamp(0.0, 100.0));
            *vis = Visibility::Inherited;
        } else {
            *vis = Visibility::Hidden;
        }
    }
    if let Ok(mut v) = qdeath.single_mut() {
        *v = if d.show_msg() { Visibility::Inherited } else { Visibility::Hidden };
    }
}

pub fn update_vignette(
    world: Res<crate::core::world::World>,
    mut q: Query<&mut BackgroundColor, With<HurtVignette>>,
) {
    let hurt = world.hurt.clamp(0.0, 1.0);
    if let Ok(mut bg) = q.single_mut() {
        bg.0 = Color::srgba(0.75, 0.08, 0.10, hurt * 0.30);
    }
}

pub fn update_crosshair(
    game: Res<Game>,
    world: Res<crate::core::world::World>,
    mut q: Query<&mut BackgroundColor, With<CrosshairPart>>,
    mut q_ch: Query<&mut Visibility, (With<Crosshair>, Without<ScopeOverlay>)>,
    mut q_scope: Query<&mut Visibility, (With<ScopeOverlay>, Without<Crosshair>)>,
) {
    let mut c = crosshair_color(game.xhair_color);
    c.set_alpha(0.9);
    let scoped = world.ads > 0.5;
    for mut bg in q.iter_mut() {
        bg.0 = c;
    }
    if let Ok(mut v) = q_ch.single_mut() {
        *v = if scoped { Visibility::Hidden } else { Visibility::Inherited };
    }
    if let Ok(mut v) = q_scope.single_mut() {
        *v = if scoped { Visibility::Inherited } else { Visibility::Hidden };
    }
}
