// BotPool: 10 bot GLTF cho mode 5v5.
//
// Khac voi bot don cua mode BOT DUEL (mod.rs), day la POOL co dinh 10 bot:
// moi bot 1 SceneInstance rieng, 1 bo material rieng (de to mau doi), 1
// AnimationPlayer rieng. He thong doc trang thai tu `Match` trong
// `core::teamfight::round` va day xuong transform/animation.
//
// Moi bot duoc gan `BotUnit { slot }`. `sync_botpool` khop slot voi
// `Actor.id` va: dua transform, an/hien theo alive, to mau theo doi.

use bevy::prelude::*;

use crate::core::teamfight::actor::{Actor, ROSTER_SIZE, TEAM_CT, TEAM_T};
use crate::core::world::World;

/// Mau doi: CT xanh, T cam. CHON NHAT de texture cua model van noi ro
// (nhan manh voi mau dam se lam bot thanh khoi mau phang).
pub const TEAM_COL_CT: Srgba = Srgba::new(0.55, 0.75, 1.00, 1.0);
pub const TEAM_COL_T: Srgba = Srgba::new(1.00, 0.72, 0.45, 1.0);

/// mot bot trong pool
#[derive(Component)]
pub struct BotUnit {
    /// khop voi `Actor.id`
    pub slot: usize,
    /// material da nhan ban de to mau (filled khi scene instance ready)
    pub mats: Vec<Handle<StandardMaterial>>,
    /// trang thai animation hien tai: 0=idle 1=walk 2=die
    pub anim: i32,
    /// player da xuat hien instance chua
    pub ready: bool,
}

#[derive(Component)]
pub struct BotPoolRoot;

/// Tai 10 SceneRoot (moi bot 1 glb). Chi chay 1 lan.
pub fn spawn_botpool(commands: &mut Commands, server: &AssetServer) {
    let root = commands
        .spawn((Transform::default(), Visibility::default(), BotPoolRoot))
        .id();
    for slot in 0..ROSTER_SIZE {
        let e = commands
            .spawn((
                Transform::default(),
                Visibility::default(),
                BotUnit {
                    slot,
                    mats: Vec::new(),
                    anim: -1,
                    ready: false,
                },
                ChildOf(root),
            ))
            .id();
        // SceneRoot la con cua BotUnit de transform/animation cua unit ap dung
        commands.spawn((
            SceneRoot(server.load(GltfAssetLabel::Scene(0).from_asset(
                crate::render::BOT_GLB,
            ))),
            ChildOf(e),
        ));
    }
}

/// Lan dau thay SceneInstance, clone material de to mau doi rieng cho tung bot.
pub fn setup_botpool_mat(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut units: Query<(Entity, &mut BotUnit), (Without<BotPoolReady>, With<BotUnit>)>,
    children: Query<&Children>,
    mats_q: Query<&MeshMaterial3d<StandardMaterial>>,
    parents: Query<&ChildOf>,
) {
    for (e, mut unit) in units.iter_mut() {
        if unit.ready {
            continue;
        }
        // scene instance da san do con SceneRoot
        let mut found = Vec::new();
        for desc in children.iter_descendants(e) {
            if let Ok(mm) = mats_q.get(desc) {
                found.push(mm.0.clone());
            }
        }
        if found.is_empty() {
            continue; // chua load xong, thu lai frame sau
        }
        // clone material de moi bot co mau rieng
        let mut clones = Vec::new();
        for h in found.iter() {
            if let Some(src) = materials.get(h) {
                let c = src.clone();
                let nh = materials.add(c);
                // gan material moi cho entity do
                for desc in children.iter_descendants(e) {
                    if let Ok(mm) = mats_q.get(desc) {
                        if mm.0 == *h {
                            commands.entity(desc).insert(MeshMaterial3d(nh.clone()));
                        }
                    }
                }
                clones.push(nh);
            }
        }
        unit.mats = clones;
        unit.ready = true;
        commands.entity(e).insert(BotPoolReady);
    }
    let _ = parents;
}

#[derive(Component)]
pub struct BotPoolReady;

/// Cap nhat transform + an cua 10 bot tu snapshot `World.bots`.
/// Tien do:
///  1. lay `BotView` theo slot
///  2. dua transform vao vi tri (chan y=0) + xoay theo yaw/pitch
///  3. an bot khi chet / hien khi song
///  4. to mau theo doi, xac khi chet
pub fn sync_botpool(
    world: Res<World>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut units: Query<(&mut BotUnit, &mut Transform, &mut Visibility)>,
) {
    if world.bots.is_empty() {
        for (_, _, mut vis) in units.iter_mut() {
            *vis = Visibility::Hidden;
        }
        return;
    }
    for (unit, mut tr, mut vis) in units.iter_mut() {
        let Some(view) = world.bots.iter().find(|b| b.slot == unit.slot) else {
            *vis = Visibility::Hidden;
            continue;
        };
        tr.translation = Vec3::new(view.pos[0], view.pos[1], view.pos[2]);
        tr.rotation = Quat::from_euler(EulerRot::YXZ, view.yaw, view.pitch, 0.0);
        *vis = if view.alive {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if !unit.mats.is_empty() {
            let c = if view.alive {
                team_color(view.team)
            } else {
                Srgba::new(0.5, 0.5, 0.55, 1.0)
            };
            for h in unit.mats.iter() {
                if let Some(m) = materials.get_mut(h) {
                    m.base_color = c.into();
                }
            }
        }
    }
}

fn team_color(team: u8) -> Srgba {
    if team == TEAM_CT {
        TEAM_COL_CT
    } else {
        TEAM_COL_T
    }
}

/// Gan animation graph cho ANIMATION PLAYER thuoc ve bot trong pool.
/// Chi xet cac player nam duoi cay con cua `BotUnit` (bo qua bot don cua
/// mode BOT DUEL).
pub fn setup_botpool_anim(
    anim: Res<crate::render::BotAnim>,
    mut commands: Commands,
    mut q_added: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
    parents: Query<&ChildOf>,
    units: Query<(), With<BotUnit>>,
) {
    for (e, mut player) in q_added.iter_mut() {
        // chi gan cho player co ancestor la BotUnit
        let mut p = e;
        let mut under_unit = false;
        for _ in 0..8 {
            let Ok(parent) = parents.get(p) else { break };
            if units.contains(parent.0) {
                under_unit = true;
                break;
            }
            p = parent.0;
        }
        if !under_unit {
            continue;
        }
        commands.entity(e).insert(AnimationGraphHandle(anim.graph.clone()));
        player.play(anim.idle).repeat();
    }
}

pub fn sync_botpool_anim(
    world: Res<World>,
    anim: Res<crate::render::BotAnim>,
    mut units: Query<(&mut BotUnit, Entity)>,
    mut players: Query<&mut AnimationPlayer>,
    children: Query<&Children>,
) {
    if world.bots.is_empty() {
        return;
    }
    for (mut unit, e) in units.iter_mut() {
        let Some(view) = world.bots.iter().find(|b| b.slot == unit.slot) else {
            continue;
        };
        let want = if !view.alive {
            2
        } else if view.moving {
            1
        } else {
            0
        };
        if unit.anim == want {
            continue;
        }
        unit.anim = want;
        for desc in children.iter_descendants(e) {
            if let Ok(mut player) = players.get_mut(desc) {
                let node = match want {
                    1 => anim.walk,
                    2 => anim.die,
                    _ => anim.idle,
                };
                if want == 2 {
                    player.play(node);
                } else {
                    player.play(node).repeat();
                }
            }
        }
    }
}

/// An pool khi khong phai mode 5v5 (snapshot rong).
pub fn hide_when_no_match(world: Res<World>, mut root: Query<&mut Visibility, With<BotPoolRoot>>) {
    let show = !world.bots.is_empty();
    for mut vis in root.iter_mut() {
        *vis = if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

// giu de trong file nay dung type Actor
const _: Option<fn(&Actor)> = None;
const _: u8 = TEAM_T;
