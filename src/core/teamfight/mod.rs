// Mode 5v5 (team deathmatch) — tach rieng khoi `core::drills` de khong
// pha 5 mode cu. Cac module:
//   actor.rs — trang thai chien dau cua 1 nguoi (bot hoac nguoi choi)
//   nav.rs   — luoi di chuyen + A* de bot tim duong quanh vat can
//   ai.rs    — hanh vi AI 3 muc do kho
//   round.rs — vong, doi, bang diem, thay doi doi giua hiem
//   fight.rs — driver moi frame (player pose, AI, giai quet ban, snapshot)
//   hud.rs   — kieu du lieu ma World giu de render + HUD doc
pub mod actor;
pub mod ai;
pub mod bomb;
pub mod fight;
pub mod hud;
pub mod nav;
pub mod round;
