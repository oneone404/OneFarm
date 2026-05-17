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
    rowLog.style.display = 'none';
    rowLog.textContent = 'San sang';
 
    row.append(info, btnResize, btnCapture, select, btnTest, rowLog);
    deviceListEl.appendChild(row);
}
 
function bindBadgeEvents(badgeSpan, device, row) {
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
