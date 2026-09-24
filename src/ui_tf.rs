// HUD rieng cho mode 5v5: kill feed, ty le doi, thanh HP + giap, radar,
// popup headshot / nhieu kill, bang KDA (Tab), banner vong.
//
// Cac node duoc spawn 1 lan boi `build_tf` (goi tu ui::build_hud) va cap
// nhat moi frame boi `update_tf` + `update_tf_radar_board`.
//
// QUAN TRONG — ly do tach nhu nay:
//   Bevy khong chung minh duoc 2 query tren cung mot component la "rieng
//   biet" (B0001). Vi vay moi node chi dung MOT trong hai marker:
//     * `TfLabel` — moi node Text (chi 1 query &mut Text)
//     * `TfVis`  — moi node co Visibility hoac Node (chi 1 query &mut
//                  Visibility + 1 query &mut Node)
//   them marker moi phai gan vao day, KHONG tao component rieng.

use bevy::prelude::*;

use crate::core::teamfight::hud::TfHud;
use crate::core::teamfight::round::ROUNDS_TO_WIN;
use crate::core::world::World;
use crate::ui::{COL_ACC, COL_ACC2, COL_DIM, COL_LINE, COL_TXT};

pub const COL_CT: Color = Color::srgb(0.30, 0.55, 1.00);
pub const COL_T: Color = Color::srgb(1.00, 0.55, 0.20);
pub const RADAR_SIZE: f32 = 150.0;
pub const RADAR_RANGE: f32 = 34.0;

/// Moi node Text cua HUD 5v5 deu mang marker nay.
#[derive(Component)]
pub enum TfLabel {
    /// diem cua doi 0 (CT) hoac 1 (T)
    Score(u8),
    /// so vong hien tai
    Round,
    /// dong ho dem nguoc
    Timer,
    /// K / D cua nguoi choi
    Kd,
    /// noi dung 1 dong kill feed
    Feed(u32),
    /// o trong bang KDA: (dong, cot)
    Board(u32, u8),
    /// noi dung popup headshot / streak
    Popup,
    /// dong bom: "BOM" / "GIU E DE GIAO" / fuse dem nguoc
    Bomb,
}

/// Moi node co `Visibility` hoac can doi `Node` deu mang marker nay.
#[derive(Component)]
pub enum TfVis {
    /// goc chua toan bo HUD
    Root,
    /// banner vong
    Banner,
    /// popup headshot / streak
    Popup,
    /// 1 dong kill feed
    Feed(u32),
    /// thanh mau (Node.width = % HP)
    HpFill,
    /// thanh giap (Node.width = % armor)
    ArmorFill,
    /// 1 cham tren radar (Node.left/top = vi tri)
    Radar(usize),
    /// khung bang KDA
    Board,
    /// 1 dong trong bang KDA
    BoardRow(u32),
    /// thanh tien do gieo / go bom
    ActionBar,
    /// phan mau cua thanh tien do
    ActionBarFill,
    /// dong chu bom (co bom / bom da giao)
    BombText,
}

fn txt(s: &str, size: f32, col: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont { font_size: size, ..default() },
        TextColor(col),
    )
}

/// Spawn toan bo HUD 5v5. An mac dinh.
pub fn build_tf(p: &mut bevy::ecs::hierarchy::ChildSpawnerCommands) {
    p.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        TfVis::Root,
        Visibility::Hidden,
    ))
    .with_children(|r| {
        // --- hang tren: diem CT - ROUND - diem T ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(70.0),
                margin: UiRect::left(Val::Px(-150.0)),
                width: Val::Px(300.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(5.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(5.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgba(0.03, 0.045, 0.07, 0.80)),
        ))
        .with_children(|t| {
            t.spawn((
                Node { width: Val::Px(60.0), justify_content: JustifyContent::Center, ..default() },
                txt("0", 24.0, COL_CT),
                TfLabel::Score(0),
            ));
            t.spawn((
                Node { justify_content: JustifyContent::Center, ..default() },
                txt("R1", 15.0, COL_DIM),
                TfLabel::Round,
            ));
            t.spawn((
                Node { width: Val::Px(60.0), justify_content: JustifyContent::Center, ..default() },
                txt("0", 24.0, COL_T),
                TfLabel::Score(1),
            ));
        });

        // --- dong ho ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(108.0),
                margin: UiRect::left(Val::Px(-40.0)),
                width: Val::Px(80.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            txt("0:00", 17.0, COL_TXT),
            TfLabel::Timer,
        ));

        // --- K/D nguoi choi ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(130.0),
                margin: UiRect::left(Val::Px(-70.0)),
                width: Val::Px(140.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            txt("K 0   D 0   KD 0.00", 13.0, COL_ACC2),
            TfLabel::Kd,
        ));

        // --- banner vong ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(32.0),
                margin: UiRect::left(Val::Px(-200.0)),
                width: Val::Px(400.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            txt("ROUND 1", 44.0, Color::srgb(1.0, 0.86, 0.30)),
            TfVis::Banner,
            Visibility::Hidden,
        ));

        // --- thanh HP + giap ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                bottom: Val::Px(48.0),
                margin: UiRect::left(Val::Px(-130.0)),
                width: Val::Px(260.0),
                height: Val::Px(16.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(3.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
        ))
        .with_children(|hp| {
            hp.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.20, 0.75, 0.35, 0.95)),
                TfVis::HpFill,
            ));
            hp.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.35, 0.65, 1.0, 0.95)),
                TfVis::ArmorFill,
            ));
        });

        // --- kill feed (gom tren phai) ---
        for i in 0..5u32 {
            r.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(16.0),
                    top: Val::Px(64.0 + 24.0 * i as f32),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(3.0)),
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::all(Val::Px(4.0)),
                    ..default()
                },
                BorderColor::all(COL_LINE),
                BackgroundColor(Color::srgba(0.03, 0.045, 0.07, 0.78)),
                txt("", 12.0, COL_TXT),
                TfLabel::Feed(i),
                TfVis::Feed(i),
                Visibility::Hidden,
            ));
        }

        // --- popup headshot / streak ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(58.0),
                margin: UiRect::left(Val::Px(-120.0)),
                width: Val::Px(240.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            txt("", 20.0, Color::srgb(1.0, 0.85, 0.25)),
            TfLabel::Popup,
            TfVis::Popup,
            Visibility::Hidden,
        ));

        // --- radar ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(16.0),
                bottom: Val::Px(16.0),
                width: Val::Px(RADAR_SIZE),
                height: Val::Px(RADAR_SIZE),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgba(0.03, 0.05, 0.07, 0.62)),
        ))
        .with_children(|rd| {
            for s in 0..10usize {
                rd.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(50.0),
                        width: Val::Px(6.0),
                        height: Val::Px(6.0),
                        margin: UiRect::left(Val::Px(-3.0)),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
                    TfVis::Radar(s),
                    Visibility::Hidden,
                ));
            }
        });

        // --- dong bom (chi hien o mode BOMB) ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Px(150.0),
                margin: UiRect::left(Val::Px(-90.0)),
                width: Val::Px(180.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            txt("", 14.0, Color::srgb(1.0, 0.45, 0.15)),
            TfLabel::Bomb,
            TfVis::BombText,
            Visibility::Hidden,
        ));

        // --- thanh tien do gieo / go bom ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(62.0),
                margin: UiRect::left(Val::Px(-90.0)),
                width: Val::Px(180.0),
                height: Val::Px(10.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgba(0.05, 0.05, 0.07, 0.85)),
            TfVis::ActionBar,
            Visibility::Hidden,
        ))
        .with_children(|bar| {
            bar.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(0.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.55, 0.15, 0.95)),
                TfVis::ActionBarFill,
            ));
        });

        // --- bang KDA (Tab) ---
        r.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                margin: UiRect::new(Val::Px(-300.0), Val::Px(-190.0), Val::Px(0.0), Val::Px(0.0)),
                width: Val::Px(600.0),
                padding: UiRect::all(Val::Px(16.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(3.0),
                ..default()
            },
            BorderColor::all(COL_LINE),
            BackgroundColor(Color::srgba(0.03, 0.04, 0.06, 0.94)),
            TfVis::Board,
            Visibility::Hidden,
        ))
        .with_children(|b| {
            b.spawn(txt("5V5 DEATHMATCH - BAN KDA", 20.0, COL_ACC));
            b.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    padding: UiRect::vertical(Val::Px(4.0)),
                    border: UiRect::bottom(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(COL_LINE),
            ))
            .with_children(|h| {
                for (label, w) in
                    [("NGUOI", 130.0f32), ("K", 50.0), ("D", 50.0), ("HS", 50.0), ("DMG", 80.0)]
                {
                    h.spawn((
                        Node { width: Val::Px(w), justify_content: JustifyContent::Center, ..default() },
                        txt(label, 13.0, COL_DIM),
                    ));
                }
            });
            for i in 0..10u32 {
                b.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(10.0),
                        padding: UiRect::vertical(Val::Px(2.0)),
                        ..default()
                    },
                    TfVis::BoardRow(i),
                    Visibility::Hidden,
                ))
                .with_children(|r| {
                    for (ci, w) in [130.0f32, 50.0, 50.0, 50.0, 80.0].into_iter().enumerate() {
                        r.spawn((
                            Node {
                                width: Val::Px(w),
                                justify_content: JustifyContent::Center,
                                ..default()
                            },
                            txt("", 13.0, COL_TXT),
                            TfLabel::Board(i, ci as u8),
                        ));
                    }
                });
            }
        });
    });
}

/// Thoi gian hien popup (giay)
pub const POPUP_TIME: f32 = 1.6;

#[derive(Resource, Default)]
pub struct TfPopupState {
    pub text: String,
    pub t: f32,
}

/// Cap nhat toan bo HUD 5v5. Doc snapshot `World.tf`.
/// Gop tat ca vao MOT system: nhieu system cung truy cap `Visibility` se
/// bi Bevy ba loi B0001 neu khong ep thu tu.
pub fn update_tf(
    world: Res<World>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut popup: ResMut<TfPopupState>,
    mut vis: Query<(&TfVis, &mut Visibility)>,
    mut nodes: Query<(&TfVis, &mut Node)>,
    mut labels: Query<(&TfLabel, &mut Text)>,
) {
    let tf: &TfHud = &world.tf;
    let on = tf.active;
    let show_board = on && keys.pressed(KeyCode::Tab);

    // --- 1. an/bat theo trang thai ---
    for (v, mut vis_v) in vis.iter_mut() {
        *vis_v = match v {
            TfVis::Root if on => Visibility::Inherited,
            TfVis::Banner if on && tf.freeze_t > 0.0 && !tf.banner.is_empty() => {
                Visibility::Inherited
            }
            // dong bom chi hien khi co gi de bao
            TfVis::BombText if on && tf.bomb_mode && (tf.bomb_planted || tf.has_bomb) => {
                Visibility::Inherited
            }
            TfVis::Popup if on && popup.t > 0.0 => Visibility::Inherited,
            TfVis::ActionBar if on && tf.bomb_action => Visibility::Inherited,
            TfVis::Feed(i) if on => match tf.feed.get(*i as usize) {
                Some(f) if f.life > 0.0 => Visibility::Inherited,
                _ => Visibility::Hidden,
            },
            TfVis::Board | TfVis::BoardRow(_) if show_board => Visibility::Inherited,
            _ => Visibility::Hidden,
        };
    }
    if !on {
        return;
    }

    // --- 2. text ---
    let mm = (tf.time_left / 60.0) as u32;
    let ss = (tf.time_left.max(0.0) % 60.0) as u32;
    for (label, mut t) in labels.iter_mut() {
        match label {
            TfLabel::Score(team) => {
                let n = tf.score[(*team as usize).min(1)];
                t.0 = if n as f32 >= ROUNDS_TO_WIN as f32 {
                    format!("{n}*")
                } else {
                    format!("{n}")
                };
            }
            TfLabel::Round => t.0 = format!("R{}", tf.round),
            TfLabel::Timer => t.0 = format!("{mm}:{ss:02}"),
            TfLabel::Kd => t.0 = format!(
                "K {}   D {}   KD {:.2}",
                tf.player_kills, tf.player_deaths, tf.player_kd
            ),
            TfLabel::Popup => t.0 = popup.text.clone(),
            TfLabel::Bomb => {
                t.0 = if !tf.bomb_mode {
                    String::new()
                } else if tf.bomb_planted {
                    let s = (tf.fuse.ceil() as i32).max(0);
                    if s <= 10 {
                        format!("BOM NO: {}s", s)
                    } else {
                        format!("BOM DA GIAO  {}s", s)
                    }
                } else if tf.has_bomb {
                    "GIU E DE GIAO BOM".to_owned()
                } else {
                    String::new()
                };
            }
            TfLabel::Feed(i) => {
                t.0 = match tf.feed.get(*i as usize) {
                    Some(f) if f.life > 0.0 => format!(
                        "{} [{}] {}",
                        f.killer,
                        if f.head { "HS" } else { "K" },
                        f.victim
                    ),
                    _ => String::new(),
                };
            }
            TfLabel::Board(row, col) if show_board => {
                let Some(r) = tf.scoreboard.get(*row as usize) else {
                    continue;
                };
                t.0 = match col {
                    0 => r.name.to_owned(),
                    1 => r.kills.to_string(),
                    2 => r.deaths.to_string(),
                    3 => r.headshots.to_string(),
                    _ => format!("{:.0}", r.damage),
                };
            }
            _ => {}
        }
    }

    // --- 3. thanh HP + giap ---
    for (v, mut n) in nodes.iter_mut() {
        match v {
            TfVis::HpFill => n.width = Val::Percent(tf.player_hp.clamp(0.0, 100.0).max(0.01)),
            TfVis::ArmorFill => n.width = Val::Percent(tf.player_armor.clamp(0.0, 100.0).max(0.01)),
            TfVis::ActionBarFill => {
                let p = if tf.plant_bar > 0.0 { tf.plant_bar } else { tf.defuse_bar };
                n.width = Val::Percent((p.clamp(0.0, 1.0) * 100.0).max(0.01));
            }
            _ => {}
        }
    }

    // --- 4. radar ---
    let ex = world.pose.pos[0];
    let ez = world.pose.pos[2];
    let yaw = world.pose.yaw;
    let pteam = tf.player_team;
    let (sy, cy) = yaw.sin_cos();
    let k = RADAR_SIZE * 0.5 / RADAR_RANGE;
    let mut radar: Vec<(usize, bool, f32, f32)> = Vec::with_capacity(world.bots.len());
    for b in world.bots.iter() {
        let ally = b.team == pteam;
        let dx = b.pos[0] - ex;
        let dz = b.pos[2] - ez;
        let dist = (dx * dx + dz * dz).sqrt();
        let seen = dist < RADAR_RANGE
            && !crate::core::math::segment_blocked(
                ex,
                world.pose.pos[1],
                ez,
                b.pos[0],
                b.pos[1] + 1.2,
                b.pos[2],
            );
        let rx = dx * cy - dz * sy;
        let rz = dx * sy + dz * cy;
        radar.push((b.slot, b.alive && (ally || seen), rx, rz));
    }
    for (v, mut n) in nodes.iter_mut() {
        if let TfVis::Radar(slot) = v {
            if let Some((_, true, rx, rz)) = radar.iter().find(|(s, _, _, _)| s == slot) {
                n.left = Val::Percent(50.0 + (rx * k / RADAR_SIZE) * 100.0);
                n.top = Val::Percent(50.0 + (rz * k / RADAR_SIZE) * 100.0);
            }
        }
    }
    for (v, mut vis_v) in vis.iter_mut() {
        if let TfVis::Radar(slot) = v {
            let show = radar
                .iter()
                .find(|(s, _, _, _)| s == slot)
                .map(|(_, s, _, _)| *s)
                .unwrap_or(false);
            *vis_v = if show {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }

    // --- 5. dem popup ---
    if popup.t > 0.0 {
        popup.t -= time.delta_secs();
    }
}
