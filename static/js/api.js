// Kiểm tra xem Tauri có tồn tại không
if (!window.__TAURI__) {
    alert("Lỗi: Không tìm thấy API Tauri. Vui lòng kiểm tra cấu hình withGlobalTauri.");
}

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

let appWindow;
try {
    appWindow = getCurrentWindow();
} catch (e) {
    console.error("Không thể khởi tạo appWindow:", e);
}

let availableTemplates = [];
let availableDevices = [];

async function loadTemplates() {
    try {
        availableTemplates = await invoke('get_templates');
    } catch (err) {
        log('Lỗi tải mẫu: ' + err, 'error');
    }
}

async function loadDevices() {
    const deviceListEl = document.getElementById('device-list');
    if (!deviceListEl) return;

    try {
        deviceListEl.innerHTML = '<div style="text-align: center; padding: 40px; font-size: 0.8rem;">Đang tìm kiếm giả lập...</div>';
        availableDevices = await invoke('get_devices');
        deviceListEl.innerHTML = '';

        if (availableDevices.length === 0) {
            deviceListEl.innerHTML = '<div style="text-align: center; padding: 40px; color: var(--text-secondary); font-size: 0.8rem;">Không tìm thấy giả lập LDPlayer đang mở.</div>';
            return;
        }

        availableDevices.forEach((dev, idx) => {
            createDeviceRow(dev, idx);
        });
        log(`Đã tìm thấy ${availableDevices.length} giả lập.`, 'info');
    } catch (err) {
        log('Lỗi quét thiết bị: ' + err, 'error');
    }
}
