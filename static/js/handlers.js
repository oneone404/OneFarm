// ─── ACTION HANDLERS ──────────────────────────────────────────────────────────
const SVG_PAUSE_HANDLERS = `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>`;

// Helper dùng chung: quản lý trạng thái nút (running / stop / restore)
async function _runAbortable(device, btn, task) {
    if (btn.dataset.running === 'true') {
        btn.disabled = true;
        try { await invoke('cancel_device_actions', { handle: device.handle }); }
        catch (e) { log('Lỗi dừng: ' + e, 'error'); }
        return false;
    }
    const origHTML   = btn.innerHTML;
    const origBG     = btn.style.backgroundColor;
    const origBorder = btn.style.borderColor;
    btn.dataset.running = 'true';

    if (origHTML.includes('<svg')) {
        btn.innerHTML = SVG_PAUSE_HANDLERS;
    } else {
        btn.textContent = 'Dừng';
    }

    btn.style.backgroundColor = '#f59e0b';
    btn.style.borderColor     = '#f59e0b';
    try { await task(); }
    finally {
        btn.dataset.running = 'false';
        btn.disabled        = false;
        btn.innerHTML       = origHTML;
        btn.style.backgroundColor   = origBG;
        btn.style.borderColor       = origBorder;
    }
    return true;
}

// ── Capture ───────────────────────────────────────────────────────────────────
async function handleCapture(device, row, btn) {
    const rowLog = row.querySelector('.row-log');
    await _runAbortable(device, btn, async () => {
        rowLog.textContent = 'Dang Chup...';
        await invoke('set_active_device', { device });
        try {
            const res = await invoke('capture_screen');
            rowLog.textContent = 'Da luu';
            log(`[${device.title}] ${res}`, 'info');
        } catch (err) {
            rowLog.textContent = 'Loi';
            log(`[${device.title}] ${err}`, 'error');
        }
    });
}

// ── Test template đơn ─────────────────────────────────────────────────────────
async function handleTest(device, template, row, btn) {
    if (!template) { log('Vui long chon mau!', 'error'); return; }
    const rowLog = row.querySelector('.row-log');
    await _runAbortable(device, btn, async () => {
        rowLog.textContent = `Tim: ${template}...`;
        await invoke('set_active_device', { device });
        try {
            const res = await invoke('test_template', { name: template });
            rowLog.textContent = 'Tim thay!';
            log(`[${device.title}] ${res}`, 'info');
        } catch (err) {
            rowLog.textContent = 'Khong thay';
            log(`[${device.title}] ${err}`, 'error');
        }
    });
}

// ── Test All templates ────────────────────────────────────────────────────────
async function handleTestAll(device, row, btn) {
    const rowLog = row.querySelector('.row-log');
    await _runAbortable(device, btn, async () => {
        rowLog.textContent = 'Checking all...';
        await invoke('set_active_device', { device });
        try {
            const res = await invoke('test_all_templates');
            log(res, 'info');
            rowLog.textContent = 'Checked';
        } catch (err) {
            log(err, 'error');
            rowLog.textContent = 'Loi';
        }
    });
}

// ── Check Lỗi seeds ───────────────────────────────────────────────────────────
async function handleCheckSeeds(device, row, btn) {
    const rowLog = row.querySelector('.row-log');
    await _runAbortable(device, btn, async () => {
        rowLog.textContent = 'Checking loi...';
        await invoke('set_active_device', { device });
        try {
            const res = await invoke('check_seeds_templates');
            log(res, 'info');
            rowLog.textContent = 'Checked';
        } catch (err) {
            log(err, 'error');
            rowLog.textContent = 'Loi';
        }
    });
}

// ── Test Số (nhận dạng chữ số trên slider) ────────────────────────────────────
async function handleTestDigits(device, row, btn) {
    const rowLog = row.querySelector('.row-log');
    await _runAbortable(device, btn, async () => {
        rowLog.textContent = 'Testing digits...';
        await invoke('set_active_device', { device });
        try {
            const res = await invoke('test_digit_recognition');
            log(res, 'info');
            rowLog.textContent = 'Checked';
        } catch (err) {
            log(err, 'error');
            rowLog.textContent = 'Loi';
        }
    });
}
