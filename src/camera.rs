// camera: dong bo Camera3d tu World.pose + FPS mouse-look qua CursorMoved (winit Locked)
use crate::core::player::apply_mouse;
use crate::core::world::World;
use crate::game::Game;
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

pub fn sync_camera(
    world: Res<World>,
    mut cam: Query<(&mut Transform, &mut Projection), With<Camera3d>>,
) {
    let Ok((mut tr, mut proj)) = cam.single_mut() else { return };
    let p = &world.pose;
    tr.translation = Vec3::new(p.pos[0], p.pos[1], p.pos[2]);
    let yaw = p.yaw.to_radians();
    let pitch = p.pitch.to_radians();
    // game forward(): (-cp*sy, sp, -cp*cy) voi yaw quanh Y, pitch ngua len
    let q = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    tr.rotation = q;

    if let Projection::Perspective(persp) = &mut *proj {
        persp.fov = world.eff_fov().to_radians();
    }
}

// mouse-look: doc delta tu MouseMotion (winit Locked grab) - thay cho SetCursorPos loop
pub fn mouse_look(world: &mut World, game: &Game, motion: &mut MessageReader<MouseMotion>) {
    if !game.grab_active {
        return;
    }
    let mut dx = 0.0f32;
    let mut dy = 0.0f32;
    for ev in motion.read() {
        dx += ev.delta.x;
        dy += ev.delta.y;
    }
    if dx != 0.0 || dy != 0.0 {
        apply_mouse(&mut world.pose, dx, dy, game.deg_per_count(), game.invert_y);
    }
}

// shake camera: cong thêm lech vao transform sau sync_camera
pub fn apply_shake(
    world: Res<World>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(mut tr) = cam.single_mut() else { return };
    if world.shake > 0.001 {
        let t = world.tick_time * 0.06;
        tr.translation.x += t.sin() * world.shake * 0.02;
        tr.translation.y += (t * 1.3).cos() * world.shake * 0.02;
    }
    // kick giat (flash) nhe ve truoc
    if world.flash > 0.0 {
        tr.translation.y -= world.flash * 0.05;
    }
}
