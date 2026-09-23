// render: arena mesh, bot mesh, gun — Bevy 3D that (thay fake 3D macroquad)
use bevy::prelude::*;
use crate::core::arena::{current_map, ARENA_SIZE, ARENA_WALL_H};
use crate::core::world::{Kind, World};

#[derive(Component)]
pub struct ArenaRoot;
#[derive(Component)]
pub struct BotBody;
#[derive(Component)]
pub struct BotHead;
#[derive(Component)]
pub struct GunModel;

#[derive(Resource, Default)]
pub struct GunDirty(pub bool);
#[derive(Resource, Default)]
pub struct MapDirty(pub i32);
#[derive(Component)]
pub struct BlocksRoot;
#[derive(Component)]
pub struct ArenaPropRoot;
#[derive(Component)]
pub struct WallHitsRoot;
#[derive(Component)]
pub struct TracerRoot;

#[derive(Component)]
pub struct TargetMesh;

#[derive(Resource)]
pub struct FxMeshes {
    pub sphere: Handle<Mesh>,
    pub unit_cube: Handle<Mesh>,
    pub tracer: Handle<Mesh>,
    pub particle: Handle<Mesh>,
}

pub fn setup_fx_meshes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>) {
    commands.insert_resource(FxMeshes {
        sphere: meshes.add(Sphere::new(1.0)),
        unit_cube: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        tracer: meshes.add(Cuboid::new(0.02, 0.02, 1.0)),
        particle: meshes.add(Cuboid::new(0.06, 0.06, 0.06)),
    });
}

#[derive(Resource)]
pub struct Mats {
    pub floor: Handle<StandardMaterial>,
    pub wall: Handle<StandardMaterial>,
    pub block: Handle<StandardMaterial>,
    pub platform: Handle<StandardMaterial>,
    pub bot_body: Handle<StandardMaterial>,
    pub bot_head: Handle<StandardMaterial>,
    pub gun: Handle<StandardMaterial>,
    pub gun_accent: Handle<StandardMaterial>,
    pub gun_grip: Handle<StandardMaterial>,
    pub gun_lens: Handle<StandardMaterial>,
    pub tracer: Handle<StandardMaterial>,
}

const BOT_BODY_COL: Srgba = Srgba::new(0.85, 0.30, 0.28, 1.0);

pub fn setup_materials(
    mats: &mut Assets<StandardMaterial>,
) -> Mats {
    let m = |mats: &mut Assets<StandardMaterial>, c: Srgba, rough: f32, metal: f32| {
        mats.add(StandardMaterial {
            base_color: c.into(),
            perceptual_roughness: rough,
            metallic: metal,
            ..default()
        })
    };
    let floor = m(mats, Srgba::rgb(0.16, 0.17, 0.20), 0.85, 0.05);
    let wall = m(mats, Srgba::rgb(0.22, 0.23, 0.27), 0.8, 0.1);
    let block = m(mats, Srgba::rgb(0.30, 0.32, 0.36), 0.7, 0.15);
    let platform = m(mats, Srgba::rgb(0.26, 0.34, 0.40), 0.6, 0.2);
    let bot_body = m(mats, BOT_BODY_COL, 0.55, 0.1);
    let bot_head = m(mats, Srgba::rgb(0.92, 0.78, 0.55), 0.5, 0.05);
    let gun = m(mats, Srgba::rgb(0.12, 0.13, 0.15), 0.45, 0.6);
    let gun_accent = m(mats, Srgba::rgb(0.20, 0.21, 0.24), 0.35, 0.85);
    let gun_grip = m(mats, Srgba::rgb(0.09, 0.10, 0.11), 0.85, 0.05);
    let gun_lens = mats.add(StandardMaterial {
        base_color: Srgba::rgb(0.15, 0.45, 0.65).into(),
        emissive: LinearRgba::rgb(0.10, 0.35, 0.55),
        perceptual_roughness: 0.05,
        metallic: 0.2,
        ..default()
    });
    let tracer = mats.add(StandardMaterial {
        base_color: Srgba::rgb(1.0, 0.95, 0.7).into(),
        emissive: LinearRgba::rgb(8.0, 6.0, 2.0),
        unlit: true,
        ..default()
    });
    Mats { floor, wall, block, platform, bot_body, bot_head, gun, gun_accent, gun_grip, gun_lens, tracer }
}

pub fn spawn_arena(
    commands: &mut Commands,
    mats: &Mats,
    meshes: &mut Assets<Mesh>,
) {
    let half = ARENA_SIZE;
    let plane = meshes.add(Plane3d::default().mesh().size(half * 2.0, half * 2.0));
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cyl = meshes.add(Cylinder::new(1.0, 2.0));
    let wall_h = ARENA_WALL_H;

    commands.spawn((
        Mesh3d(plane),
        MeshMaterial3d(mats.floor.clone()),
        Transform::from_xyz(0.0, 0.0, 0.0),
        ArenaRoot,
    ));
    let walls = [
        (0.0, wall_h * 0.5, -half, half * 2.0, wall_h, 0.4),
        (0.0, wall_h * 0.5, half, half * 2.0, wall_h, 0.4),
        (-half, wall_h * 0.5, 0.0, 0.4, wall_h, half * 2.0),
        (half, wall_h * 0.5, 0.0, 0.4, wall_h, half * 2.0),
    ];
    for (x, y, z, sx, sy, sz) in walls {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(mats.wall.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::new(sx, sy, sz)),
            ArenaRoot,
        ));
    }
    spawn_map_blocks(commands, mats, &cube, &cyl);
    commands.spawn((Transform::default(), Visibility::default(), WallHitsRoot));
    commands.spawn((Transform::default(), Visibility::default(), TracerRoot));
}

// spawn blocks + extras + props cua map dang chon, gom vao BlocksRoot/ArenaPropRoot de rebuild
fn spawn_map_blocks(
    commands: &mut Commands,
    mats: &Mats,
    cube: &Handle<Mesh>,
    cyl: &Handle<Mesh>,
) {
    let m = current_map();
    let blocks = commands.spawn((Transform::default(), Visibility::default(), BlocksRoot)).id();
    let push_block = |commands: &mut Commands, b: &[f32; 6]| {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        let mat = if hh <= 0.3 { mats.platform.clone() } else { mats.block.clone() };
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(bx, by, bz).with_scale(Vec3::new(hw * 2.0, hh * 2.0, hd * 2.0)),
            ChildOf(blocks),
        ));
    };
    for b in m.blocks.iter() {
        push_block(commands, b);
    }
    let extras = commands.spawn((Transform::default(), Visibility::default(), BlocksRoot)).id();
    for b in m.extras.iter() {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        let mat = if hh <= 0.3 { mats.platform.clone() } else { mats.block.clone() };
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(bx, by, bz).with_scale(Vec3::new(hw * 2.0, hh * 2.0, hd * 2.0)),
            ChildOf(extras),
        ));
    }

    // prop trang tri luon hien: cot, thung, container
    let prop_root = commands.spawn((Transform::default(), Visibility::default(), ArenaPropRoot)).id();
    for p in m.props.iter() {
        let (x, y, z, kind, sc) = (p[0], p[1], p[2], p[3] as i32, p[4]);
        let (mesh, mat, scl) = match kind {
            0 => (cyl.clone(), mats.wall.clone(), Vec3::new(0.35 * sc, sc, 0.35 * sc)),
            1 => (cube.clone(), mats.block.clone(), Vec3::new(sc * 0.9, sc * 0.9, sc * 0.9)),
            2 => (cube.clone(), mats.platform.clone(), Vec3::new(sc * 1.0, sc * 1.0, sc * 3.0)),
            _ => (cube.clone(), mats.wall.clone(), Vec3::new(sc * 0.6, sc, sc * 0.6)),
        };
        commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_xyz(x, y, z).with_scale(scl),
            ChildOf(prop_root),
        ));
    }
}

// doi map: xoa blocks + props cu, spawn lai theo map moi
pub fn rebuild_map(
    mut commands: Commands,
    mut dirty: ResMut<MapDirty>,
    mats: Option<Res<Mats>>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_blocks: Query<Entity, With<BlocksRoot>>,
    q_props: Query<Entity, With<ArenaPropRoot>>,
) {
    if dirty.0 < 0 { return; }
    dirty.0 = -1;
    let Some(mats) = mats else { return };
    for e in q_blocks.iter().chain(q_props.iter()) {
        commands.entity(e).despawn();
    }
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cyl = meshes.add(Cylinder::new(1.0, 2.0));
    spawn_map_blocks(&mut commands, &mats, &cube, &cyl);
}

pub fn spawn_bot(
    commands: &mut Commands,
    mats: &Mats,
    meshes: &mut Assets<Mesh>,
) {
    let body = meshes.add(Capsule3d::new(0.32, 1.0));
    let head = meshes.add(Sphere::new(0.30));
    commands.spawn((
        Mesh3d(body),
        MeshMaterial3d(mats.bot_body.clone()),
        Transform::from_xyz(0.0, 1.05, -10.0),
        BotBody,
    ));
    commands.spawn((
        Mesh3d(head),
        MeshMaterial3d(mats.bot_head.clone()),
        Transform::from_xyz(0.0, 1.92, -10.0),
        BotHead,
    ));
}

// viewmodel: sung cam tay nhieu part, parent vao camera
// kind: 0=Pistol 1=Rifle 2=Sniper 3=Smg
pub fn spawn_gun(
    commands: &mut Commands,
    mats: &Mats,
    meshes: &mut Assets<Mesh>,
    kind: u8,
    camera: Entity,
) {
    let root = commands.spawn((
        Transform::default(),
        Visibility::default(),
        GunModel,
        ChildOf(camera),
    )).id();

    // mesh dung chung
    let boxm = |meshes: &mut Assets<Mesh>, x: f32, y: f32, z: f32| meshes.add(Cuboid::new(x, y, z));
    let cyl = |meshes: &mut Assets<Mesh>, r: f32, h: f32| meshes.add(Cylinder::new(r, h));

    let dark = mats.gun.clone();
    let accent = mats.gun_accent.clone();
    let grip = mats.gun_grip.clone();

    let part = |commands: &mut Commands, mesh: Handle<Mesh>, mat: Handle<StandardMaterial>, t: Transform| {
        commands.spawn((Mesh3d(mesh), MeshMaterial3d(mat), t, ChildOf(root)));
    };

    match kind {
        0 => {
            // PISTOL: than ngan, grip nghieng, no'ng ngan
            let body = boxm(meshes, 0.055, 0.075, 0.17);
            part(commands, body, dark.clone(), Transform::from_xyz(0.0, 0.0, 0.0));
            let slide = boxm(meshes, 0.05, 0.035, 0.19);
            part(commands, slide, accent.clone(), Transform::from_xyz(0.0, 0.052, -0.005));
            let barrel = cyl(meshes, 0.016, 0.10);
            part(commands, barrel, dark.clone(), Transform::from_xyz(0.0, 0.0, -0.13).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let g = boxm(meshes, 0.045, 0.13, 0.06);
            part(commands, g, grip.clone(), Transform::from_xyz(0.0, -0.095, 0.085).with_rotation(Quat::from_rotation_x(-0.30)));
            let sight = boxm(meshes, 0.012, 0.018, 0.012);
            part(commands, sight, accent.clone(), Transform::from_xyz(0.0, 0.075, -0.075));
        }
        1 => {
            // RIFLE: than dai, bang dan cong, no'ng + phanh giam giat
            let body = boxm(meshes, 0.06, 0.085, 0.50);
            part(commands, body, dark.clone(), Transform::from_xyz(0.0, 0.0, 0.0));
            let rail = boxm(meshes, 0.045, 0.02, 0.34);
            part(commands, rail, accent.clone(), Transform::from_xyz(0.0, 0.052, -0.02));
            let barrel = cyl(meshes, 0.017, 0.30);
            part(commands, barrel, dark.clone(), Transform::from_xyz(0.0, 0.01, -0.36).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let brake = cyl(meshes, 0.026, 0.06);
            part(commands, brake, accent.clone(), Transform::from_xyz(0.0, 0.01, -0.50).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            // bang dan cong
            let mag = boxm(meshes, 0.05, 0.17, 0.09);
            part(commands, mag, grip.clone(), Transform::from_xyz(0.0, -0.115, 0.02).with_rotation(Quat::from_rotation_x(0.12)));
            let g = boxm(meshes, 0.05, 0.13, 0.07);
            part(commands, g, grip.clone(), Transform::from_xyz(0.0, -0.10, 0.20).with_rotation(Quat::from_rotation_x(-0.22)));
            let stock = boxm(meshes, 0.05, 0.09, 0.20);
            part(commands, stock, dark.clone(), Transform::from_xyz(0.0, -0.02, 0.34));
            let sight = boxm(meshes, 0.014, 0.022, 0.02);
            part(commands, sight, accent.clone(), Transform::from_xyz(0.0, 0.075, -0.10));
        }
        2 => {
            // SNIPER: than rat dai, scope to, chan 2 chan
            let body = boxm(meshes, 0.06, 0.09, 0.62);
            part(commands, body, dark.clone(), Transform::from_xyz(0.0, 0.0, 0.0));
            let barrel = cyl(meshes, 0.016, 0.52);
            part(commands, barrel, dark.clone(), Transform::from_xyz(0.0, 0.012, -0.56).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let muzzle = cyl(meshes, 0.028, 0.09);
            part(commands, muzzle, accent.clone(), Transform::from_xyz(0.0, 0.012, -0.84).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            // scope
            let scope = cyl(meshes, 0.035, 0.26);
            part(commands, scope, accent.clone(), Transform::from_xyz(0.0, 0.095, -0.08).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let lens = cyl(meshes, 0.032, 0.012);
            part(commands, lens, mats.gun_lens.clone(), Transform::from_xyz(0.0, 0.095, -0.215).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let mag = boxm(meshes, 0.045, 0.13, 0.08);
            part(commands, mag, grip.clone(), Transform::from_xyz(0.0, -0.10, 0.05));
            let g = boxm(meshes, 0.05, 0.13, 0.07);
            part(commands, g, grip.clone(), Transform::from_xyz(0.0, -0.10, 0.24).with_rotation(Quat::from_rotation_x(-0.20)));
            let stock = boxm(meshes, 0.05, 0.10, 0.26);
            part(commands, stock, dark.clone(), Transform::from_xyz(0.0, -0.02, 0.42));
        }
        _ => {
            // SMG: ngan gon, bang dan dai, grip truoc
            let body = boxm(meshes, 0.055, 0.08, 0.34);
            part(commands, body, dark.clone(), Transform::from_xyz(0.0, 0.0, 0.0));
            let barrel = cyl(meshes, 0.015, 0.16);
            part(commands, barrel, dark.clone(), Transform::from_xyz(0.0, 0.006, -0.24).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let supp = cyl(meshes, 0.026, 0.14);
            part(commands, supp, accent.clone(), Transform::from_xyz(0.0, 0.006, -0.36).with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)));
            let mag = boxm(meshes, 0.05, 0.22, 0.08);
            part(commands, mag, grip.clone(), Transform::from_xyz(0.0, -0.14, 0.03).with_rotation(Quat::from_rotation_x(0.16)));
            let g = boxm(meshes, 0.05, 0.12, 0.065);
            part(commands, g, grip.clone(), Transform::from_xyz(0.0, -0.095, 0.16).with_rotation(Quat::from_rotation_x(-0.24)));
            let fg = boxm(meshes, 0.04, 0.10, 0.05);
            part(commands, fg, grip.clone(), Transform::from_xyz(0.0, -0.09, -0.16).with_rotation(Quat::from_rotation_x(0.20)));
            let sight = boxm(meshes, 0.013, 0.02, 0.016);
            part(commands, sight, accent.clone(), Transform::from_xyz(0.0, 0.07, -0.08));
        }
    }
}

// viewmodel: bam theo camera + bob theo di chuyen + recoil kick
pub fn sync_gun(
    world: Res<World>,
    game: Res<crate::game::Game>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut Transform, With<GunModel>>,
    mut vis: Query<&mut Visibility, (With<GunModel>, Without<BlocksRoot>)>,
) {
    let dt = time.delta_secs();
    // dao dong khi di chuyen
    let moving = game.drill.is_some()
        && (keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::KeyA)
            || keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::KeyD));
    let speed = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) { 1.6 } else { 1.0 };
    let t = world.tick_time * if moving { 8.0 * speed } else { 1.8 };
    let bob_x = if moving { (t).sin() * 0.010 } else { (t).sin() * 0.0022 };
    let bob_y = if moving { (t * 2.0).sin() * 0.008 } else { (t * 2.0).sin() * 0.0018 };

    let kick = world.gun_kick;
    // kick: lui ve sau (+z), ngoc len, giat ngang nhe
    let kx = (world.tick_time * 55.0).sin() * kick * 0.006;
    let kz = kick * 0.075;
    let ky = kick * 0.030;
    let rx = kick * 0.16;

    for mut tr in q.iter_mut() {
        tr.translation = Vec3::new(0.17 + bob_x + kx, -0.16 + bob_y + ky, -0.42 + kz);
        tr.rotation = Quat::from_euler(
            EulerRot::XYZ,
            rx + bob_y * 1.5,
            -0.06 + bob_x * 2.0,
            kx * 3.0,
        );
        let _ = dt;
    }
    for mut v in vis.iter_mut() {
        *v = if world.show_gun { Visibility::Inherited } else { Visibility::Hidden };
    }
}

// doi sung: xoa viewmodel cu, spawn lai theo kind moi (khi bam nut chon sung)
pub fn rebuild_gun(
    mut commands: Commands,
    mut dirty: ResMut<GunDirty>,
    game: Res<crate::game::Game>,
    mats: Option<Res<Mats>>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_old: Query<Entity, With<GunModel>>,
    q_cam: Query<Entity, With<crate::MainCamera>>,
) {
    if !dirty.0 { return; }
    dirty.0 = false;
    let Some(mats) = mats else { return };
    let Ok(cam) = q_cam.single() else { return };
    for e in q_old.iter() {
        commands.entity(e).despawn();
    }
    spawn_gun(&mut commands, &mats, &mut meshes, game.current_gun() as u8, cam);
}

// cap nhat vi tri bot + visibility theo drill hien tai
pub fn sync_bot(
    world: Res<World>,
    mut q_body: Query<(&mut Transform, &mut Visibility), (With<BotBody>, Without<BotHead>)>,
    mut q_head: Query<(&mut Transform, &mut Visibility), (With<BotHead>, Without<BotBody>)>,
) {
    match bot_display(&world) {
        Some((x, y, z, alive)) => {
            for (mut tr, mut vis) in q_body.iter_mut() {
                tr.translation = Vec3::new(x, y, z);
                *vis = if alive { Visibility::Inherited } else { Visibility::Hidden };
            }
            for (mut tr, mut vis) in q_head.iter_mut() {
                tr.translation = Vec3::new(x, y + 0.87, z);
                *vis = if alive { Visibility::Inherited } else { Visibility::Hidden };
            }
        }
        None => {
            for (_, mut vis) in q_body.iter_mut() { *vis = Visibility::Hidden; }
            for (_, mut vis) in q_head.iter_mut() { *vis = Visibility::Hidden; }
        }
    }
}

pub fn sync_bot_color(
    world: Res<World>,
    mats: Res<Mats>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let flash = world.bot_hit();
    if let Some(m) = materials.get_mut(&mats.bot_body) {
        let c = if flash { Srgba::new(1.0, 1.0, 1.0, 1.0) } else { BOT_BODY_COL };
        m.base_color = c.into();
    }
}

pub fn toggle_blocks(world: Res<World>, mut q: Query<&mut Visibility, With<BlocksRoot>>) {
    for mut vis in q.iter_mut() {
        *vis = if world.show_blocks { Visibility::Inherited } else { Visibility::Hidden };
    }
}

// bot display (x, y_body_center, z, alive) — y da la tam hitbox body
fn bot_display(w: &World) -> Option<(f32, f32, f32, bool)> {
    let t = w.bot_target()?;
    Some((t.x, t.y, t.z, t.alive))
}

pub fn kind_is_bot(k: Kind) -> bool {
    matches!(k, Kind::Bot)
}

// ve target tinh (gridshot/flick/tracking) + bot phu (duel) bang sphere mau
pub fn sync_targets(
    mut commands: Commands,
    world: Res<World>,
    mats: Res<Mats>,
    fx_meshes: Res<FxMeshes>,
    q: Query<Entity, With<TargetMesh>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
    if world.disp_targets.is_empty() {
        return;
    }
    for t in world.disp_targets.iter() {
        if t.kind == Kind::Bot || !t.alive {
            continue;
        }
        let mat = if t.kind == Kind::Track { mats.bot_head.clone() } else { mats.tracer.clone() };
        commands.spawn((
            Mesh3d(fx_meshes.sphere.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(t.x, t.y, t.z).with_scale(Vec3::splat(t.r)),
            TargetMesh,
        ));
    }
}pub fn crosshair_color(idx: usize) -> Color {
    match idx {
        0 => Color::srgb(0.20, 0.85, 1.0),
        1 => Color::srgb(0.30, 1.0, 0.35),
        2 => Color::srgb(1.0, 0.85, 0.20),
        3 => Color::srgb(1.0, 0.30, 0.35),
        4 => Color::srgb(1.0, 1.0, 1.0),
        _ => Color::srgb(0.85, 0.40, 1.0),
    }
}
