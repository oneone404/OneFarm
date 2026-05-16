const { invoke } = window.__TAURI__.core;
const templateList = document.getElementById('template-list');
const btnResize = document.getElementById('btn-resize');
const btnRefresh = document.getElementById('btn-refresh');
const btnClear = document.getElementById('btn-clear');
const statusMsg = document.getElementById('status-msg');
const consoleEl = document.getElementById('console');
const selectDevice = document.getElementById('select-device');
const btnRefreshDevices = document.getElementById('btn-refresh-devices');
const btnConnect = document.getElementById('btn-connect');
const btnCapture = document.getElementById('btn-capture');
const themeToggle = document.getElementById('theme-toggle');

let availableDevices = [];

// Theme logic
function initTheme() {
    const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    if (isDark) {
        document.documentElement.setAttribute('data-theme', 'dark');
        document.getElementById('sun-icon').classList.add('hidden');
        document.getElementById('moon-icon').classList.remove('hidden');
    }
}

themeToggle.addEventListener('click', () => {
    const current = document.documentElement.getAttribute('data-theme');
    if (current === 'dark') {
        document.documentElement.removeAttribute('data-theme');
        document.getElementById('sun-icon').classList.remove('hidden');
        document.getElementById('moon-icon').classList.add('hidden');
    } else {
        document.documentElement.setAttribute('data-theme', 'dark');
        document.getElementById('sun-icon').classList.add('hidden');
        document.getElementById('moon-icon').classList.remove('hidden');
    }
});

function log(msg, type = 'info') {
    const now = new Date();
    const timeStr = now.toLocaleTimeString('vi-VN', { hour12: false });
    
    const entry = document.createElement('div');
    entry.className = 'log-entry';
    
    const timeSpan = document.createElement('span');
    timeSpan.className = 'log-time';
    timeSpan.textContent = `[${timeStr}]`;
    
    const msgSpan = document.createElement('span');
    msgSpan.className = `log-msg ${type}`;
    msgSpan.textContent = msg;
    
    entry.appendChild(timeSpan);
    entry.appendChild(msgSpan);
    
    consoleEl.appendChild(entry);
    
    // Cuộn xuống đáy mượt mà
    setTimeout(() => {
        consoleEl.scrollTop = consoleEl.scrollHeight;
    }, 10);
    
    statusMsg.textContent = msg;
}

async function loadTemplates() {
    try {
        const templates = await invoke('get_templates');
        templateList.innerHTML = '';
        templates.forEach(name => {
            const btn = document.createElement('button');
            btn.className = 'tpl-btn';
            btn.textContent = name.replace('.png', '');
            btn.onclick = () => testTemplate(name);
            templateList.appendChild(btn);
        });
        log(`Đã load ${templates.length} templates.`, 'success');
    } catch (err) {
        log('Lỗi load templates: ' + err, 'error');
    }
}

async function testTemplate(name) {
    log(`Đang quét mẫu: ${name}...`, 'info');
    try {
        const result = await invoke('test_template', { name });
        // Tách các dòng log và hiển thị từng dòng
        result.split('\n').forEach(line => {
            if (line.trim()) log(line, 'success');
        });
    } catch (err) {
        log(err, 'error');
    }
}

async function captureScreen() {
    log('Đang thực hiện chụp ảnh màn hình...', 'info');
    try {
        const res = await invoke('capture_screen');
        log(res, 'success');
    } catch (err) {
        log('Lỗi chụp ảnh: ' + err, 'error');
    }
}

btnResize.addEventListener('click', async () => {
    log('Đang gửi lệnh chuẩn hóa LDPlayer...', 'info');
    try {
        const res = await invoke('resize_ld');
        log(res, 'success');
    } catch (err) {
        log('Lỗi chuẩn hóa: ' + err, 'error');
    }
});

btnRefresh.addEventListener('click', loadTemplates);
document.getElementById('btn-capture').addEventListener('click', captureScreen);
btnClear.addEventListener('click', () => {
    consoleEl.innerHTML = '';
    log('Đã xóa log.', 'info');
});

async function loadDevices() {
    try {
        availableDevices = await invoke('get_devices');
        selectDevice.innerHTML = '<option value="">-- Chọn LDPlayer --</option>';
        availableDevices.forEach((dev, index) => {
            const opt = document.createElement('option');
            opt.value = index;
            opt.textContent = `${dev.title} (${dev.serial})`;
            selectDevice.appendChild(opt);
        });
        log(`Đã quét thấy ${availableDevices.length} giả lập.`, 'info');
    } catch (err) {
        log('Lỗi quét thiết bị: ' + err, 'error');
    }
}

btnConnect.addEventListener('click', async () => {
    const idx = selectDevice.value;
    if (idx === "") {
        log('Vui lòng chọn một giả lập!', 'error');
        return;
    }
    const device = availableDevices[idx];
    try {
        const res = await invoke('set_active_device', { device });
        log(res, 'success');
    } catch (err) {
        log('Lỗi kết nối: ' + err, 'error');
    }
});

btnRefreshDevices.addEventListener('click', loadDevices);

// Initial Load
initTheme();
loadTemplates();
loadDevices();
log('Hệ thống khởi động thành công.', 'success');
