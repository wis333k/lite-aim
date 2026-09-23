#![windows_subsystem = "windows"]

pub mod audio;
pub mod camera;
pub mod core;
pub mod editor;
pub mod fx;
pub mod game;
pub mod render;
pub mod ui;
pub mod win_mouse;

use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::ecs::message::{MessageReader, MessageWriter};
use bevy::input::mouse::MouseMotion;
use bevy::render::view::Msaa;
use bevy::window::{CursorGrabMode, CursorOptions};

use crate::audio::sfx::{self, SfxKind};
use crate::core::drills::Keys;
use crate::core::world::{Sfx, World};
use crate::game::{Game, Screen};

#[derive(Component)]
pub struct MainCamera;

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
                    title: "LITE-AIM".into(),
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
        )
        .init_state::<Screen>()
        .insert_resource(ClearColor(Color::srgb(0.03, 0.045, 0.075)))
        .insert_resource(Game::new())
        .insert_resource(World::default())
        .init_resource::<ui::UiRes>()
        .init_resource::<render::GunDirty>()
        .init_resource::<render::MapDirty>()
        .init_resource::<editor::Editor>()
        .init_resource::<editor::EditorDirty>()
        .add_systems(Startup, (setup_scene, setup_audio, render::setup_fx_meshes))
        .add_systems(Update, ui::sync_ui)
        .add_systems(Update, ui::hover_buttons)
        .add_systems(Update, menu_input)
        .add_systems(Update, editor::editor_input)
        .add_systems(Update, playing_input.run_if(in_state(Screen::Playing)))
        .add_systems(
            Update,
            (
                render::sync_bot,
                render::sync_bot_color,
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
            ),
        )
        .add_systems(
            Update,
            (camera::sync_camera, camera::apply_shake)
                .run_if(not(in_state(Screen::Editor))),
        )
        .run();
}

fn setup_audio(mut commands: Commands, mut assets: ResMut<Assets<AudioSource>>) {
    commands.insert_resource(sfx::init(&mut assets));
}

fn setup_scene(
    mut commands: Commands,
    game: Res<Game>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
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

    let mats_res = render::setup_materials(&mut mats);
    render::spawn_arena(&mut commands, &mats_res, &mut meshes);
    render::spawn_bot(&mut commands, &mats_res, &mut meshes);
    render::spawn_gun(&mut commands, &mats_res, &mut meshes, game.current_gun() as u8, cam_entity);
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
        Option<&ui::QuitButton>,
        Option<&ui::LangButton>,
        Option<&ui::QualityButton>,
    ), Changed<Interaction>>,
    q_mode: Query<(&Interaction, &ui::ModeButton), Changed<Interaction>>,
    q_game: Query<(&Interaction, &ui::GameButton), Changed<Interaction>>,
    mut ui_res: ResMut<ui::UiRes>,
    mut gun_dirty: ResMut<render::GunDirty>,
    mut map_dirty: ResMut<render::MapDirty>,
    mut ed: ResMut<editor::Editor>,
    mut ed_dirty: ResMut<editor::EditorDirty>,
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
    for (inter, start, resume, menu, retry, board, gun, map, editor, quit, lang, quality) in q_btn.iter() {
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
            gun_dirty.0 = true;
            ui_res.dirty = true;
        }
        if map.is_some() && screen == Screen::Menu {
            game.cycle_map();
            map_dirty.0 = 1;
            ui_res.dirty = true;
        }
        if editor.is_some() && screen == Screen::Menu {
            ed.begin();
            ed_dirty.0 = true;
            state.set(Screen::Editor);
            ui_res.dirty = true;
        }
        if board.is_some() && screen == Screen::Menu {
            state.set(Screen::Board);
            ui_res.dirty = true;
        }
        if menu.is_some() && (screen == Screen::Paused || screen == Screen::Results || screen == Screen::Board) {
            game.drill = None;
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
) {
    let dt = time.delta_secs();

    if keys.just_pressed(KeyCode::Escape) {
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
        w: keys.pressed(KeyCode::KeyW),
        a: keys.pressed(KeyCode::KeyA),
        s: keys.pressed(KeyCode::KeyS),
        d: keys.pressed(KeyCode::KeyD),
        sprint: keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight),
        crouch: keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight),
        jump: keys.just_pressed(KeyCode::Space),
    };
    // ban: auto giu chuot trai, semi bam tung phat
    world.fire_cd = (world.fire_cd - dt).max(0.0);
    let trigger = if auto { mouse.pressed(MouseButton::Left) } else { mouse.just_pressed(MouseButton::Left) };
    let shot = trigger && world.fire_cd <= 0.0;
    if shot {
        world.fire_cd = fire_interval;
    }

    if let Some(d) = game.drill.as_mut() {
        d.update(&mut world, dt, shot, &k);
        if shot {
            d.on_mousedown(&mut world);
            world.gun_kick = 1.0;
        }
        world.disp_targets = d.targets().to_vec();
        world.show_blocks = d.draw_blocks();
        world.show_gun = d.has_gun();
    }

    let done = game.drill.as_ref().map(|d| d.timer().0 <= 0.0).unwrap_or(false);
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
