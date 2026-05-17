# Ultra-Fast Auto (Rust) - LDPlayer Automation Suite

Bộ công cụ tự động hóa hiệu năng cao dành cho LDPlayer, được tối ưu hóa bằng ngôn ngữ Rust và công nghệ đồ họa Windows hiện đại.

## Các tính năng đã hoàn thiện:

### 1. Hệ thống Thị giác (Vision System)
*   **WGC Capture Engine**: Sử dụng Windows Graphics Capture (WGC) để chụp ảnh màn hình tốc độ cao, độ trễ cực thấp (gần như 0ms).
*   **Smart Memory (ROI Scan)**: Cơ chế "Trí nhớ thông minh" giúp ghi nhớ vị trí các nút bấm. Sau khi tìm thấy lần đầu, tool chỉ quét vùng nhỏ (Region of Interest) xung quanh nút đó, tăng tốc độ quét lên tới 30-50 lần (chỉ mất 0-1ms).
*   **Resolution Guard**: Tự động phát hiện khi người dùng co dãn cửa sổ LDPlayer. Tool sẽ tự động khởi động lại phiên capture và xóa trí nhớ cũ để học lại tọa độ mới, đảm bảo độ chính xác tuyệt đối.

### 2. Hệ thống Tương tác (Interaction System)
*   **ADB TCP Client**: Giao tiếp trực tiếp với ADB Server qua TCP (cổng 5037), bỏ qua việc gọi file `adb.exe` để giảm latency tối đa.
*   **Centered Click**: Tự động tính toán và click vào chính giữa ảnh mẫu (Template Center).
*   **Coordinate Scaling**: Tự động quy đổi tọa độ từ pixel cửa sổ Windows sang độ phân giải thực tế của Android (mặc định 960x540).
*   **Anti-Cheat Randomization**: Tự động làm lệch tọa độ click ngẫu nhiên (±2 pixel) để mô phỏng thao tác tay người dùng.

### 3. Tối ưu hóa & Debug
*   **SAD Template Matching**: Sử dụng thuật toán Sum of Absolute Differences (SAD) tối ưu hóa bằng Rust để nhận diện hình ảnh cực nhanh.
*   **Debug Mode**: Tự động xuất file `debug_view.png` sau mỗi vòng quét để người dùng theo dõi quá trình nhận diện.
*   **Configurable Polling**: Nhịp độ quét hiện tại được đặt ở mức 5 giây/lần để tiết kiệm tài nguyên trong quá trình kiểm thử.

## Cách sử dụng:
1.  Bỏ các ảnh mẫu cần click vào thư mục `templates/`.
2.  Mở LDPlayer (tên cửa sổ mặc định là `LD-1`).
3.  Chạy tool: `cargo run --release`.

---
*Dự án được phát triển với tiêu chí: Tốc độ - Thông minh - Tự chữa lành.* 🛠️🚀💎
