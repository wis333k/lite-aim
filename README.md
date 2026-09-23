# LITE-AIM

FPS aim trainer viết bằng Rust + [Bevy](https://bevyengine.org/) (0.18) với render 3D PBR.
Port từ một prototype macroquad, tối ưu cho máy yếu (Intel iGPU chạy được).

## Tính năng

- **5 chế độ luyện**: Gridshot, Flick, Tracking, Recoil, Bot Duel.
- **4 game preset** (Valorant / CS2 / Apex / Overwatch) kèm quy đổi cm/360.
- **4 loại súng** (Pistol, Rifle, Sniper, SMG) với ADS + fire-rate riêng, hoặc để chế độ auto.
- **Headshot**: bắn trúng đầu gây sát thương gấp đôi (bot duel: chết ngay).
- **5 map preset** + **map editor** trong game: bay tự do, đặt/xóa khối, lưu map riêng vào `maps.json`.
- **Bảng xếp hạng** lưu top 100 kết quả.
- Song ngữ Việt / Anh.
- Chất lượng đồ họa LOW/HIGH cho máy yếu.

## Điều khiển

| Phím | Chức năng |
|------|-----------|
| `W A S D` | Di chuyển |
| `Shift` | Chạy nhanh |
| `Ctrl` | Ngồi |
| `Space` | Nhảy |
| Chuột | Ngắm |
| Chuột trái | Bắn |
| Chuột phải | ADS (ngắm scope) |
| `R` | Nạp đạn |
| `Esc` | Tạm dừng |
| `1` | Đổi súng |

### Trong editor

| Phím | Chức năng |
|------|-----------|
| `WASD` | Bay ngang |
| `Space` / `E` | Lên |
| `Q` | Xuống |
| `Shift` | Bay nhanh |
| Chuột trái | Đặt khối |
| Chuột phải | Xóa khối |
| `[` / `]` | Đổi kích thước khối |
| `F` | Lưu map mới vào `maps.json` |
| `Esc` | Thoát editor |

## Build

```sh
cargo build --release
```

Binary nằm ở `target/release/aimcoach.exe`. File cấu hình (`aimcoach_data.json`) và map
tùy chỉnh (`maps.json`) được lưu cạnh binary.

## Yêu cầu

- Rust stable (đã test với 1.97).
- GPU hỗ trợ Vulkan 1.2 hoặc DirectX 11/12.

## License

MIT — xem [LICENSE](LICENSE).
