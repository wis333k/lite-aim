// render: arena mesh, bot mesh, gun — Bevy 3D that (thay fake 3D macroquad)
pub mod botpool;
use bevy::prelude::*;
use bevy::image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor};
use bevy::light::EnvironmentMapLight;
use bevy::core_pipeline::Skybox;
use bevy::asset::RenderAssetUsages;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};
use crate::core::arena::{arena_size, current_map, ARENA_WALL_H};
use crate::core::world::{Kind, World};

// ---- asset nhung san (duong dan embedded) ----
// Ban lean: chi 1 skybox 1K + 6 texture 512px de exe nho (~2.4MB asset).
pub const HDRI_SKY: &str = "embedded://assets/hdri/studio_small_03_1k.hdr";
pub const TEX_FLOOR: &str = "embedded://assets/tex/small/floor.jpg";
pub const TEX_WALL: &str = "embedded://assets/tex/small/wall.jpg";
pub const TEX_BLOCK: &str = "embedded://assets/tex/small/block.jpg";
pub const NRM_FLOOR: &str = "embedded://assets/tex/small/floor_n.jpg";
pub const NRM_WALL: &str = "embedded://assets/tex/small/wall_n.jpg";
pub const NRM_BLOCK: &str = "embedded://assets/tex/small/block_n.jpg";

// ---- model 3D that (Kenney CC0) ----
pub const BOT_GLB: &str = "embedded://assets/models/bot/character-a.glb";
pub const GUN_PISTOL: &str = "embedded://assets/models/gun/blaster-b.glb";
pub const GUN_RIFLE: &str = "embedded://assets/models/gun/blaster-d.glb";
pub const GUN_SNIPER: &str = "embedded://assets/models/gun/blaster-e.glb";
pub const GUN_SMG: &str = "embedded://assets/models/gun/blaster-j.glb";

pub fn gun_glb(kind: u8) -> &'static str {
    match kind {
        0 => GUN_PISTOL,
        1 => GUN_RIFLE,
        2 => GUN_SNIPER,
        _ => GUN_SMG,
    }
}

pub fn gun_scale(kind: u8) -> f32 {
    match kind {
        0 => 0.50,
        1 => 0.36,
        2 => 0.30,
        _ => 0.40,
    }
}

// animation graph cho bot: idle / walk / die
#[derive(Resource)]
pub struct BotAnim {
    pub graph: Handle<AnimationGraph>,
    pub idle: AnimationNodeIndex,
    pub walk: AnimationNodeIndex,
    pub die: AnimationNodeIndex,
}

// tien do gan graph + animation dang chay
#[derive(Resource)]
pub struct BotAnimPlayer {
    pub entity: Option<Entity>,
    pub current: i32,
}

impl Default for BotAnimPlayer {
    fn default() -> Self {
        Self { entity: None, current: -1 }
    }
}

// material bot da nhan ban de to mau skin + flash
#[derive(Resource, Default)]
pub struct BotVisual {
    pub ready: bool,
    pub mats: Vec<Handle<StandardMaterial>>,
}

pub fn build_bot_anim(
    graphs: &mut Assets<AnimationGraph>,
    server: &AssetServer,
) -> BotAnim {
    let clips: Vec<Handle<AnimationClip>> = [1usize, 2, 6]
        .iter()
        .map(|i| server.load(GltfAssetLabel::Animation(*i).from_asset(BOT_GLB)))
        .collect();
    let (graph, indices) = AnimationGraph::from_clips(clips);
    BotAnim {
        graph: graphs.add(graph),
        idle: indices[0],
        walk: indices[1],
        die: indices[2],
    }
}

// normal map: khong srgb + sampler lap
fn load_normal(server: &AssetServer, path: &str) -> Handle<Image> {
    let p = path.to_owned();
    server.load_with_settings(p, |s: &mut ImageLoaderSettings| {
        s.is_srgb = false;
        s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..default()
        });
    })
}

// tai texture lap (repeat) — tra ve handle de dua vao StandardMaterial
fn load_repeat(server: &AssetServer, path: &str) -> Handle<Image> {
    let p = path.to_owned();
    server.load_with_settings(p, |s: &mut ImageLoaderSettings| {
        s.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..default()
        });
    })
}

// HDRI skybox + IBL: HDR tu Poly Haven la equirectangular (D2) -> phai chuyen cubemap.
// setup chi danh mau pending; system env_convert se chuyen khi anh load xong.
// moi map dung 1 skybox khac nhau (dung het kho HDRI).
#[derive(Resource)]
pub struct PendingEnv {
    pub cam: Entity,
    pub hdr: Handle<Image>,
    pub cubemap: Handle<Image>,
    pub intensity: f32,
    pub brightness: f32,
    pub done: bool,
}

pub fn setup_environment(
    commands: &mut Commands,
    images: &mut Assets<Image>,
    cam: Entity,
    server: &AssetServer,
    high: bool,
) {
    let hdr: Handle<Image> = server.load(HDRI_SKY);
    let cubemap = images.add(Image::new_uninit(
        Extent3d { width: 1, height: 1, depth_or_array_layers: 6 },
        TextureDimension::D2,
        TextureFormat::Rgba32Float,
        RenderAssetUsages::default(),
    ));
    commands.insert_resource(PendingEnv {
        cam,
        hdr,
        cubemap,
        intensity: if high { 900.0 } else { 400.0 },
        brightness: if high { 1200.0 } else { 700.0 },
        done: false,
    });
}

// chuyen equirect RGB32F -> cubemap (6 mat), cập nhật Skybox + EnvironmentMapLight
pub fn env_convert(
    mut pending: ResMut<PendingEnv>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
) {
    if pending.done {
        return;
    }
    let Some(src) = images.get(&pending.hdr) else { return };
    if src.data.is_none() || src.texture_descriptor.size.width < 4 {
        return;
    }
    let w = src.texture_descriptor.size.width as usize;
    let h = src.texture_descriptor.size.height as usize;
    let Some(data) = src.data.as_ref() else { return };
    let face = 512usize.min(w / 4).max(64);
    let mut out: Vec<u8> = Vec::with_capacity(face * face * 6 * 16);
    let sample = |dir: Vec3| -> [f32; 3] {
        let d = dir.normalize();
        let lon = d.x.atan2(-d.z);
        let lat = d.y.clamp(-1.0, 1.0).asin();
        let u = (lon / (2.0 * std::f32::consts::PI) + 0.5).rem_euclid(1.0);
        let v = (0.5 - lat / std::f32::consts::PI).clamp(0.0, 1.0);
        let px = ((u * (w - 1) as f32) as usize).min(w - 1);
        let py = ((v * (h - 1) as f32) as usize).min(h - 1);
        let idx = (py * w + px) * 16;
        if idx + 12 <= data.len() {
            let f = |o: usize| f32::from_le_bytes([data[idx + o], data[idx + o + 1], data[idx + o + 2], data[idx + o + 3]]);
            [f(0), f(4), f(8)]
        } else {
            [0.0, 0.0, 0.0]
        }
    };
    let faces: [(Vec3, Vec3, Vec3); 6] = [
        (Vec3::X, Vec3::NEG_Z, Vec3::NEG_Y),
        (Vec3::NEG_X, Vec3::Z, Vec3::NEG_Y),
        (Vec3::Y, Vec3::X, Vec3::Z),
        (Vec3::NEG_Y, Vec3::X, Vec3::NEG_Z),
        (Vec3::Z, Vec3::X, Vec3::NEG_Y),
        (Vec3::NEG_Z, Vec3::NEG_X, Vec3::NEG_Y),
    ];
    for (fwd, right, up) in faces.iter() {
        for y in 0..face {
            for x in 0..face {
                let u = 2.0 * (x as f32 + 0.5) / face as f32 - 1.0;
                let v = 2.0 * (y as f32 + 0.5) / face as f32 - 1.0;
                let dir = *fwd + *right * u - *up * v;
                let c = sample(dir);
                out.extend_from_slice(&c[0].to_le_bytes());
                out.extend_from_slice(&c[1].to_le_bytes());
                out.extend_from_slice(&c[2].to_le_bytes());
                out.extend_from_slice(&1.0f32.to_le_bytes());
            }
        }
    }
    if let Some(dst) = images.get_mut(&pending.cubemap) {
        dst.data = Some(out);
        dst.texture_descriptor.size = Extent3d { width: face as u32, height: face as u32, depth_or_array_layers: 6 };
        dst.texture_descriptor.format = TextureFormat::Rgba32Float;
        dst.texture_view_descriptor = Some(TextureViewDescriptor {
            dimension: Some(TextureViewDimension::Cube),
            ..default()
        });
        dst.asset_usage = RenderAssetUsages::default();
    }
    let cubemap = pending.cubemap.clone();
    commands.entity(pending.cam).insert((
        EnvironmentMapLight {
            diffuse_map: cubemap.clone(),
            specular_map: cubemap.clone(),
            intensity: pending.intensity,
            ..default()
        },
        Skybox {
            image: cubemap,
            brightness: pending.brightness,
            rotation: Quat::IDENTITY,
        },
    ));
    pending.done = true;
}

#[derive(Component)]
pub struct ArenaRoot;
#[derive(Component)]
pub struct BotBody;
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
    // mau block theo tung theme dia hinh: (block, platform)
    pub themes: Vec<(Handle<StandardMaterial>, Handle<StandardMaterial>)>,
    pub strip: Handle<StandardMaterial>,
}

pub const THEME_COUNT: usize = 5;
// mau theme: (block, platform) — [r,g,b]
pub const THEME_COLS: [([f32; 3], [f32; 3]); THEME_COUNT] = [
    ([0.30, 0.32, 0.36], [0.26, 0.34, 0.40]), // xam lanh (mac dinh)
    ([0.45, 0.34, 0.24], [0.52, 0.40, 0.28]), // go
    ([0.24, 0.38, 0.30], [0.30, 0.46, 0.36]), // xanh reu
    ([0.38, 0.26, 0.40], [0.46, 0.32, 0.48]), // tim
    ([0.20, 0.34, 0.48], [0.26, 0.42, 0.58]), // xanh bien
];

const BOT_BODY_COL: Srgba = Srgba::new(0.85, 0.30, 0.28, 1.0);

// cac mau skin bot de chon
pub const BOT_SKINS: [Srgba; 6] = [
    Srgba::new(0.85, 0.30, 0.28, 1.0), // do
    Srgba::new(0.25, 0.55, 0.90, 1.0), // xanh duong
    Srgba::new(0.30, 0.80, 0.40, 1.0), // xanh la
    Srgba::new(0.90, 0.75, 0.20, 1.0), // vang
    Srgba::new(0.70, 0.30, 0.85, 1.0), // tim
    Srgba::new(0.90, 0.90, 0.92, 1.0), // trang
];

pub fn bot_skin_color(i: usize) -> Srgba {
    BOT_SKINS[i % BOT_SKINS.len()]
}

pub fn setup_materials(
    mats: &mut Assets<StandardMaterial>,
    server: &AssetServer,
) -> Mats {
    let m = |mats: &mut Assets<StandardMaterial>, c: Srgba, rough: f32, metal: f32| {
        mats.add(StandardMaterial {
            base_color: c.into(),
            perceptual_roughness: rough,
            metallic: metal,
            ..default()
        })
    };
    let tex_floor = load_repeat(server, TEX_FLOOR);
    let tex_wall = load_repeat(server, TEX_WALL);
    let tex_block = load_repeat(server, TEX_BLOCK);
    let nrm_floor = load_normal(server, NRM_FLOOR);
    let nrm_wall = load_normal(server, NRM_WALL);
    let nrm_block = load_normal(server, NRM_BLOCK);
    let floor = mats.add(StandardMaterial {
        base_color: Srgba::rgb(0.42, 0.43, 0.46).into(),
        base_color_texture: Some(tex_floor),
        normal_map_texture: Some(nrm_floor),
        perceptual_roughness: 0.85,
        metallic: 0.05,
        ..default()
    });
    let wall = mats.add(StandardMaterial {
        base_color: Srgba::rgb(0.55, 0.56, 0.58).into(),
        base_color_texture: Some(tex_wall),
        normal_map_texture: Some(nrm_wall),
        perceptual_roughness: 0.8,
        metallic: 0.1,
        ..default()
    });
    let block = mats.add(StandardMaterial {
        base_color: Srgba::rgb(0.68, 0.70, 0.74).into(),
        base_color_texture: Some(tex_block),
        normal_map_texture: Some(nrm_block),
        perceptual_roughness: 0.7,
        metallic: 0.15,
        ..default()
    });
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
    let strip = mats.add(StandardMaterial {
        base_color: Srgba::rgb(0.20, 0.70, 0.95).into(),
        emissive: LinearRgba::rgb(0.6, 2.6, 4.0),
        unlit: true,
        ..default()
    });
    let themes = THEME_COLS
        .iter()
        .map(|(b, p)| {
            (
                mats.add(StandardMaterial {
                    base_color: Srgba::rgb(b[0], b[1], b[2]).into(),
                    perceptual_roughness: 0.7,
                    metallic: 0.15,
                    ..default()
                }),
                mats.add(StandardMaterial {
                    base_color: Srgba::rgb(p[0], p[1], p[2]).into(),
                    perceptual_roughness: 0.6,
                    metallic: 0.2,
                    ..default()
                }),
            )
        })
        .collect();
    Mats { floor, wall, block, platform, bot_body, bot_head, gun, gun_accent, gun_grip, gun_lens, tracer, themes, strip }
}

pub fn spawn_arena(
    commands: &mut Commands,
    mats: &Mats,
    meshes: &mut Assets<Mesh>,
) {
    let half = arena_size();
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
    // vien san phat sang quanh tuong (trang tri, unlit -> re)
    let inset = half - 0.6;
    let strips = [
        (0.0, 0.02, -inset, inset * 2.0, 0.04, 0.10),
        (0.0, 0.02, inset, inset * 2.0, 0.04, 0.10),
        (-inset, 0.02, 0.0, 0.10, 0.04, inset * 2.0),
        (inset, 0.02, 0.0, 0.10, 0.04, inset * 2.0),
    ];
    for (x, y, z, sx, sy, sz) in strips {
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(mats.strip.clone()),
            Transform::from_xyz(x, y, z).with_scale(Vec3::new(sx, sy, sz)),
            ArenaRoot,
        ));
    }
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
    let theme = mats.themes.get(m.theme.min(mats.themes.len().saturating_sub(1)));
    let (theme_block, theme_plat) = match theme {
        Some((b, p)) => (b.clone(), p.clone()),
        None => (mats.block.clone(), mats.platform.clone()),
    };
    let blocks = commands.spawn((Transform::default(), Visibility::default(), BlocksRoot)).id();
    let push_block = |commands: &mut Commands, b: &[f32; 6]| {
        let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
        let mat = if hh <= 0.3 { theme_plat.clone() } else { theme_block.clone() };
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
        let mat = if hh <= 0.3 { theme_plat.clone() } else { theme_block.clone() };
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

pub fn spawn_bot(commands: &mut Commands, server: &AssetServer) {
    let root = commands
        .spawn((Transform::from_xyz(0.0, 0.0, -10.0), Visibility::default(), BotBody))
        .id();
    commands.spawn((
        SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset(BOT_GLB))),
        ChildOf(root),
    ));
}

// viewmodel: sung 3D that (Kenney CC0), parent vao camera
// kind: 0=Pistol 1=Rifle 2=Sniper 3=Smg
pub fn spawn_gun(
    commands: &mut Commands,
    server: &AssetServer,
    kind: u8,
    camera: Entity,
) {
    let root = commands.spawn((
        Transform::default(),
        Visibility::default(),
        GunModel,
        ChildOf(camera),
    )).id();
    commands.spawn((
        SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset(gun_glb(kind)))),
        Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::splat(gun_scale(kind))),
        ChildOf(root),
    ));
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
    server: Res<AssetServer>,
    q_old: Query<Entity, With<GunModel>>,
    q_cam: Query<Entity, With<crate::MainCamera>>,
) {
    if !dirty.0 { return; }
    dirty.0 = false;
    let Ok(cam) = q_cam.single() else { return };
    for e in q_old.iter() {
        commands.entity(e).despawn();
    }
    spawn_gun(&mut commands, &server, game.current_gun() as u8, cam);
}

// cap nhat vi tri bot + visibility theo drill hien tai
// bot glb: chan y=0, cao ~2.0 -> offset de tam hitbox trung voi y cua Target
const BOT_HEIGHT_OFF: f32 = 1.0;

pub fn sync_bot(
    world: Res<World>,
    mut q_body: Query<(&mut Transform, &mut Visibility), With<BotBody>>,
    mut anim: ResMut<BotAnimPlayer>,
    graph: Res<BotAnim>,
    mut q_players: Query<&mut AnimationPlayer>,
) {
    let display = bot_display(&world);
    match display {
        Some((x, y, z, alive)) => {
            for (mut tr, mut vis) in q_body.iter_mut() {
                tr.translation = Vec3::new(x, y - BOT_HEIGHT_OFF, z);
                *vis = Visibility::Inherited;
            }
            let want = if alive { 0 } else { 1 };
            if anim.current != want {
                if let Some(e) = anim.entity {
                    if let Ok(mut player) = q_players.get_mut(e) {
                        if alive {
                            player.play(graph.idle).repeat();
                        } else {
                            player.play(graph.die);
                        }
                    }
                }
                anim.current = want;
            }
        }
        None => {
            for (_, mut vis) in q_body.iter_mut() { *vis = Visibility::Hidden; }
        }
    }
}

// gan animation graph cho bot DON cua mode BOT DUEL.
// Chi xet player thuoc cay con cua `BotBody` (bo qua 10 bot cua 5v5).
pub fn setup_bot_anim(
    anim: Res<BotAnim>,
    mut progress: ResMut<BotAnimPlayer>,
    mut commands: Commands,
    mut q_added: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parents: Query<&ChildOf>,
    bots: Query<(), With<BotBody>>,
) {
    if progress.entity.is_some() {
        return;
    }
    for (e, mut player) in q_added.iter_mut() {
        let mut p = e;
        let mut under_bot = false;
        for _ in 0..8 {
            let Ok(parent) = parents.get(p) else { break };
            if bots.contains(parent.0) {
                under_bot = true;
                break;
            }
            p = parent.0;
        }
        if !under_bot {
            continue;
        }
        commands.entity(e).insert(AnimationGraphHandle(anim.graph.clone()));
        player.play(anim.idle).repeat();
        progress.entity = Some(e);
        progress.current = 0;
        break;
    }
}

// lay material cua bot tu scene gltf de to mau skin + flash trang khi trung dan
pub fn sync_bot_color(
    world: Res<World>,
    game: Res<crate::game::Game>,
    mut visual: ResMut<BotVisual>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    q_bot: Query<Entity, With<BotBody>>,
    q_children: Query<&Children>,
    q_mat: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    if !visual.ready {
        for bot in q_bot.iter() {
            for desc in q_children.iter_descendants(bot) {
                if let Ok(mm) = q_mat.get(desc) {
                    visual.mats.push(mm.0.clone());
                }
            }
            if !visual.mats.is_empty() {
                visual.ready = true;
            }
        }
    }
    if visual.ready {
        let flash = world.bot_hit();
        let c = if flash { Srgba::new(1.0, 1.0, 1.0, 1.0) } else { bot_skin_color(game.bot_skin) };
        for h in visual.mats.iter() {
            if let Some(m) = materials.get_mut(h) {
                m.base_color = c.into();
            }
        }
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
        let mat = if t.kind == Kind::Track { mats.bot_body.clone() } else { mats.tracer.clone() };
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

/// Ban lean: chi dung 1 bo texture 512px (san / tuong / vat) + 1 skybox 1K.

#[cfg(test)]
mod tests {
    use super::*;

    /// Ban lean chi dung 1 bo texture 512px + 1 skybox 1K.
    /// Test nay chan khi ai do them lai asset lon vao exe.
    #[test]
    fn all_texture_and_sky_assets_are_small() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let mut total = 0u64;
        let mut count = 0;
        for e in walkdir(&root) {
            let len = std::fs::metadata(&e).unwrap().len();
            // khong asset nao duoc vuot 2 MB (tru file goc bi bo qua)
            assert!(len <= 2 * 1024 * 1024, "asset qua lon ({} KB): {}", len / 1024, e.display());
            total += len;
            count += 1;
        }
        assert!(count > 0, "khong tim thay asset nao");
        assert!(
            total <= 4 * 1024 * 1024,
            "tong asset phai <= 4 MB, hien tai {:.2} MB",
            total as f64 / 1048576.0
        );
    }

    #[test]
    fn texture_and_sky_paths_exist() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        for rel in [
            "hdri/studio_small_03_1k.hdr",
            "tex/small/floor.jpg",
            "tex/small/wall.jpg",
            "tex/small/block.jpg",
            "tex/small/floor_n.jpg",
            "tex/small/wall_n.jpg",
            "tex/small/block_n.jpg",
        ] {
            assert!(root.join(rel).exists(), "thieu asset: {}", rel);
        }
    }

    fn walkdir(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut out = Vec::new();
        let Ok(rd) = std::fs::read_dir(dir) else { return out };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir(&p));
            } else {
                out.push(p);
            }
        }
        out
    }
}
