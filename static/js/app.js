// Chờ DOM sẵn sàng
document.addEventListener('DOMContentLoaded', () => {
    initWindowControls();
    initKeyboardBlocker();
    initAppLogic();
});

function initWindowControls() {
    const header = document.querySelector('header');
    if (!header) return;

    // 1. Kéo thả
    header.addEventListener('mousedown', (e) => {
        if (!e.target.closest('button') && !e.target.closest('.win-btn')) {
            try { appWindow.startDragging(); } catch(err) { console.error(err); }
        }
    });

    // 2. Nhấn đúp
    header.addEventListener('dblclick', (e) => {
        if (!e.target.closest('button') && !e.target.closest('.win-btn')) {
            try { appWindow.toggleMaximize(); } catch(err) { console.error(err); }
        }
    });

    // 3. Nút điều khiển
    document.addEventListener('click', (e) => {
        const winBtn = e.target.closest('.win-btn');
        if (!winBtn) return;

        try {
            if (winBtn.id === 'win-min') appWindow.minimize();
            if (winBtn.id === 'win-max') appWindow.toggleMaximize();
            if (winBtn.id === 'win-close') appWindow.close();
        } catch (err) {
            console.error("Lỗi điều khiển cửa sổ:", err);
        }
    });
}

async function refreshAll() {
    log('Đang làm mới dữ liệu (Templates & Devices)...', 'info');
    await loadTemplates();
    
    // 1. Quét thiết bị và thực hiện Resize (ngắt session cũ) trước
    try {
        const devices = await invoke('get_devices');
        if (devices.length > 0) {
            log('Đang tự động căn chuẩn độ phân giải cho tất cả giả lập (960x540)...', 'info');
            for (const device of devices) {
                try {
                    await invoke('set_active_device', { device });
                    const res = await invoke('resize_ld');
                    log(`[${device.title}] ${res}`, 'success');
                } catch (err) {
                    console.error(`[${device.title}] Không thể tự động resize: ${err}`);
                }
            }
        }
    } catch (err) {
        log('Lỗi quét thiết bị để resize: ' + err, 'error');
    }

    // 2. Load và render giao diện thiết bị sau (để hiển thị đúng trạng thái session đã ngắt)
    await loadDevices();
}

function initAppLogic() {
    const btnRefresh = document.getElementById('btn-refresh-devices');

    if (btnRefresh) {
        btnRefresh.addEventListener('click', refreshAll);
    }

    // Initialize only the light theme
    initTheme();

    log('Hệ thống OneFarm Multi đã sẵn sàng. Vui lòng bấm nút Làm mới để quét thiết bị.', 'success');
}

function initKeyboardBlocker() {
    window.addEventListener('keydown', (e) => {
        // Chặn phím chức năng F1 đến F12 (ví dụ F5 refresh, F12 DevTools...)
        if (e.key.startsWith('F') && e.key.length > 1) {
            const fNum = parseInt(e.key.substring(1));
            if (fNum >= 1 && fNum <= 12) {
                e.preventDefault();
                e.stopPropagation();
                return false;
            }
        }
        
        // Chặn các tổ hợp phím hệ thống Ctrl hoặc Command (Ctrl+R, Ctrl+F, Ctrl+S, Ctrl+P...)
        if (e.ctrlKey || e.metaKey) {
            const key = e.key.toLowerCase();
            // Cho phép sao chép (C), dán (V), cắt (X), chọn tất cả (A)
            if (key === 'c' || key === 'v' || key === 'x' || key === 'a') {
                return;
            }
            e.preventDefault();
            e.stopPropagation();
            return false;
        }
    }, true);
}
