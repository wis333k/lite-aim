#![windows_subsystem = "windows"]

pub mod audio;
pub mod camera;
pub mod core;
pub mod editor;
pub mod fx;
pub mod game;
pub mod render;
pub mod ui;
pub mod ui_tf;
pub mod win_mouse;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::input::mouse::MouseMotion;
use bevy::render::view::Msaa;
use bevy::window::{CursorGrabMode, CursorOptions};

use crate::audio::sfx::{self, SfxKind};
use crate::core::drills::{Drill, Keys};
use crate::core::world::{Sfx, World};
use crate::game::{Bench, Game, KeyWait, Screen};

mod embedded_assets {
    include!(concat!(env!("OUT_DIR"), "/embedded_assets.rs"));
}

// nhung toan bo assets/ vao exe (embedded asset source) — chay 1 lan luc startup
fn register_embedded(
    reg: Option<Res<bevy::asset::io::embedded::EmbeddedAssetRegistry>>,
) {
    let Some(reg) = reg else { return };
    embedded_assets::register_embedded_assets(&reg);
}

#[derive(Component)]
pub struct MainCamera;

// logo da load xong -> rebuild UI 1 lan de anh hien
#[derive(Resource, Default)]
pub struct LogoReady(pub bool);

// thu muc assets: canh exe (khong phu thuoc cwd khi double-click)
fn assets_dir() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let a = dir.join("assets");
            if a.exists() {
                return a.to_string_lossy().into_owned();
            }
        }
    }
    "assets".to_owned()
}

fn main() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {info}\n");
        let _ = std::fs::write(std::env::temp_dir().join("ac_panic.txt"), msg);
        eprintln!("{info}");
    }));
    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "WLITE".into(),
                    resolution: (1280u32, 720u32).into(),
                    resizable: true,
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    ..default()
                }),
                ..default()
            })
            .set(bevy::render::RenderPlugin {
                render_creation: bevy::render::settings::WgpuSettings {
                    priority: bevy::render::settings::WgpuSettingsPriority::Functionality,
                    ..default()
                }
                .into(),
                ..default()
            })
            .set(bevy::asset::AssetPlugin {
                file_path: assets_dir(),
                ..default()
            })
        )
        .init_state::<Screen>()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.045, 0.075)))
        .insert_resource(Game::new())
        .insert_resource(World::default())
        .init_resource::<ui::UiRes>()
        .init_resource::<ui_tf::TfPopupState>()
        .init_resource::<render::GunDirty>()
        .init_resource::<render::MapDirty>()
        .init_resource::<render::BotAnimPlayer>()
        .init_resource::<render::BotVisual>()
        .init_resource::<editor::Editor>()
        .init_resource::<editor::EditorDirty>()
        .init_resource::<KeyWait>()
        .init_resource::<Bench>()
        .init_resource::<LogoReady>()
        .add_systems(Startup, (register_embedded, setup_scene, setup_audio, render::setup_fx_meshes).chain())
        .add_systems(Update, ui::sync_ui)
        .add_systems(Update, ui::hover_buttons)
        .add_systems(Update, menu_input)
        .add_systems(Update, apply_fps_limit)
        .add_systems(Update, ui::keybind_capture)
        .add_systems(Update, bench_tick)
        .add_systems(Update, logo_ready_sync)
        .add_systems(Update, editor::editor_input)
        .add_systems(Update, playing_input.run_if(in_state(Screen::Playing)))
        .add_systems(
            Update,
            (
                render::sync_bot,
                render::sync_bot_color,
                render::setup_bot_anim,
                render::sync_targets,
                render::toggle_blocks,
                render::sync_gun,
                render::rebuild_gun,
                render::rebuild_map,
                editor::sync_editor,
                fx::clear_particles,
                fx::sync_tracers,
                fx::sync_wall_hits,
                fx::sync_particles,
                ui::update_hud,
                ui::update_vignette,
                ui::update_crosshair,
                play_queued_sfx,
                render::env_convert,
            ),
        )
        .add_systems(
            Update,
            (
                render::botpool::setup_botpool_anim,
                render::botpool::setup_botpool_mat,
                render::botpool::hide_when_no_match,
            ),
        )
        .add_systems(
            Update,
            (render::botpool::sync_botpool, render::botpool::sync_botpool_anim),
        )
        .add_systems(Update, ui_tf::update_tf)
        .add_systems(
            Update,
            (camera::sync_camera, camera::apply_shake)
                .run_if(not(in_state(Screen::Editor))),
        )
        .run();
}

fn setup_audio(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(sfx::init(&asset_server));
}

fn setup_scene(
    mut commands: Commands,
    game: Res<Game>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let high = game.quality == 1;
    crate::core::arena::load_custom_maps();
    crate::core::arena::set_active_map(game.map_id);
    let mut cam = commands.spawn((
        Camera3d::default(),
        Camera { ..default() },
        Projection::Perspective(PerspectiveProjection { fov: 103f32.to_radians(), ..default() }),
        Transform::from_xyz(0.0, 1.6, 5.0).looking_at(Vec3::new(0.0, 1.6, -8.0), Vec3::Y),
        DistanceFog {
            color: Color::srgb(0.10, 0.13, 0.18),
            falloff: FogFalloff::Linear { start: 18.0, end: 40.0 },
            ..default()
        },
        MainCamera,
    ));
    let cam_entity = cam.id();
    if high {
        cam.insert((Bloom::NATURAL, Tonemapping::TonyMcMapface, Msaa::Off));
    } else {
        cam.insert((Tonemapping::None, Msaa::Off));
    }

    // HDRI skybox + IBL (environment map) — nang cap do hoa
    render::setup_environment(&mut commands, &mut images, cam_entity, &asset_server, high);

    commands.spawn(AmbientLight {
        color: Color::srgb(0.55, 0.60, 0.70),
        brightness: if high { 450.0 } else { 350.0 },
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: if high { 4200.0 } else { 3200.0 },
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(6.0, 12.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let mats_res = render::setup_materials(&mut mats, &asset_server);
    render::spawn_arena(&mut commands, &mats_res, &mut meshes);
    let bot_anim = render::build_bot_anim(&mut graphs, &asset_server);
    commands.insert_resource(bot_anim);
    render::spawn_bot(&mut commands, &asset_server);
    render::botpool::spawn_botpool(&mut commands, &asset_server);
    render::spawn_gun(&mut commands, &asset_server, game.current_gun() as u8, cam_entity);
    commands.insert_resource(mats_res);
}

// ==== menu / pause / results input ====
fn menu_input(
    mut state: ResMut<NextState<Screen>>,
    cur: Res<State<Screen>>,
    mut game: ResMut<Game>,
    q_btn: Query<(
        &Interaction,
        Option<&ui::StartButton>,
        Option<&ui::ResumeButton>,
        Option<&ui::MenuButton>,
        Option<&ui::RetryButton>,
        Option<&ui::BoardButton>,
        Option<&ui::GunButton>,
        Option<&ui::MapButton>,
        Option<&ui::EditorButton>,
        Option<&ui::SettingsButton>,
        Option<&ui::AboutButton>,
        Option<&ui::QuitButton>,
        Option<&ui::LangButton>,
        Option<&ui::QualityButton>,
    ), Changed<Interaction>>,
    q_mode: Query<(&Interaction, &ui::ModeButton), Changed<Interaction>>,
    q_game: Query<(&Interaction, &ui::GameButton), Changed<Interaction>>,
    q_set: Query<(&Interaction, Option<&ui::SetMinus>, Option<&ui::SetPlus>), Changed<Interaction>>,
    q_key: Query<(&Interaction, &ui::KeyBindButton), Changed<Interaction>>,
    q_bench: Query<(&Interaction, &ui::BenchButton), Changed<Interaction>>,
    q_bf: Query<(&Interaction, &ui::BoardFilterButton), Changed<Interaction>>,
    mut ui_res: ResMut<ui::UiRes>,
    mut aux: ParamSet<(
        ResMut<render::GunDirty>,
        ResMut<render::MapDirty>,
        ResMut<editor::Editor>,
        ResMut<editor::EditorDirty>,
        ResMut<KeyWait>,
        ResMut<Bench>,
    )>,
    mut exit: MessageWriter<AppExit>,
) {
    let screen = *cur.get();

    for (inter, mb) in q_mode.iter() {
        if *inter == Interaction::Pressed {
            game.mode_id = mb.0;
            ui_res.dirty = true;
        }
    }
    for (inter, gb) in q_game.iter() {
        if *inter == Interaction::Pressed {
            game.game_id = gb.0;
            game.save_cfg();
            ui_res.dirty = true;
        }
    }
    // settings +/- (id o SetMinus hoac SetPlus; id dung chung SET_*)
    for (inter, minus, plus) in q_set.iter() {
        if *inter != Interaction::Pressed { continue; }
        let (id, plus_dir) = if let Some(p) = plus { (p.0, true) }
            else if let Some(m) = minus { (m.0, false) }
            else { continue };
        ui::apply_setting(&mut game, id, plus_dir);
        if id == ui::SET_GUN { aux.p0().0 = true; }
        ui_res.dirty = true;
    }
    // keybind: bam nut -> cho phim
    for (inter, kb) in q_key.iter() {
        if *inter == Interaction::Pressed {
            aux.p4().0 = Some(kb.0);
            ui_res.dirty = true;
        }
    }
    // benchmark
    // board filter
    for (inter, bf) in q_bf.iter() {
        if *inter == Interaction::Pressed {
            game.board_filter = if bf.0 == 0 { 5 } else { bf.0 - 1 };
            ui_res.dirty = true;
        }
    }
    for (inter, _) in q_bench.iter() {        if *inter == Interaction::Pressed && screen == Screen::Settings {
            let mut b = aux.p5();
            b.active = true;
            b.t = 0.0;
            b.frames = 0;
            b.sum_dt = 0.0;
            b.min_fps = 1e9;
            b.max_fps = 0.0;
            b.result = None;
            state.set(Screen::Playing);
            ui_res.dirty = true;
        }
    }
    for (inter, start, resume, menu, retry, board, gun, map, editor, settings, about, quit, lang, quality) in q_btn.iter() {
        if *inter != Interaction::Pressed { continue; }
        if lang.is_some() {
            game.lang = 1 - game.lang;
            game.save_cfg();
            ui_res.dirty = true;
        }
        if quality.is_some() {
            game.quality = 1 - game.quality;
            game.save_cfg();
        }
        if start.is_some() && screen == Screen::Menu {
            game.start_mode();
            state.set(Screen::Playing);
            ui_res.dirty = true;
        }
        if resume.is_some() && screen == Screen::Paused {
            state.set(Screen::Playing);
            ui_res.dirty = true;
        }
        if retry.is_some() && screen == Screen::Results {
            game.start_mode();
            state.set(Screen::Playing);
            ui_res.dirty = true;
        }
        if gun.is_some() && screen == Screen::Menu {
            game.cycle_gun();
            aux.p0().0 = true;
            ui_res.dirty = true;
        }
        if map.is_some() && screen == Screen::Menu {
            game.cycle_map();
            aux.p1().0 = 1;
            ui_res.dirty = true;
        }
        if editor.is_some() && screen == Screen::Menu {
            aux.p2().begin();
            aux.p3().0 = true;
            state.set(Screen::Editor);
            ui_res.dirty = true;
        }
        if settings.is_some() && screen == Screen::Menu {
            state.set(Screen::Settings);
            ui_res.dirty = true;
        }
        if about.is_some() {
            crate::ui::open_url("https://wis333k.github.io/");
        }
        if board.is_some() && screen == Screen::Menu {
            state.set(Screen::Board);
            ui_res.dirty = true;
        }
        if menu.is_some() && (screen == Screen::Paused || screen == Screen::Results || screen == Screen::Board || screen == Screen::Settings) {
            game.drill = None;
            aux.p5().active = false;
            state.set(Screen::Menu);
            ui_res.dirty = true;
        }
        if quit.is_some() {
            exit.write(AppExit::Success);
        }
    }
}

// ==== gameplay ====
fn playing_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut world: ResMut<World>,
    mut next: ResMut<NextState<Screen>>,
    mut ui_res: ResMut<ui::UiRes>,
    mut windows: Query<&mut CursorOptions>,
    mut popup: ResMut<ui_tf::TfPopupState>,
) {
    let dt = time.delta_secs();

    // doc keybind tu game (index: 0 W,1 S,2 A,3 D,4 sprint,5 jump,6 crouch,7 reload,8 swap,9 pause)
    let kb: [Option<KeyCode>; 10] =
        std::array::from_fn(|i| game.keycode_at(i));
    let kp = |i: usize| kb[i].map(|k| keys.pressed(k)).unwrap_or(false);
    let kc = |i: usize| kb[i];

    if kc(9).map(|k| keys.just_pressed(k)).unwrap_or(false) || keys.just_pressed(KeyCode::Escape) {
        game.release_grab();
        set_grab(&mut windows, false);
        next.set(Screen::Paused);
        ui_res.dirty = true;
        return;
    }

    if !game.grab_active {
        game.grab_active = true;
        game.set_cursor_visible(false);
        set_grab(&mut windows, true);
    }

    camera::mouse_look(&mut world, &game, &mut motion);

    world.fov = game.fov;
    world.gun_kind = game.current_gun() as u8;
    let (rpm, auto, zoom) = world.gun_stats();
    let fire_interval = 60.0 / rpm.max(1.0);

    // ADS (sniper): giu chuot phai
    let want_ads = zoom > 0.0 && mouse.pressed(MouseButton::Right);
    let ads_t = (dt / 0.12).clamp(0.0, 1.0);
    world.ads = if want_ads { (world.ads + ads_t).min(1.0) } else { (world.ads - ads_t).max(0.0) };

    world.fx_parts = game.particles;
    world.fx_dmg = game.dmgnum;
    world.fx_hit = game.hitmark_on;
    world.shake_on = game.shake_on;
    world.tick_fx(dt);
    sfx::set_volume(game.eff_volume());

    let k = Keys {
        w: kp(0),
        a: kp(2),
        s: kp(1),
        d: kp(3),
        sprint: kp(4),
        crouch: kp(6),
        jump: kc(5).map(|c| keys.just_pressed(c)).unwrap_or(false),
    };
    // ban: auto giu chuot trai, semi bam tung phat
    world.fire_cd = (world.fire_cd - dt).max(0.0);
    let trigger = if auto { mouse.pressed(MouseButton::Left) } else { mouse.just_pressed(MouseButton::Left) };
    let shot = trigger && world.fire_cd <= 0.0;
    if shot {
        world.fire_cd = fire_interval;
    }

    let mut tf_over = false;
    if let Some(d) = game.drill.as_mut() {
        // phim T: doi doi trong mode 5v5
        if d.is_teamfight() && keys.just_pressed(KeyCode::KeyT) {
            if let Drill::TeamFight(tf) = d {
                tf.manual_switch(&mut world);
            }
        }
        // giu E: trong bom / gỡ bom (mode 5V5 BOMB)
        if let Drill::TeamFight(tf) = d {
            tf.holding = keys.pressed(KeyCode::KeyE);
        }
        d.update(&mut world, dt, shot, &k);
        if shot {
            match d {
                Drill::TeamFight(tf) => {
                    // player ban: kiem tra ket qua de hien popup
                    let streak_before = tf.gm.actors[tf.gm.player_id].kills;
                    if let Some((_id, head, _dmg)) = tf.player_shoot(&mut world) {
                        let streak = tf.gm.actors[tf.gm.player_id].kills;
                        let msg = if head {
                            Some("HEADSHOT".to_owned())
                        } else if streak >= 2 && streak > streak_before {
                            Some(format!("{streak} KILL STREAK"))
                        } else {
                            None
                        };
                        if let Some(m) = msg {
                            popup.text = m;
                            popup.t = 1.6;
                        }
                    }
                }
                _ => {
                    d.on_mousedown(&mut world);
                }
            }
            world.gun_kick = 1.0;
        }
        world.disp_targets = d.targets().to_vec();
        world.show_blocks = d.draw_blocks();
        world.show_gun = d.has_gun();
        if let Drill::TeamFight(tf) = d {
            tf_over = tf.is_over();
        }
    }

    let done = tf_over
        || game.drill.as_ref().map(|d| d.timer().0 <= 0.0).unwrap_or(false);
    if done {
        let res = game.drill.as_ref().unwrap().results();
        let is_best = game.stats.submit(res.mode_id, res.score_num);
        game.best_flash = is_best;
        game.result = Some(res);
        game.record_result();
        game.release_grab();
        set_grab(&mut windows, false);
        game.drill = None;
        next.set(Screen::Results);
        ui_res.dirty = true;
    }
}

// khi logo load xong (hoac loi), danh dau de rebuild UI 1 lan
fn logo_ready_sync(
    asset_server: Res<AssetServer>,
    mut ready: ResMut<LogoReady>,
    mut ui_res: ResMut<ui::UiRes>,
    mut ev: MessageReader<bevy::asset::AssetEvent<Image>>,
) {
    if ready.0 {
        return;
    }
    for e in ev.read() {
        if let bevy::asset::AssetEvent::LoadedWithDependencies { .. } = e {
            ready.0 = true;
            ui_res.dirty = true;
        }
    }
    let _ = &asset_server;
}

// do fps: tich luy trong khi Playing & bench.active, ket thuc -> in ket qua ra file
fn bench_tick(
    time: Res<Time>,
    state: Res<State<Screen>>,
    mut bench: ResMut<Bench>,
    mut game: ResMut<Game>,
    mut ui_res: ResMut<ui::UiRes>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !bench.active { return; }
    if *state.get() != Screen::Playing { return; }
    let dt = time.delta_secs();
    if dt <= 0.0 { return; }
    let fps = 1.0 / dt;
    bench.t += dt;
    bench.frames += 1;
    bench.sum_dt += dt;
    bench.min_fps = bench.min_fps.min(fps);
    bench.max_fps = bench.max_fps.max(fps);
    // ket thuc sau 20s hoac drill xong
    let drill_done = game.drill.as_ref().map(|d| d.timer().0 <= 0.0).unwrap_or(false);
    if bench.t >= 20.0 || drill_done {
        let avg = if bench.sum_dt > 0.0 { bench.frames as f32 / bench.sum_dt } else { 0.0 };
        let txt = format!(
            "BENCHMARK: avg {:.1} fps | {} frames | {:.2} ms | min {:.0} | max {:.0}",
            avg, bench.frames, 1000.0 / avg.max(0.001), bench.min_fps, bench.max_fps
        );
        // luu file + ghi vao result
        if let Some(dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
            let _ = std::fs::write(dir.join("benchmark.txt"), &txt);
        }
        bench.result = Some(txt.clone());
        bench.active = false;
        game.drill = None;
        game.best_flash = false;
        game.result = Some(crate::core::config::Results::new(5, "BENCHMARK", txt, 0.0, 0, 0.0));
        next.set(Screen::Results);
        ui_res.dirty = true;
    }
}

// gioi han fps: doi PresentMode khi cai dat thay doi
fn apply_fps_limit(
    game: Res<Game>,
    mut last: Local<u32>,
    mut wins: Query<&mut Window>,
) {
    if game.fps_limit == *last && *last != 0 { return; }
    let first = *last == 0 && game.fps_limit == 0;
    *last = game.fps_limit;
    if first { return; }
    let mode = if game.fps_limit == 0 {
        bevy::window::PresentMode::AutoNoVsync
    } else {
        // Fifo cap o refresh man hinh; khong co API cap fps truc tiep -> dung Fifo cho <= refresh
        bevy::window::PresentMode::Fifo
    };
    for mut w in wins.iter_mut() {
        w.present_mode = mode;
        if game.fps_limit != 0 {
            w.desired_maximum_frame_latency = std::num::NonZero::new(1);
        }
    }
}

// bat/tat khoa chuot qua winit (thay FFI SetCursorPos loop)
fn set_grab(windows: &mut Query<&mut CursorOptions>, on: bool) {
    let Ok(mut c) = windows.single_mut() else { return };
    c.visible = !on;
    c.grab_mode = if on { CursorGrabMode::Locked } else { CursorGrabMode::None };
}

fn play_queued_sfx(mut commands: Commands, mut world: ResMut<World>, bank: Res<sfx::SfxBank>) {
    if world.sfx.is_empty() {
        return;
    }
    let queued: Vec<Sfx> = world.sfx.drain(..).collect();
    for s in queued {
        let kind = match s {
            Sfx::Shoot => SfxKind::Shoot,
            Sfx::Hit => SfxKind::Hit,
            Sfx::Head => SfxKind::Head,
            Sfx::Kill => SfxKind::Kill,
            Sfx::Click => SfxKind::Click,
            Sfx::Death => SfxKind::Death,
        };
        sfx::play(&mut commands, &bank, kind);
    }
}
