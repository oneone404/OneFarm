# OneFarm - LDPlayer Auto-Farm (Ultra Fast Edition) 🚀

Bộ công cụ Auto-Farm tối ưu hóa cho LDPlayer, tập trung vào tốc độ phản hồi cực nhanh và tiết kiệm tài nguyên.

## ✨ Tính năng nổi bật đã hoàn thiện:
- **Tốc độ "Bàn thờ" (1ms)**: Sử dụng Win32 `PostMessage` để click trực tiếp vào vùng Render của LDPlayer, loại bỏ hoàn toàn độ trễ của ADB.
- **Công nghệ Chụp ảnh WGC**: Tận dụng Windows Graphics Capture để chụp ảnh màn hình từ GPU, đạt 60fps mà không tốn CPU.
- **RAM Template Caching**: Tự động nạp toàn bộ ảnh mẫu vào RAM ngay khi khởi động hoặc làm mới, triệt tiêu độ trễ đọc đĩa cứng.
- **Mapping giả lập chuyên nghiệp**: Tích hợp `ldconsole list2` để quản lý đa giả lập chính xác theo Index, Title và Bind Handle.
- **Giao diện hiện đại (Tauri/Vite)**: Hỗ trợ Light/Dark mode, log console tự động cuộn và trình chọn thiết bị thông minh.
- **Thuật toán FastRecognizer**: Bộ nhận diện ảnh mẫu được tối ưu hóa bằng Rust, đảm bảo độ chính xác cao nhất với thời gian xử lý cực thấp.
- **Bao ve an toan click**: Chi cho phep thao tac tren gia lap khi trang thai la Connected (nguoi dung phai click chu dong vao badge Ready de kich hoat), giup tranh click nham vao tab khac.
- **Icon thuong hieu rieng**: Tich hop bo icon doc quyen duoc sinh tu dong voi day du cac dinh dang phu hop cho moi nen tang (PNG, ICO, ICNS).
- **Chu dong huy ket noi**: Cho phep nguoi dung nhap chu dong vao nhan Connected de ngat ket noi (Disconnect), lap tuc giai phong hoan toan tai nguyen GPU/RAM cua gia lap do.
- **Tu dong thich ung do phan giai**: Tu dong doi chieu, lam moi grabber khi cua so bi keo gian hoac thu nho va tu dong nhan ti le toa do click de dam bao click luon trung muc tieu tren moi do phan giai cua so.
- **Sao chep pixel sieu toc (Direct Pointer Memory Access)**: Loai bo hoan toan cac buoc kiem tra an toan chi so (Bounds checking) trong vong lap va thuc hien sao chep truc tiep tren vung con tro dong bo, giam thoi gian copy xuong duoi 1ms cho moi khung hinh.
- **Bo qua Chuan hoa thong minh & Nearest Neighbor**: Tu dong bo qua buoc co gian anh khi sai lech duoi 4 pixel (giam ton CPU ve 0ms). Neu can co gian, su dung thuat toan Nearest Neighbor de giu nguyen canh sac net cho template matching va giam thoi gian xu ly xuong duoi 1ms.
- **Nhan dien RGBA Zero-Copy**: Bo qua hoan toan viec doi he mau va cap phat lai bo nho heap (malloc) tu RGBA sang RGB tren moi khung hinh chup tu GPU. FastRecognizer duoc nang cap de ho tro quet tren vung nho dem RGBA voi dung sai dong, tiet kiem 2ms va triet tieu phan manh RAM.
- **Triet tieu phan manh RAM & Roc bo nho**: Thiet ke lai co che FrameArrived de tai su dung duy nhat mot vung nho dem pre-allocated trong Mutex cho luong chup anh GPU (loai bo 270MB/giay phan bo Heap vo ich o luong nen). Dong thoi ap dung con tro thong minh Arc de chia se anh mau trong cache, triet tieu 100% ruy ro ro ri bo nho (memory leak).
- **Kien truc Modular huong Tuong lai (Future-proof Modular Architecture)**: Tai cau truc toan dien Backend thanh cac phan lop chuyen biet nhu `core/` (tinh toan GPU Capture, Fast Matching), `emulator/` (ho tro giao tiep da gia lap Nox/LDPlayer/ADB), giup viec tich hop them cac kich ban auto-farm phuc tap sau nay cuc ky de dang va khoa hoc.

## 🛠️ Hướng dẫn sử dụng:
1. Mở LDPlayer (nên dùng LDPlayer 9).
2. Chạy ứng dụng OneFarm.
3. Chọn giả lập trong danh sách và ấn **Kết nối**.
4. Sử dụng tính năng **Resize** để chuẩn hóa cửa sổ về 960x540.
5. Quét mẫu và bắt đầu hành trình Auto-Farm!

---
*Ghi chú: Luôn đảm bảo `ldconsole.exe` nằm đúng đường dẫn trong cấu hình.*
