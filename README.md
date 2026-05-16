# OneFarm - LDPlayer Auto-Farm (Ultra Fast Edition) 🚀

Bộ công cụ Auto-Farm tối ưu hóa cho LDPlayer, tập trung vào tốc độ phản hồi cực nhanh và tiết kiệm tài nguyên.

## ✨ Tính năng nổi bật đã hoàn thiện:
- **Tốc độ "Bàn thờ" (1ms)**: Sử dụng Win32 `PostMessage` để click trực tiếp vào vùng Render của LDPlayer, loại bỏ hoàn toàn độ trễ của ADB.
- **Công nghệ Chụp ảnh WGC**: Tận dụng Windows Graphics Capture để chụp ảnh màn hình từ GPU, đạt 60fps mà không tốn CPU.
- **RAM Template Caching**: Tự động nạp toàn bộ ảnh mẫu vào RAM ngay khi khởi động hoặc làm mới, triệt tiêu độ trễ đọc đĩa cứng.
- **Mapping giả lập chuyên nghiệp**: Tích hợp `ldconsole list2` để quản lý đa giả lập chính xác theo Index, Title và Bind Handle.
- **Giao diện hiện đại (Tauri/Vite)**: Hỗ trợ Light/Dark mode, log console tự động cuộn và trình chọn thiết bị thông minh.
- **Thuật toán FastRecognizer**: Bộ nhận diện ảnh mẫu được tối ưu hóa bằng Rust, đảm bảo độ chính xác cao nhất với thời gian xử lý cực thấp.

## 🛠️ Hướng dẫn sử dụng:
1. Mở LDPlayer (nên dùng LDPlayer 9).
2. Chạy ứng dụng OneFarm.
3. Chọn giả lập trong danh sách và ấn **Kết nối**.
4. Sử dụng tính năng **Resize** để chuẩn hóa cửa sổ về 960x540.
5. Quét mẫu và bắt đầu hành trình Auto-Farm!

---
*Ghi chú: Luôn đảm bảo `ldconsole.exe` nằm đúng đường dẫn trong cấu hình.*
