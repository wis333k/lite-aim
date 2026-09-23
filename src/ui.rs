// ui: menu / HUD / pause / results — Bevy UI (thay immediate-mode macroquad)
use crate::game::{Game, Screen};
use crate::render::crosshair_color;
use bevy::prelude::*;

pub const COL_ACC: Color = Color::srgb(0.20, 0.85, 1.0);
pub const COL_ACC2: Color = Color::srgb(0.55, 0.45, 1.0);
pub const COL_PANEL: Color = Color::srgba(0.055, 0.075, 0.115, 0.98);
pub const COL_PANEL_HOVER: Color = Color::srgba(0.09, 0.13, 0.20, 0.98);
pub const COL_ACTIVE: Color = Color::srgba(0.10, 0.32, 0.46, 1.0);
pub const COL_TXT: Color = Color::srgb(0.94, 0.96, 1.0);
pub const COL_DIM: Color = Color::srgb(0.48, 0.58, 0.70);
pub const COL_BG: Color = Color::srgb(0.025, 0.038, 0.062);
pub const COL_LINE: Color = Color::srgb(0.10, 0.16, 0.24);

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

#[derive(Resource, Default)]
pub struct UiRes {
    pub dirty: bool,
    pub last: Option<Screen>,
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
) {
    let cur = *state.get();
    let changed = res.last != Some(cur);
    if !changed && !res.dirty {
        return;
    }
    res.dirty = false;
    res.last = Some(cur);

    for e in q.iter() {
        commands.entity(e).despawn();
    }
    match cur {
        Screen::Menu => build_menu(&mut commands, &game),
        Screen::Playing => build_hud(&mut commands),
        Screen::Paused => build_pause(&mut commands, &game),
        Screen::Results => build_results(&mut commands, &game),
        Screen::Board => build_board(&mut commands, &game),
        Screen::Editor => build_editor(&mut commands, &game),
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
            vi("F: luu map moi vao maps.json", "F: save as new map to maps.json"),
            vi("ESC: thoat editor", "ESC: exit editor"),
        ];
        for l in lines {
            p.spawn(text_node(l, 14.0, COL_DIM));
        }
    });
}

pub fn build_menu(p: &mut Commands, game: &Game) {
    let lang = game.lang;
    let vi = |v: &'static str, e: &'static str| -> &'static str { if lang == 0 { v } else { e } };

    p.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(26.0)),
            ..default()
        },
        BackgroundColor(COL_BG),
        UiRoot,
    ))
    .with_children(|p| {
        // ---- header ----
        p.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(66.0),
                align_items: AlignItems::Center,
                column_gap: Val::Px(14.0),
                padding: UiRect::horizontal(Val::Px(16.0)),
                border: UiRect::bottom(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgb(0.045, 0.06, 0.09)),
        ))
        .with_children(|h| {
            // logo badge
            h.spawn((
                Node {
                    width: Val::Px(40.0),
                    height: Val::Px(40.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border_radius: BorderRadius::all(Val::Px(9.0)),
                    ..default()
                },
                BackgroundColor(COL_ACC),
            ))
            .with_children(|b| { b.spawn(text_node("L", 24.0, Color::srgb(0.03, 0.05, 0.08))); });
            h.spawn(text_node("LITE-AIM", 30.0, COL_TXT));
            h.spawn(Node { flex_grow: 1.0, ..default() });
            h.spawn(text_node(vi("TRUNG TAM LUYEN AIM FPS", "FPS AIM TRAINING CENTER"), 12.0, COL_DIM));
            h.spawn((
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
                b.spawn(text_node(&format!("{:.1} cm/360", cm), 15.0, COL_ACC));
            });
        });

        // ---- mode ----
        p.spawn(text_node(vi("CHE DO", "MODE"), 13.0, COL_DIM));
        let modes = ["GRIDSHOT", "FLICK", "TRACKING", "RECOIL", "BOT DUEL"];
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
            .with_children(|r| {
                for (i, m) in modes.iter().enumerate() {
                    r.spawn((panel(18.0, 9.0, i == game.mode_id), ModeButton(i)))
                        .with_children(|b| { b.spawn(text_node(m, 15.0, COL_TXT)); });
                }
            });

        // ---- game ----
        p.spawn(text_node(vi("GAME", "GAME"), 13.0, COL_DIM));
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(10.0), ..default() })
            .with_children(|r| {
                for (i, pr) in crate::core::presets::PRESETS.iter().enumerate() {
                    r.spawn((panel(15.0, 8.0, i == game.game_id), GameButton(i)))
                        .with_children(|b| { b.spawn(text_node(pr.name, 14.0, COL_TXT)); });
                }
            });

        p.spawn(text_node(
            &format!(
                "{}: {:.0}s   |   {}: {}   |   {}: {}",
                vi("THOI GIAN", "DURATION"),
                game.duration,
                vi("DO KHO", "DIFFICULTY"),
                game.difficulty,
                vi("CHAT LUONG", "QUALITY"),
                if game.quality == 1 { "HIGH" } else { "LOW" },
            ),
            13.0,
            COL_DIM,
        ));

        // ---- actions ----
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            row_gap: Val::Px(8.0),
            flex_wrap: FlexWrap::Wrap,
            margin: UiRect::top(Val::Px(8.0)),
            ..default()
        })
            .with_children(|r| {
                r.spawn((panel_acc(30.0, 13.0), StartButton))
                    .with_children(|b| { b.spawn(text_node(vi("BAT DAU", "START"), 19.0, Color::srgb(0.03, 0.05, 0.08))); });
                r.spawn((panel(18.0, 11.0, false), GunButton))
                    .with_children(|b| { b.spawn(text_node(&format!("{}: {}", vi("SUNG", "GUN"), game.gun_label()), 15.0, COL_ACC)); });
                r.spawn((panel(18.0, 11.0, false), MapButton))
                    .with_children(|b| { b.spawn(text_node(&format!("{}: {}", vi("MAP", "MAP"), game.map_label()), 15.0, COL_ACC)); });
                r.spawn((panel(18.0, 11.0, false), EditorButton))
                    .with_children(|b| { b.spawn(text_node(vi("EDITOR", "EDITOR"), 15.0, COL_DIM)); });
                r.spawn((panel(18.0, 11.0, false), BoardButton))
                    .with_children(|b| { b.spawn(text_node(vi("BXH", "RANKS"), 15.0, COL_DIM)); });
                r.spawn((panel(18.0, 11.0, false), QualityButton))
                    .with_children(|b| { b.spawn(text_node(vi("CHAT LUONG", "QUALITY"), 13.0, COL_DIM)); });
                r.spawn((panel(18.0, 11.0, false), LangButton))
                    .with_children(|b| { b.spawn(text_node(vi("EN", "VI"), 15.0, COL_DIM)); });
                r.spawn((panel(18.0, 11.0, false), QuitButton))
                    .with_children(|b| { b.spawn(text_node(vi("THOAT", "QUIT"), 15.0, COL_DIM)); });
            });
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
    let board = game.stats.board.clone();
    let total = board.len();

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
                (vi("#", "#"), Val::Px(46.0)),
                (vi("CHE DO", "MODE"), Val::Percent(18.0)),
                (vi("GAME", "GAME"), Val::Percent(16.0)),
                (vi("DIEM", "SCORE"), Val::Percent(14.0)),
                (vi("CHINH XAC", "ACC"), Val::Px(90.0)),
                (vi("DO KHO", "DIFF"), Val::Px(70.0)),
                (vi("NGAY", "DATE"), Val::Px(130.0)),
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
                    row.spawn((Node { width: Val::Px(46.0), ..default() }, text_node(&format!("{rank}"), 15.0, rank_col)));
                    row.spawn((Node { width: Val::Percent(18.0), ..default() }, text_node(&e.mode, 14.0, COL_TXT)));
                    row.spawn((Node { width: Val::Percent(16.0), ..default() }, text_node(&e.game, 14.0, COL_DIM)));
                    row.spawn((Node { width: Val::Percent(14.0), ..default() }, text_node(&e.score, 14.0, COL_ACC)));
                    row.spawn((Node { width: Val::Px(90.0), ..default() }, text_node(&format!("{}%", e.acc), 14.0, COL_DIM)));
                    row.spawn((Node { width: Val::Px(70.0), ..default() }, text_node(&format!("{}", e.difficulty + 1), 14.0, COL_DIM)));
                    row.spawn((Node { width: Val::Px(130.0), ..default() }, text_node(&fmt_date(e.ts), 13.0, COL_DIM)));
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
