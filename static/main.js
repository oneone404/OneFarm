// Kiểm tra xem Tauri có tồn tại không
if (!window.__TAURI__) {
    alert("Lỗi: Không tìm thấy API Tauri. Vui lòng kiểm tra cấu hình withGlobalTauri.");
}

const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

// Khởi tạo cửa sổ
let appWindow;
try {
    appWindow = getCurrentWindow();
} catch (e) {
    console.error("Không thể khởi tạo appWindow:", e);
}

// Chờ DOM sẵn sàng
document.addEventListener('DOMContentLoaded', () => {
    initWindowControls();
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

// --- LOGIC APP CHÍNH ---
let availableTemplates = [];
let availableDevices = [];

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

function toggleTheme() {
    const current = document.documentElement.getAttribute('data-theme');
    const target = current === 'dark' ? 'light' : 'dark';
    document.documentElement.setAttribute('data-theme', target);
    localStorage.setItem('theme', target);
    updateThemeIcons(target);
}

function initTheme() {
    const savedTheme = localStorage.getItem('theme') || 'light';
    document.documentElement.setAttribute('data-theme', savedTheme);
    updateThemeIcons(savedTheme);
}

function updateThemeIcons(theme) {
    const sun = document.getElementById('sun-icon');
    const moon = document.getElementById('moon-icon');
    if (!sun || !moon) return;
    if (theme === 'dark') {
        sun.classList.add('hidden');
        moon.classList.remove('hidden');
    } else {
        sun.classList.remove('hidden');
        moon.classList.add('hidden');
    }
}

function log(msg, type = 'info') {
    const consoleEl = document.getElementById('console');
    if (!consoleEl) return;
    const time = new Date().toLocaleTimeString('vi-VN', { hour12: false });
    const entry = document.createElement('div');
    entry.className = `log-entry ${type}`;
    entry.innerHTML = `<span style="color: #64748b; font-size: 0.7rem;">[${time}]</span> ${msg}`;
    consoleEl.appendChild(entry);
    setTimeout(() => { consoleEl.scrollTop = consoleEl.scrollHeight; }, 10);
}

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

async function createDeviceRow(device, idx) {
    const deviceListEl = document.getElementById('device-list');
    const row = document.createElement('div');
    row.className = 'device-row';
    
    // Check initial session status
    const isSessionActive = await invoke('check_session', { handle: device.handle });
    
    const info = document.createElement('div');
    info.className = 'device-info';
    
    const nameDiv = document.createElement('div');
    nameDiv.className = 'device-name';
    nameDiv.textContent = device.title + ' ';
    
    const badgeSpan = document.createElement('span');
    badgeSpan.className = 'session-badge';
    if (isSessionActive) {
        badgeSpan.classList.add('active');
        badgeSpan.textContent = 'Connected';
    } else {
        badgeSpan.classList.add('inactive', 'clickable');
        badgeSpan.textContent = 'Ready';
        row.classList.add('disabled-row');
    }
    
    // Bind all connect/disconnect/hover events dynamically
    bindBadgeEvents(badgeSpan, device, row);
    
    nameDiv.appendChild(badgeSpan);
    
    const serialDiv = document.createElement('div');
    serialDiv.className = 'device-serial';
    serialDiv.textContent = device.serial;
    
    info.append(nameDiv, serialDiv);
    
    const btnResize = document.createElement('button');
    btnResize.className = 'primary-btn';
    btnResize.innerHTML = 'Resize';
    btnResize.onclick = () => handleResize(device, row);

    const btnCapture = document.createElement('button');
    btnCapture.className = 'primary-btn';
    btnCapture.innerHTML = 'Capture';
    btnCapture.onclick = async () => {
        await handleCapture(device, row);
        updateSessionBadge(device, row);
    };

    const select = document.createElement('select');
    select.className = 'template-select';
    availableTemplates.forEach(t => {
        const opt = document.createElement('option');
        opt.value = t; opt.textContent = t;
        select.appendChild(opt);
    });

    const btnTest = document.createElement('button');
    btnTest.className = 'accent-btn';
    btnTest.innerHTML = 'Test';
    btnTest.onclick = async () => {
        await handleTest(device, select.value, row);
        updateSessionBadge(device, row);
    };

    const rowLog = document.createElement('div');
    rowLog.className = 'row-log';
    rowLog.textContent = 'San sang';

    row.append(info, btnResize, btnCapture, select, btnTest, rowLog);
    deviceListEl.appendChild(row);
}

function bindBadgeEvents(badgeSpan, device, row) {
    // Hover text changing for active state (Connected -> Disconnect)
    badgeSpan.onmouseenter = () => {
        if (badgeSpan.classList.contains('active')) {
            badgeSpan.textContent = 'Disconnect';
            badgeSpan.style.backgroundColor = '#fee2e2';
            badgeSpan.style.color = '#991b1b';
            badgeSpan.style.borderColor = '#fecaca';
        }
    };
    
    badgeSpan.onmouseleave = () => {
        if (badgeSpan.classList.contains('active')) {
            badgeSpan.textContent = 'Connected';
            badgeSpan.style.backgroundColor = '';
            badgeSpan.style.color = '';
            badgeSpan.style.borderColor = '';
        }
    };

    badgeSpan.onclick = async () => {
        // 1. Connect Session
        if (badgeSpan.classList.contains('inactive') && badgeSpan.classList.contains('clickable')) {
            badgeSpan.className = 'session-badge connecting';
            badgeSpan.textContent = 'Connecting...';
            
            try {
                await invoke('connect_session', { device });
                badgeSpan.className = 'session-badge active';
                badgeSpan.textContent = 'Connected';
                row.classList.remove('disabled-row');
                log(`[${device.title}] Da ket noi thanh cong session moi.`, 'success');
            } catch (err) {
                badgeSpan.className = 'session-badge inactive clickable';
                badgeSpan.textContent = 'Ready';
                log(`[${device.title}] Ket noi that bai: ${err}`, 'error');
            }
        } 
        // 2. Disconnect Session
        else if (badgeSpan.classList.contains('active')) {
            badgeSpan.className = 'session-badge connecting';
            badgeSpan.textContent = 'Disconnecting...';
            badgeSpan.style.backgroundColor = '';
            badgeSpan.style.color = '';
            badgeSpan.style.borderColor = '';
            
            try {
                await invoke('disconnect_session', { handle: device.handle });
                badgeSpan.className = 'session-badge inactive clickable';
                badgeSpan.textContent = 'Ready';
                row.classList.add('disabled-row');
                log(`[${device.title}] Da ngat ket noi session.`, 'info');
            } catch (err) {
                badgeSpan.className = 'session-badge active';
                badgeSpan.textContent = 'Connected';
                log(`[${device.title}] Ngat ket noi that bai: ${err}`, 'error');
            }
        }
    };
}

async function updateSessionBadge(device, row) {
    const isSessionActive = await invoke('check_session', { handle: device.handle });
    const badgeSpan = row.querySelector('.session-badge');
    if (!badgeSpan) return;
    
    if (isSessionActive) {
        badgeSpan.className = 'session-badge active';
        badgeSpan.textContent = 'Connected';
        row.classList.remove('disabled-row');
    } else {
        badgeSpan.className = 'session-badge inactive clickable';
        badgeSpan.textContent = 'Ready';
        row.classList.add('disabled-row');
    }
    
    // Re-bind all connect/disconnect/hover events dynamically to match current state
    bindBadgeEvents(badgeSpan, device, row);
}

async function handleResize(device, row) {
    const rowLog = row.querySelector('.row-log');
    try {
        rowLog.textContent = 'Dang Resize...';
        await invoke('set_active_device', { device });
        const res = await invoke('resize_ld');
        rowLog.textContent = 'Xong';
        log(`[${device.title}] ${res}`, 'success');
    } catch (err) {
        rowLog.textContent = 'Loi';
        log(`[${device.title}] ${err}`, 'error');
    } finally {
        await updateSessionBadge(device, row);
    }
}

async function handleCapture(device, row) {
    const rowLog = row.querySelector('.row-log');
    try {
        rowLog.textContent = 'Dang Chup...';
        await invoke('set_active_device', { device });
        const res = await invoke('capture_screen');
        rowLog.textContent = 'Da luu';
        log(`[${device.title}] ${res}`, 'info');
    } catch (err) {
        rowLog.textContent = 'Loi';
        log(`[${device.title}] ${err}`, 'error');
    }
}

async function handleTest(device, template, row) {
    const rowLog = row.querySelector('.row-log');
    if (!template) { log('Vui long chon mau!', 'error'); return; }
    try {
        rowLog.textContent = `Tim: ${template}...`;
        await invoke('set_active_device', { device });
        const res = await invoke('test_template', { name: template });
        if (res.includes('[KHOP]') || res.includes('[KHỚP]')) {
            rowLog.textContent = 'Tim thay!';
        } else {
            rowLog.textContent = 'Khong thay';
        }
        log(`[${device.title}] ${res}`, 'info');
    } catch (err) {
        rowLog.textContent = 'Loi';
        log(`[${device.title}] ${err}`, 'error');
    }
}
