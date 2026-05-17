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

function initAppLogic() {
    const btnRefresh = document.getElementById('btn-refresh-devices');
    const themeToggle = document.getElementById('theme-toggle');

    if (btnRefresh) {
        btnRefresh.addEventListener('click', async () => {
            log('Đang làm mới dữ liệu (Templates & Devices)...', 'info');
            await loadTemplates();
            await loadDevices();
        });
    }
    if (themeToggle) {
        themeToggle.addEventListener('click', toggleTheme);
        initTheme();
    }

    loadTemplates().then(loadDevices);
    log('Hệ thống OneFarm Multi đã sẵn sàng.', 'success');
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
            e.preventDefault();
            e.stopPropagation();
            return false;
        }
    }, true);
}
