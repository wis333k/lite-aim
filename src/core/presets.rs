// presets (PORT NGUYEN)

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GunKind {
    Pistol,
    Rifle,
    Sniper,
    Smg,
}

pub struct Preset {
    pub name: &'static str,
    pub yaw: f32,
    pub default_sens: f32,
    pub default_dpi: f32,
    pub fov: f32,
    pub crosshair: usize,
    pub recoil_mag: usize,
    pub recoil_spread: f32,
    pub gun: GunKind,
}

pub const PRESETS: [Preset; 4] = [
    Preset { name: "Valorant",     yaw: 0.07,   default_sens: 0.4, default_dpi: 800.0,  fov: 103.0, crosshair: 2, recoil_mag: 25, recoil_spread: 0.55, gun: GunKind::Pistol },
    Preset { name: "CS2",          yaw: 0.022,  default_sens: 1.1, default_dpi: 800.0,  fov: 90.0,  crosshair: 1, recoil_mag: 30, recoil_spread: 1.00, gun: GunKind::Rifle },
    Preset { name: "Apex Legends", yaw: 0.022,  default_sens: 1.6, default_dpi: 1600.0, fov: 110.0, crosshair: 0, recoil_mag: 28, recoil_spread: 0.70, gun: GunKind::Smg },
    Preset { name: "Overwatch 2",  yaw: 0.0066, default_sens: 6.0, default_dpi: 1600.0, fov: 103.0, crosshair: 0, recoil_mag: 30, recoil_spread: 0.50, gun: GunKind::Sniper },
];

pub fn cm360(yaw: f32, sens: f32, dpi: f32) -> f32 {
    (360.0 / (yaw * sens * dpi)) * 2.54
}
