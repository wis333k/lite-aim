// editor: map editor 3D — dat/xoa khoi, luu map rieng (maps.json)
use crate::core::arena::{current_map, save_custom_map, total_maps};
use crate::core::math;
use crate::core::world::World;
use bevy::ecs::message::MessageReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

// cac co khoi co san (hw, hh, hd)
const SIZES: [[f32; 3]; 4] = [
    [0.5, 0.5, 0.5],
    [1.0, 0.5, 1.0],
    [0.75, 1.5, 0.75],
    [2.0, 0.25, 2.0],
];

#[derive(Resource)]
pub struct Editor {
    pub active: bool,
    pub blocks: Vec<[f32; 6]>,
    pub size: usize,
    pub target: Option<[f32; 3]>,
    pub msg: f32,
    pub msg_txt: String,
    // bay tu do
    pub yaw: f32,
    pub pitch: f32,
    pub pos: Vec3,
    pub theme: usize,
}

impl Default for Editor {
    fn default() -> Self {
        Editor {
            active: false,
            blocks: Vec::new(),
            size: 0,
            target: None,
            msg: 0.0,
            msg_txt: String::new(),
            yaw: 0.0,
            pitch: 0.0,
            pos: Vec3::new(0.0, 4.0, 8.0),
            theme: 0,
        }
    }
}

#[derive(Component)]
pub struct EditorPreview;

#[derive(Component)]
pub struct EditorCam;

// snapshot block de ve: tat ca block dang co trong editor + preview
#[derive(Resource, Default)]
pub struct EditorDirty(pub bool);

impl Editor {
    pub fn begin(&mut self) {
        self.active = true;
        // bat dau tu map dang chon: copy cac block co san (chi block, khong extras/props)
        self.blocks = current_map().blocks.to_vec();
        self.pos = Vec3::new(0.0, 3.0, 10.0);
        self.yaw = 180.0;
        self.pitch = -10.0;
        self.target = None;
        self.msg = 0.0;
        self.theme = crate::core::arena::active_map_theme();
    }

    pub fn cur_size(&self) -> [f32; 3] {
        SIZES[self.size % SIZES.len()]
    }

    // ray tu editor cam theo goc nhin, tim block gan nhat -> diem dat + toa do
    pub fn pick(&self) -> Option<([f32; 3], [f32; 3])> {
        let (dx, dy, dz) = math::forward(self.yaw, self.pitch);
        let (fx, fy, fz) = (self.pos.x, self.pos.y, self.pos.z);
        let mut best_t = 1e9f32;
        for b in self.blocks.iter() {
            let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
            if let Some(t) = math::ray_aabb(
                fx, fy, fz, dx, dy, dz,
                bx - hw, by - hh, bz - hd, bx + hw, by + hh, bz + hd,
            ) {
                if t < best_t {
                    best_t = t;
                }
            }
        }
        // giao voi mat dat y=0
        if dy.abs() > 1e-6 {
            let t = -fy / dy;
            if t > 0.0 && t < best_t {
                best_t = t;
            }
        }
        if best_t >= 1e9 {
            return None;
        }
        let hit = [fx + dx * best_t, fy + dy * best_t, fz + dz * best_t];
        // snap 0.5
        let snap = |v: f32| (v * 2.0).round() / 2.0;
        Some(([snap(hit[0]), snap(hit[1]), snap(hit[2])], hit))
    }

    // diem dat block: day theo phap tuyen gan dung (mat block huong ve camera)
    pub fn place_pos(&self, size: &[f32; 3]) -> Option<[f32; 3]> {
        let (p, hit) = self.pick()?;
        let dir = Vec3::new(hit[0] - self.pos.x, hit[1] - self.pos.y, hit[2] - self.pos.z).normalize();
        // day ra khoi be mat nua o
        let off = if dir.x.abs() > dir.y.abs() && dir.x.abs() > dir.z.abs() {
            [dir.x.signum() * size[0], 0.0, 0.0]
        } else if dir.y.abs() > dir.z.abs() {
            [0.0, dir.y.signum() * size[1], 0.0]
        } else {
            [0.0, 0.0, dir.z.signum() * size[2]]
        };
        Some([p[0] + off[0], p[1].max(size[1]) + off[1], p[2] + off[2]])
    }
}

// input editor: bay tu do + dat/xoa + doi size + luu
pub fn editor_input(
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut motion: MessageReader<MouseMotion>,
    time: Res<Time>,
    mut ed: ResMut<Editor>,
    mut dirty: ResMut<EditorDirty>,
    mut next: ResMut<NextState<crate::game::Screen>>,
    mut ui_res: ResMut<crate::ui::UiRes>,
    mut windows: Query<&mut bevy::window::CursorOptions>,
) {
    if !ed.active {
        return;
    }
    let dt = time.delta_secs();

    // bat grab chuot khi vao editor
    if let Ok(mut c) = windows.single_mut() {
        if c.grab_mode != bevy::window::CursorGrabMode::Locked {
            c.visible = false;
            c.grab_mode = bevy::window::CursorGrabMode::Locked;
        }
    }

    if keys.just_pressed(KeyCode::Escape) {
        ed.active = false;
        if let Ok(mut c) = windows.single_mut() {
            c.visible = true;
            c.grab_mode = bevy::window::CursorGrabMode::None;
        }
        next.set(crate::game::Screen::Menu);
        ui_res.dirty = true;
        return;
    }

    // mouse look
    let mut dx = 0.0;
    let mut dy = 0.0;
    for ev in motion.read() {
        dx += ev.delta.x;
        dy += ev.delta.y;
    }
    ed.yaw -= dx * 0.12;
    ed.pitch -= dy * 0.12;
    ed.pitch = ed.pitch.clamp(-89.0, 89.0);
    if ed.yaw > 180.0 { ed.yaw -= 360.0; }
    if ed.yaw < -180.0 { ed.yaw += 360.0; }

    // bay tu do
    let (fw_x, fw_y, fw_z) = math::forward(ed.yaw, ed.pitch);
    let (rt_x, _, rt_z) = math::forward(ed.yaw - 90.0, 0.0);
    let speed = if keys.pressed(KeyCode::ShiftLeft) { 18.0 } else { 8.0 };
    let mut mv = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) { mv += Vec3::new(fw_x, fw_y, fw_z); }
    if keys.pressed(KeyCode::KeyS) { mv -= Vec3::new(fw_x, fw_y, fw_z); }
    if keys.pressed(KeyCode::KeyD) { mv += Vec3::new(rt_x, 0.0, rt_z); }
    if keys.pressed(KeyCode::KeyA) { mv -= Vec3::new(rt_x, 0.0, rt_z); }
    if keys.pressed(KeyCode::KeyE) { mv += Vec3::Y; }
    if keys.pressed(KeyCode::KeyQ) { mv -= Vec3::Y; }
    if mv.length_squared() > 0.0 {
        ed.pos += mv.normalize() * speed * dt;
        ed.pos.y = ed.pos.y.max(0.2);
    }

    // doi size bang [ / ]
    if keys.just_pressed(KeyCode::BracketRight) {
        ed.size = (ed.size + 1) % SIZES.len();
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        ed.size = (ed.size + SIZES.len() - 1) % SIZES.len();
    }
    // doi mau dia hinh bang T
    if keys.just_pressed(KeyCode::KeyT) {
        ed.theme = (ed.theme + 1) % crate::render::THEME_COUNT;
    }

    // cap nhat diem ngam
    ed.target = ed.place_pos(&ed.cur_size());
    dirty.0 = true;

    // chuot trai: dat block
    if mouse.just_pressed(MouseButton::Left) {
        if let Some(p) = ed.place_pos(&ed.cur_size()) {
            let s = ed.cur_size();
            ed.blocks.push([p[0], p[1], p[2], s[0], s[1], s[2]]);
        }
    }
    // chuot phai: xoa block gan nhat theo tia
    if mouse.just_pressed(MouseButton::Right) {
        if let Some((_, hit)) = ed.pick() {
            let (dxf, dyf, dzf) = math::forward(ed.yaw, ed.pitch);
            let (_fx, _fy, _fz) = (ed.pos.x, ed.pos.y, ed.pos.z);
            let mut best_t = 1e9f32;
            let mut best_i = None;
            for (i, b) in ed.blocks.iter().enumerate() {
                let (bx, by, bz, hw, hh, hd) = (b[0], b[1], b[2], b[3], b[4], b[5]);
                if let Some(t) = math::ray_aabb(
                    ed.pos.x, ed.pos.y, ed.pos.z, dxf, dyf, dzf,
                    bx - hw, by - hh, bz - hd, bx + hw, by + hh, bz + hd,
                ) {
                    if t < best_t {
                        best_t = t;
                        best_i = Some(i);
                    }
                }
            }
            let _ = hit;
            if let Some(i) = best_i {
                ed.blocks.remove(i);
            }
        }
    }
    // F: luu map
    if keys.just_pressed(KeyCode::KeyF) {
        let name = format!("CUSTOM {}", total_maps().saturating_sub(4));
        if save_custom_map(&name, &ed.blocks, ed.theme) {
            ed.msg_txt = format!("SAVED: {}", name);
        } else {
            ed.msg_txt = "SAVE FAILED".to_owned();
        }
        ed.msg = 2.5;
    }
    if ed.msg > 0.0 { ed.msg -= dt; }
}

// dong bo editor camera + ve preview block
pub fn sync_editor(
    mut commands: Commands,
    ed: Res<Editor>,
    mut dirty: ResMut<EditorDirty>,
    mut q_cam: Query<&mut Transform, (With<Camera3d>, Without<EditorPreview>)>,
    q_prev: Query<Entity, With<EditorPreview>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut mats: ResMut<Assets<StandardMaterial>>,
    q_mats: Option<Res<crate::render::Mats>>,
    mut prev_mat: Local<Option<Handle<StandardMaterial>>>,
    mut prev_mesh: Local<Option<Handle<Mesh>>>,
) {
    let Ok(mut cam) = q_cam.single_mut() else { return };
    if ed.active {
        cam.translation = ed.pos;
        let yaw = ed.yaw.to_radians();
        let pitch = ed.pitch.to_radians();
        cam.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.0);
    }
    if !dirty.0 {
        return;
    }
    dirty.0 = false;
    for e in q_prev.iter() {
        commands.entity(e).despawn();
    }
    if !ed.active {
        return;
    }
    let mesh = prev_mesh.get_or_insert_with(|| meshes.add(Cuboid::new(1.0, 1.0, 1.0))).clone();
    let mat = prev_mat
        .get_or_insert_with(|| {
            mats.add(StandardMaterial {
                base_color: Color::srgba(0.20, 0.85, 1.0, 0.35),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })
        })
        .clone();
    let _ = q_mats;
    // ve cac block dang edit (highlight theo theme)
    let tc = crate::render::THEME_COLS[ed.theme % crate::render::THEME_COUNT].0;
    let block_mat = mats.add(StandardMaterial {
        base_color: Color::srgb(tc[0] * 1.25, tc[1] * 1.25, tc[2] * 1.25),
        perceptual_roughness: 0.7,
        ..default()
    });
    for b in ed.blocks.iter() {
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(block_mat.clone()),
            Transform::from_xyz(b[0], b[1], b[2])
                .with_scale(Vec3::new(b[3] * 2.0, b[4] * 2.0, b[5] * 2.0)),
            EditorPreview,
        ));
    }
    // preview vi tri dat
    if let Some(p) = ed.target {
        let s = ed.cur_size();
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(mat),
            Transform::from_xyz(p[0], p[1], p[2]).with_scale(Vec3::new(s[0] * 2.0, s[1] * 2.0, s[2] * 2.0)),
            EditorPreview,
        ));
    }
}

// ap dung map dang edit vao collision tam thoi (khong luu) khi choi? -> khong, chi luu file
