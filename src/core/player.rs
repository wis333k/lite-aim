// core player: pose + mouse look + step (PORT NGUYEN)

use super::arena::{EYE_CROUCH, EYE_STAND};
use super::math;

#[derive(Clone, Copy)]
pub struct Pose {
    pub yaw: f32,
    pub pitch: f32,
    pub pos: [f32; 3], // eye position
    pub feet: f32,
    pub vy: f32,
    pub eye_h: f32,
    pub bob: f32,
    pub pmove: bool,
    pub psprint: bool,
    pub pcrouch: bool,
}

impl Default for Pose {
    fn default() -> Self {
        Pose {
            yaw: 0.0,
            pitch: 0.0,
            pos: [0.0, EYE_STAND, 0.0],
            feet: 0.0,
            vy: 0.0,
            eye_h: EYE_STAND,
            bob: 0.0,
            pmove: false,
            psprint: false,
            pcrouch: false,
        }
    }
}

// mouse phai -> quay phai (system verified)
pub fn apply_mouse(p: &mut Pose, dx: f32, dy: f32, deg_per_count: f32, invert_y: bool) {
    p.yaw -= dx * deg_per_count;
    if invert_y {
        p.pitch += dy * deg_per_count;
    } else {
        p.pitch -= dy * deg_per_count;
    }
    if p.pitch > 89.0 { p.pitch = 89.0; }
    if p.pitch < -89.0 { p.pitch = -89.0; }
    if p.yaw > 180.0 { p.yaw -= 360.0; }
    if p.yaw < -180.0 { p.yaw += 360.0; }
}

// 1 step di chuyen FPS: wish XZ world-space (da chuan hoa), speed m/s
pub fn step_player(p: &mut Pose, dt: f32, wx: f32, wz: f32, speed: f32, crouch: bool, jump: bool) {
    let target_eye = if crouch { EYE_CROUCH } else { EYE_STAND };
    p.eye_h += (target_eye - p.eye_h) * (1.0 - (-dt * 10.0).exp());
    let nx0 = p.pos[0] + wx * speed * dt;
    let nz0 = p.pos[2] + wz * speed * dt;
    let (nx, nz) = math::collide(nx0, nz0, p.feet);
    p.pos[0] = nx;
    p.pos[2] = nz;
    let g = math::ground_height(nx, nz, p.feet);
    if jump && p.feet <= g + 0.05 {
        p.vy = 5.2;
    }
    p.vy -= 13.0 * dt;
    p.feet += p.vy * dt;
    if p.feet <= g {
        p.feet = g;
        p.vy = 0.0;
    }
    p.pos[1] = p.feet + p.eye_h;
    p.pmove = wx != 0.0 || wz != 0.0;
    p.psprint = speed > 5.5 && p.pmove;
    p.pcrouch = crouch;
    if p.pmove && p.feet <= g + 0.05 {
        p.bob += dt * speed * 1.6;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_right_turns_yaw_negative() {
        let mut p = Pose::default();
        apply_mouse(&mut p, 10.0, 0.0, 0.1, false);
        assert!((p.yaw + 1.0).abs() < 1e-4);
    }

    #[test]
    fn pitch_clamped() {
        let mut p = Pose::default();
        apply_mouse(&mut p, 0.0, 100000.0, 0.1, false);
        assert!(p.pitch <= 89.0 && p.pitch >= -89.0);
    }

    #[test]
    fn eye_follows_feet() {
        let mut p = Pose::default();
        step_player(&mut p, 0.016, 0.0, 0.0, 4.0, false, false);
        assert!((p.pos[1] - (p.feet + p.eye_h)).abs() < 1e-4);
    }
}
