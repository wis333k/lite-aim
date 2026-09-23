// fx: tracer, wall hits, particles — Bevy 3D (dung FxMeshes cache)
use bevy::prelude::*;
use crate::core::world::World;
use crate::render::{FxMeshes, Mats};

#[derive(Component)]
pub struct TracerFx;

#[derive(Component)]
pub struct WallHitFx;

#[derive(Component)]
pub struct ParticleFx;

// pool tracer: spawn lai moi frame tu world.tracers
pub fn sync_tracers(
    mut commands: Commands,
    world: Res<World>,
    mats: Res<Mats>,
    fx_meshes: Res<FxMeshes>,
    q: Query<Entity, With<TracerFx>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
    for tr in world.tracers.iter() {
        let a = Vec3::new(tr[0], tr[1], tr[2]);
        let b = Vec3::new(tr[3], tr[4], tr[5]);
        let d = b - a;
        let len = d.length();
        if len < 1e-4 {
            continue;
        }
        let mid = (a + b) * 0.5;
        let rot = Quat::from_rotation_arc(Vec3::NEG_Z, d.normalize());
        commands.spawn((
            Mesh3d(fx_meshes.tracer.clone()),
            MeshMaterial3d(mats.tracer.clone()),
            Transform::from_translation(mid).with_rotation(rot).with_scale(Vec3::new(1.0, 1.0, len)),
            TracerFx,
        ));
    }
}

// wall hits (vach ban recoil)
pub fn sync_wall_hits(
    mut commands: Commands,
    world: Res<World>,
    mats: Res<Mats>,
    fx_meshes: Res<FxMeshes>,
    q: Query<Entity, With<WallHitFx>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
    for wh in world.wall_hits.iter() {
        let x = wh[0];
        let y = wh[1];
        commands.spawn((
            Mesh3d(fx_meshes.sphere.clone()),
            MeshMaterial3d(mats.tracer.clone()),
            Transform::from_xyz(x * 0.6, y, -20.0).with_scale(Vec3::splat(0.05)),
            WallHitFx,
        ));
    }
}

pub fn sync_particles(
    mut commands: Commands,
    world: Res<World>,
    mats: Res<Mats>,
    fx_meshes: Res<FxMeshes>,
) {
    for p in world.parts.iter() {
        commands.spawn((
            Mesh3d(fx_meshes.particle.clone()),
            MeshMaterial3d(mats.tracer.clone()),
            Transform::from_xyz(p[0], p[1], p[2]),
            ParticleFx,
        ));
    }
}

pub fn clear_particles(mut commands: Commands, q: Query<Entity, With<ParticleFx>>) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
