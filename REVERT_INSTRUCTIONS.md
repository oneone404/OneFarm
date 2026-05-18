# Hướng dẫn Khôi phục Giao diện Tauri Ổn định (Revert Guide)

Tài liệu này cung cấp các lệnh Git chính xác để khôi phục toàn bộ mã nguồn về phiên bản giao diện Tauri ổn định hiện tại (nhánh `backup/tauri-dialogue-bypass`) trong trường hợp quá trình thử nghiệm giao diện siêu nhẹ mới (egui/Slint) gặp sự cố hoặc hoạt động không như mong muốn.

---

## Cách 1: Chuyển hẳn sang làm việc trên nhánh Sao lưu
Nếu bạn chỉ muốn chuyển nhánh làm việc để xem lại mã nguồn hoặc tiếp tục phát triển trên giao diện Tauri cũ:

1. Cập nhật danh sách nhánh từ GitHub:
   ```bash
   git fetch origin
   ```

2. Chuyển sang nhánh sao lưu:
   ```bash
   git checkout backup/tauri-dialogue-bypass
   ```

---

## Cách 2: Khôi phục đè hoàn toàn nhánh main (Khuyên Dùng)
Nếu bạn đã thử nghiệm giao diện mới trên nhánh `main` và muốn xóa bỏ toàn bộ mã nguồn thử nghiệm đó để đưa `main` quay trở về bản giao diện Tauri ổn định này:

> [!WARNING]
> Lệnh này sẽ xóa toàn bộ các thay đổi chưa commit và các thay đổi thử nghiệm mới trên nhánh hiện tại của bạn để đồng bộ hoàn toàn với bản backup. Hãy chắc chắn bạn muốn khôi phục.

1. Chuyển về nhánh `main` (nếu đang ở nhánh khác):
   ```bash
   git checkout main
   ```

2. Cập nhật danh sách từ GitHub:
   ```bash
   git fetch origin
   ```

3. Reset cứng nhánh `main` khớp 100% với nhánh sao lưu ổn định:
   ```bash
   git reset --hard origin/backup/tauri-dialogue-bypass
   ```

4. Đẩy đè cấu hình khôi phục lên GitHub (nếu cần thiết lập lại nhánh main trên remote):
   ```bash
   git push -f origin main
   ```

---

## Các tính năng được bảo toàn trong bản khôi phục này:
* **NPC Dialogue Rapid Bypass**: Tự động click 3 lần cách nhau 300ms đối với nút mở shop (`open-farm-shop.png`, `open-seed-shop.png`), nút đóng mua hạt (`close-seed.png`) và đóng cửa hàng bán (`close-harvest.png`). Loại trừ nút đóng bảng thu hoạch sau khi hái xong quả.
* **Auto-Login**: Tự động mở lại game khi crash, có ô nhập cấu hình chờ load game động từ UI.
* **Dynamic Hot-Reloading**: Cập nhật cài đặt bật/tắt kịch bản mua hạt/thu hoạch theo thời gian thực mà không cần tạm dừng bot.
* **Ultra-Fast Sell**: Bán nông sản siêu tốc với thời gian quét chờ confirm.png chỉ 1 giây.
