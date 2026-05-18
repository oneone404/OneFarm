// ─── SVG CONTROLS ─────────────────────────────────────────────────────────────
const SVG_PLAY  = `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>`;
const SVG_PAUSE = `<svg width="16" height="16" viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>`;

// ─── DEVICE ROW BUILDER ───────────────────────────────────────────────────────
async function createDeviceRow(device, idx) {
    const deviceListEl = document.getElementById('device-list');
    let tableBody = document.getElementById('device-table-body');

    // Khởi tạo khung Bảng Premium nếu chưa tồn tại
    if (!tableBody) {
        deviceListEl.innerHTML = '';

        const container = document.createElement('div');
        container.className = 'table-container';
        container.style.cssText = 'width: 100%; overflow-x: auto; border-radius: 8px; border: 2px solid var(--border-color); background-color: var(--card-bg);';

        const table = document.createElement('table');
        table.className = 'premium-table';
        table.style.cssText = 'width: 100%; border-collapse: collapse; text-align: left; font-size: 0.75rem; border: none;';

        const thead = document.createElement('thead');
        thead.innerHTML = `
            <tr style="background-color: var(--secondary-color); border-bottom: 2px solid var(--border-color);">
                <th style="padding: 12px 14px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; font-size: 0.68rem;">LD</th>
                <th style="padding: 12px 14px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; font-size: 0.68rem;">STATUS</th>
                <th style="padding: 12px 14px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; font-size: 0.68rem;">CONNECT</th>
                <th style="padding: 12px 14px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; font-size: 0.68rem;">CHỨC NĂNG</th>
                <th style="padding: 12px 14px; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; font-size: 0.68rem;">THAO TÁC</th>
            </tr>
        `;
        table.appendChild(thead);

        tableBody = document.createElement('tbody');
        tableBody.id = 'device-table-body';
        table.appendChild(tableBody);

        container.appendChild(table);
        deviceListEl.appendChild(container);
    }

    const isSessionActive = await invoke('check_session', { handle: device.handle });

    let savedConfig = { selected_seeds: [] };
    try { savedConfig = await invoke('get_config'); }
    catch (e) { console.error('Lỗi đọc config:', e); }

    // Tạo dòng bảng <tr> đại diện cho giả lập
    const row = document.createElement('tr');
    row.className = 'device-row';
    row.style.cssText = 'transition: background-color 0.2s;';

    // 1. Cột LD (Tên thiết bị)
    const tdLd = document.createElement('td');
    tdLd.style.cssText = 'padding: 8px 14px; font-weight: 700; color: var(--text-primary); vertical-align: middle; white-space: nowrap;';
    tdLd.textContent = device.title;
    row.appendChild(tdLd);


    // 3. Cột STATUS (Trạng thái và Log tích hợp)
    const tdStatus = document.createElement('td');
    tdStatus.style.cssText = 'padding: 8px 14px; vertical-align: middle;';
    const rowLog = document.createElement('div');
    rowLog.className = 'row-log';
    rowLog.style.cssText = 'font-size: 0.72rem; font-weight: 600; color: var(--text-secondary); max-width: 140px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;';
    rowLog.textContent = 'Sẵn sàng';
    tdStatus.appendChild(rowLog);
    row.appendChild(tdStatus);

    // 4. Cột CONNECT (Session Toggle Switch)
    const tdConnect = document.createElement('td');
    tdConnect.style.cssText = 'padding: 8px 14px; text-align: left; vertical-align: middle; white-space: nowrap;';
    
    const switchContainer = document.createElement('div');
    switchContainer.className = 'switch-container';
    
    const toggleSwitch = document.createElement('div');
    toggleSwitch.className = 'toggle-switch';
    
    const switchLabel = document.createElement('span');
    switchLabel.className = 'switch-label';
    
    if (isSessionActive) {
        toggleSwitch.classList.add('active');
        switchLabel.classList.add('active');
        switchLabel.textContent = 'Connected';
    } else {
        switchLabel.textContent = 'Ready';
        row.classList.add('disabled-row');
    }
    
    switchContainer.append(toggleSwitch, switchLabel);
    bindBadgeEvents(toggleSwitch, switchLabel, device, row);
    tdConnect.appendChild(switchContainer);
    row.appendChild(tdConnect);

    // 5. Cột CHỨC NĂNG (Gồm Chọn mẫu & Lựa chọn thao tác nhanh)
    const tdFunctions = document.createElement('td');
    tdFunctions.style.cssText = 'padding: 8px 14px; vertical-align: middle;';

    const funcContainer = document.createElement('div');
    funcContainer.style.cssText = 'display: flex; align-items: center; gap: 8px; width: 100%;';

    // Select dropdown chọn mẫu
    const select = document.createElement('select');
    select.className = 'template-select';
    select.style.cssText = 'flex: 1; min-width: 120px; width: 0;';
    availableTemplates.forEach(t => {
        const opt = document.createElement('option');
        opt.value = t; opt.textContent = t;
        select.appendChild(opt);
    });

    // Custom select chứa các chức năng chẩn đoán & thao tác nhanh
    const selectAction = document.createElement('select');
    selectAction.className = 'action-select';
    selectAction.style.cssText = 'width: 130px; flex-shrink: 0; font-size: 0.72rem; font-weight: 700;';
    selectAction.innerHTML = `
        <option value="run_script" selected>Chạy kịch bản</option>
        <option value="capture">Chụp màn hình</option>
        <option value="test">Test mẫu đang chọn</option>
        <option value="test_all">Test tất cả mẫu</option>
        <option value="check_seeds">Chẩn đoán lỗi</option>
        <option value="test_digits">Nhận dạng số</option>
    `;

    funcContainer.append(select, selectAction);
    tdFunctions.appendChild(funcContainer);
    row.appendChild(tdFunctions);

    // 6. Cột THAO TÁC (Tích hợp phím Play/Pause và nút cấu hình)
    const tdActions = document.createElement('td');
    tdActions.style.cssText = 'padding: 8px 14px; text-align: left; vertical-align: middle; white-space: nowrap;';

    const actionContainer = document.createElement('div');
    actionContainer.style.cssText = 'display: flex; align-items: center; justify-content: flex-start; gap: 6px;';

    // ── KỊCH BẢN LOOP ────────────────────────────────────────────────────────
    let loopInterval = null;
    let isExecuting = false;
    let lastHarvestTime = 0;

    async function executeHarvest() {
        if (isExecuting || btnRunScript.dataset.running !== 'true') return;
        isExecuting = true;
        rowLog.textContent = 'Thu hoạch...';
        try {
            await invoke('set_active_device', { device });
            log(`[${device.title}] Bắt đầu kịch bản: Thu hoạch & Bán nông sản...`, 'info');
            const res = await invoke('run_harvest_sell_script');
            log(res, 'success');
            lastHarvestTime = Date.now();
            rowLog.textContent = 'Thu hoạch Xong';
        } catch (err) {
            log(`[${device.title}] Lỗi kịch bản Thu hoạch: ${err}`, 'error');
            rowLog.textContent = 'Thu hoạch Lỗi';
        } finally {
            isExecuting = false;
            await updateSessionBadge(device, row);
            if (btnRunScript.dataset.running === 'true') {
                rowLog.textContent = 'Chờ Auto...';
            }
        }
    }

    async function executeBuy() {
        if (isExecuting || btnRunScript.dataset.running !== 'true') return;
        isExecuting = true;
        rowLog.textContent = 'Mua hạt...';
        try {
            await invoke('set_active_device', { device });
            const targets = savedConfig.selected_seeds || [];
            log(`[${device.title}] Bắt đầu kịch bản Mua hạt: ${targets.join(', ')}`, 'info');
            const res = await invoke('run_buy_seeds_script', { targetSeeds: targets });
            log(res, 'success');
            rowLog.textContent = 'Mua hạt Xong';
        } catch (err) {
            log(`[${device.title}] Lỗi kịch bản Mua hạt: ${err}`, 'error');
            rowLog.textContent = 'Mua hạt Lỗi';
        } finally {
            isExecuting = false;
            await updateSessionBadge(device, row);
            if (btnRunScript.dataset.running === 'true') {
                rowLog.textContent = 'Chờ Auto...';
            }
        }
    }

    function stopScriptLoop() {
        if (loopInterval) { clearInterval(loopInterval); loopInterval = null; }
        btnRunScript.dataset.running = 'false';
        btnRunScript.disabled = false;
        btnRunScript.innerHTML = SVG_PLAY;
        btnRunScript.style.backgroundColor = 'var(--accent-color)';
        btnRunScript.style.borderColor = 'var(--accent-color)';
        rowLog.textContent = 'Sẵn sàng';

        // Nếu đang thực thi giữa chừng, gửi lệnh dừng xuống backend
        if (isExecuting) {
            invoke('cancel_device_actions', { handle: device.handle })
                .catch(e => console.error('Lỗi dừng:', e));
        }
    }

    // Nút chạy kịch bản chính (Icon Play/Pause duy nhất)
    const btnRunScript = _makeBtn('btn-run-script', '', null);
    btnRunScript.style.cssText = 'width: 26px; height: 26px; padding: 0; display: flex; align-items: center; justify-content: center; border-radius: 4px; border: 2px solid var(--accent-color); background-color: var(--accent-color); color: white; cursor: pointer; transition: all 0.2s;';
    btnRunScript.innerHTML = SVG_PLAY;

    btnRunScript.onclick = async () => {
        if (btnRunScript.dataset.running === 'true') {
            btnRunScript.disabled = true;
            stopScriptLoop();
            return;
        }

        const act = selectAction.value;

        if (act === 'run_script') {
            const targets = savedConfig.selected_seeds || [];
            const enableBuy = savedConfig.enable_buy_seeds !== false;
            const enableHarvest = savedConfig.enable_harvest_sell !== false;

            if (enableBuy && targets.length === 0) {
                log('Vui lòng mở cấu hình hạt giống (⚙️) và chọn ít nhất một loại!', 'error');
                rowLog.textContent = 'Lỗi cấu hình';
                return;
            }

            if (!enableBuy && !enableHarvest) {
                log('Vui lòng mở cấu hình hạt giống (⚙️) và kích hoạt ít nhất một kịch bản hoạt động!', 'error');
                rowLog.textContent = 'Lỗi cấu hình';
                return;
            }

            // Chuyển phím sang trạng thái "Đang chạy" màu vàng cam đậm bền vững
            btnRunScript.dataset.running = 'true';
            btnRunScript.innerHTML = SVG_PAUSE;
            btnRunScript.style.backgroundColor = '#f59e0b';
            btnRunScript.style.borderColor = '#f59e0b';
            rowLog.textContent = 'Đang chạy...';

            lastHarvestTime = (savedConfig.enable_harvest_sell !== false) ? 0 : -1;
            let lastBuyTime = (savedConfig.enable_buy_seeds !== false) ? 0 : -1;
            row.dataset.lastRunMinute = '';

            loopInterval = setInterval(async () => {
                if (btnRunScript.dataset.running !== 'true') {
                    stopScriptLoop();
                    return;
                }

                if (isExecuting) return;

                // Đọc trực tiếp cấu hình in-memory động thời gian thực
                const enableBuy = savedConfig.enable_buy_seeds !== false;
                const enableHarvest = savedConfig.enable_harvest_sell !== false;

                // Tự động điều chỉnh trạng thái kịch bản khi người dùng lưu thay đổi
                if (enableHarvest && lastHarvestTime === -1) {
                    lastHarvestTime = 0;
                } else if (!enableHarvest) {
                    lastHarvestTime = -1;
                }

                if (enableBuy && lastBuyTime === -1) {
                    lastBuyTime = 0;
                } else if (!enableBuy) {
                    lastBuyTime = -1;
                }

                // Nếu người dùng tắt toàn bộ kịch bản
                if (!enableBuy && !enableHarvest) {
                    rowLog.textContent = 'Auto Tắt';
                    return;
                }

                const harvestIntervalMins = savedConfig.harvest_interval_mins || 30;
                const harvestIntervalMs = harvestIntervalMins * 60 * 1000;
                const now = Date.now();

                // 1. Chạy lượt đầu tiên (Hoặc chu kỳ lặp) của Thu hoạch & Bán
                if (enableHarvest && (lastHarvestTime === 0 || (lastHarvestTime > 0 && now - lastHarvestTime >= harvestIntervalMs))) {
                    await executeHarvest();
                    return;
                }

                // 2. Chạy lượt đầu tiên của Mua hạt giống
                if (enableBuy && lastBuyTime === 0) {
                    await executeBuy();
                    lastBuyTime = Date.now();
                    const currentMin = new Date().getMinutes();
                    row.dataset.lastRunMinute = String(currentMin);
                    return;
                }

                // 3. Vòng lặp Mua hạt giống (cứ mỗi 5 phút đồng hồ tuyệt đối)
                const dateObj = new Date();
                const min = dateObj.getMinutes();
                if (enableBuy && min % 5 === 0 && row.dataset.lastRunMinute !== String(min)) {
                    row.dataset.lastRunMinute = String(min);
                    await executeBuy();
                    return;
                }

                // 4. Cập nhật dòng trạng thái chờ đếm ngược cực kỳ đẹp mắt
                if (enableBuy && enableHarvest) {
                    let nextBuyMin = Math.ceil((min + 0.001) / 5) * 5;
                    const tgt = new Date(dateObj);
                    if (nextBuyMin >= 60) { tgt.setHours(tgt.getHours() + 1); tgt.setMinutes(0); }
                    else { tgt.setMinutes(nextBuyMin); }
                    tgt.setSeconds(0); tgt.setMilliseconds(0);

                    const diff = Math.max(0, Math.ceil((tgt - dateObj) / 1000));
                    const cdStr = `${Math.floor(diff / 60)}:${String(diff % 60).padStart(2, '0')}`;

                    const nextHarvestMs = Math.max(0, harvestIntervalMs - (now - lastHarvestTime));
                    const harvestCDStr = `${Math.ceil(nextHarvestMs / 60000)}m`;

                    rowLog.textContent = `Chờ Auto (Mua: ${cdStr} | TH: ${harvestCDStr})`;
                } else if (enableBuy) {
                    let nextBuyMin = Math.ceil((min + 0.001) / 5) * 5;
                    const tgt = new Date(dateObj);
                    if (nextBuyMin >= 60) { tgt.setHours(tgt.getHours() + 1); tgt.setMinutes(0); }
                    else { tgt.setMinutes(nextBuyMin); }
                    tgt.setSeconds(0); tgt.setMilliseconds(0);

                    const diff = Math.max(0, Math.ceil((tgt - dateObj) / 1000));
                    const cdStr = `${Math.floor(diff / 60)}:${String(diff % 60).padStart(2, '0')}`;

                    rowLog.textContent = `Chờ Auto (Mua: ${cdStr})`;
                } else if (enableHarvest) {
                    const nextHarvestMs = Math.max(0, harvestIntervalMs - (now - lastHarvestTime));
                    const harvestCDStr = `${Math.ceil(nextHarvestMs / 60000)}m`;

                    rowLog.textContent = `Chờ Auto (TH: ${harvestCDStr})`;
                }
            }, 1000);
        } else {
            // Chạy các hành động chẩn đoán thông thường qua _runAbortable
            if (act === 'capture') {
                await handleCapture(device, row, btnRunScript);
                updateSessionBadge(device, row);
            } else if (act === 'test') {
                await handleTest(device, select.value, row, btnRunScript);
                updateSessionBadge(device, row);
            } else if (act === 'test_all') {
                await handleTestAll(device, row, btnRunScript);
                updateSessionBadge(device, row);
            } else if (act === 'check_seeds') {
                await handleCheckSeeds(device, row, btnRunScript);
                updateSessionBadge(device, row);
            } else if (act === 'test_digits') {
                await handleTestDigits(device, row, btnRunScript);
                updateSessionBadge(device, row);
            }
        }
    };

    row.cleanupAutoInterval = () => {
        if (loopInterval) { clearInterval(loopInterval); loopInterval = null; }
    };

    // Cài đặt cấu hình tổng hợp ⚙️ (Gộp cài đặt chung + hạt giống)
    const btnSeedConfig = document.createElement('button');
    btnSeedConfig.className = 'icon-btn';
    btnSeedConfig.style.cssText = 'background-color: var(--secondary-color); border: 2px solid var(--border-color); color: var(--text-secondary); width: 26px; height: 26px; padding: 0; display: flex; align-items: center; justify-content: center; border-radius: 4px; cursor: pointer; transition: all 0.2s;';
    btnSeedConfig.innerHTML = SVG_GEAR;
    
    // Tạo hiệu ứng Hover mượt mà cho phím bánh răng
    btnSeedConfig.onmouseenter = () => {
        btnSeedConfig.style.borderColor = 'var(--text-secondary)';
        btnSeedConfig.style.color = 'var(--text-primary)';
    };
    btnSeedConfig.onmouseleave = () => {
        btnSeedConfig.style.borderColor = 'var(--border-color)';
        btnSeedConfig.style.color = 'var(--text-secondary)';
    };
    
    btnSeedConfig.onclick = () => openSeedConfigModal(
        device,
        () => savedConfig,
        (newConf) => { savedConfig = newConf; }
    );

    actionContainer.append(btnRunScript, btnSeedConfig);
    tdActions.appendChild(actionContainer);
    row.appendChild(tdActions);

    tableBody.appendChild(row);
}

// ─── HELPERS ──────────────────────────────────────────────────────────────────
function _makeBtn(cls, label, onclick) {
    const btn = document.createElement('button');
    btn.className = cls;
    btn.textContent = label;
    if (onclick) btn.onclick = onclick;
    return btn;
}

// ─── SESSION BADGE ────────────────────────────────────────────────────────────
function bindBadgeEvents(toggleSwitch, switchLabel, device, row) {
    const triggerToggle = async () => {
        if (toggleSwitch.classList.contains('connecting')) return;

        // Nếu đang Inactive (Chưa Connected) -> Tiến hành kết nối
        if (!toggleSwitch.classList.contains('active')) {
            toggleSwitch.className = 'toggle-switch connecting';
            switchLabel.textContent = 'Connecting...';
            switchLabel.className = 'switch-label';
            try {
                await invoke('connect_session', { device });
                toggleSwitch.className = 'toggle-switch active';
                switchLabel.className = 'switch-label active';
                switchLabel.textContent = 'Connected';
                row.classList.remove('disabled-row');
                log(`[${device.title}] Đã kết nối thành công session mới.`, 'success');
            } catch (err) {
                toggleSwitch.className = 'toggle-switch';
                switchLabel.className = 'switch-label';
                switchLabel.textContent = 'Ready';
                log(`[${device.title}] Kết nối thất bại: ${err}`, 'error');
            }
        } 
        // Nếu đang Active (Đã Connected) -> Tiến hành ngắt kết nối
        else {
            toggleSwitch.className = 'toggle-switch connecting';
            switchLabel.textContent = 'Disconnecting...';
            switchLabel.className = 'switch-label';
            try {
                await invoke('disconnect_session', { handle: device.handle });
                toggleSwitch.className = 'toggle-switch';
                switchLabel.className = 'switch-label';
                switchLabel.textContent = 'Ready';
                row.classList.add('disabled-row');
                if (row.cleanupAutoInterval) row.cleanupAutoInterval();
                log(`[${device.title}] Đã ngắt kết nối session.`, 'info');
            } catch (err) {
                toggleSwitch.className = 'toggle-switch active';
                switchLabel.className = 'switch-label active';
                switchLabel.textContent = 'Connected';
                log(`[${device.title}] Ngắt kết nối thất bại: ${err}`, 'error');
            }
        }
    };

    toggleSwitch.onclick = triggerToggle;
}

async function updateSessionBadge(device, row) {
    const isActive = await invoke('check_session', { handle: device.handle });
    const toggleSwitch = row.querySelector('.toggle-switch');
    const switchLabel = row.querySelector('.switch-label');
    if (!toggleSwitch || !switchLabel) return;
    
    if (isActive) {
        toggleSwitch.className = 'toggle-switch active';
        switchLabel.className = 'switch-label active';
        switchLabel.textContent = 'Connected';
        row.classList.remove('disabled-row');
    } else {
        toggleSwitch.className = 'toggle-switch';
        switchLabel.className = 'switch-label';
        switchLabel.textContent = 'Ready';
        row.classList.add('disabled-row');
        if (row.cleanupAutoInterval) row.cleanupAutoInterval();
    }
    bindBadgeEvents(toggleSwitch, switchLabel, device, row);
}
