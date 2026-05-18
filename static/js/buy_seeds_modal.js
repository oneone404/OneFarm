// ─── SVG ICONS ───────────────────────────────────────────────────────────────
const SVG_TRASH = `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>`;
const SVG_HISTORY = `<svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"></circle><polyline points="12 6 12 12 16 14"></polyline></svg>`;

// ─── MODAL: CẤU HÌNH TỔNG HỢP (bao gồm Cài đặt chung + Chọn hạt giống + Lịch sử mua) ──
// Mở bằng: openSeedConfigModal(device, () => savedConfig, (newConf) => { ... })
async function openSeedConfigModal(device, getConfig, onSaved) {
    let cfg = getConfig();
    
    // Create modal with blank header title
    const { overlay, body, footer } = _createModal('');
    
    // Access the modal title container to inject the Tabs Header
    const headerEl = overlay.querySelector('.modal-header');
    const titleEl = headerEl.querySelector('.modal-title');
    titleEl.innerHTML = ''; // Clear default title completely
    titleEl.style.cssText = 'display:flex;gap:6px;align-items:center;flex:1;margin-right:16px;overflow-x:auto;';

    // Fetch the in-memory purchase history from backend state
    let inMemoryHistory = {};
    try {
        inMemoryHistory = await invoke('get_purchase_history');
    } catch (err) {
        console.error('Lỗi lấy lịch sử mua hạt:', err);
    }

    // ── TABS IN HEADER ───────────────────────────────────────────────────────
    // Tab 1: Cài đặt hệ thống (Mặc định active)
    const tab1 = document.createElement('div');
    tab1.className = 'modal-tab active';
    tab1.style.cssText = 'display:flex;align-items:center;justify-content:center;gap:6px;padding:6px 10px;border-radius:6px;cursor:pointer;font-size:0.75rem;font-weight:700;color:var(--accent-color);background:var(--secondary-color);border:1.5px solid var(--border-color);transition:all 0.2s;user-select:none;flex:1;text-align:center;white-space:nowrap;';
    tab1.innerHTML = `${SVG_GEAR}<span>Cài đặt chung</span>`;

    // Tab 2: Chọn hạt giống
    const tab2 = document.createElement('div');
    tab2.className = 'modal-tab';
    tab2.style.cssText = 'display:flex;align-items:center;justify-content:center;gap:6px;padding:6px 10px;border-radius:6px;cursor:pointer;font-size:0.75rem;font-weight:600;color:var(--text-secondary);background:transparent;border:1.5px solid transparent;transition:all 0.2s;user-select:none;flex:1;text-align:center;white-space:nowrap;';
    tab2.innerHTML = `${SVG_SEED}<span>Chọn hạt giống</span>`;

    // Tab 3: Lịch sử mua
    const tab3 = document.createElement('div');
    tab3.className = 'modal-tab';
    tab3.style.cssText = 'display:flex;align-items:center;justify-content:center;gap:6px;padding:6px 10px;border-radius:6px;cursor:pointer;font-size:0.75rem;font-weight:600;color:var(--text-secondary);background:transparent;border:1.5px solid transparent;transition:all 0.2s;user-select:none;flex:1;text-align:center;white-space:nowrap;';
    tab3.innerHTML = `${SVG_HISTORY}<span>Lịch sử mua</span>`;

    titleEl.append(tab1, tab2, tab3);

    // ── TABS CONTENT CONTAINER (GRID CO-LOCATION FOR HEIGHT MATCHING) ─────────
    const contentContainer = document.createElement('div');
    contentContainer.style.cssText = 'display:grid;grid-template-columns:1fr;grid-template-rows:1fr;width:100%;';
    body.appendChild(contentContainer);

    // content1: Cài đặt chung (hiển thị đầu tiên)
    const content1 = document.createElement('div');
    content1.style.cssText = 'grid-area: 1 / 1 / 2 / 2; transition: opacity 0.2s ease, visibility 0.2s; visibility: visible; opacity: 1; pointer-events: auto;';

    // content2: Chọn hạt giống (ẩn mặc định)
    const content2 = document.createElement('div');
    content2.style.cssText = 'grid-area: 1 / 1 / 2 / 2; transition: opacity 0.2s ease, visibility 0.2s; visibility: hidden; opacity: 0; pointer-events: none;';

    // content3: Lịch sử mua (ẩn mặc định)
    const content3 = document.createElement('div');
    content3.style.cssText = 'grid-area: 1 / 1 / 2 / 2; transition: opacity 0.2s ease, visibility 0.2s; visibility: hidden; opacity: 0; pointer-events: none;';

    contentContainer.append(content1, content2, content3);

    // ── TAB 1 CONTENT: CÀI ĐẶT HỆ THỐNG ──────────────────────────────────────
    const secScripts = _makeSection('Kịch bản hoạt động');
    secScripts.style.marginTop = '8px';

    const rowScripts = document.createElement('div');
    rowScripts.style.cssText = 'display:flex;gap:24px;width:100%;box-sizing:border-box;margin-top:12px;';

    const labelBuy = document.createElement('label');
    labelBuy.style.cssText = 'display:flex;align-items:center;gap:8px;font-size:0.75rem;font-weight:700;color:var(--text-primary);cursor:pointer;';
    const checkBuy = document.createElement('input');
    checkBuy.type = 'checkbox';
    checkBuy.checked = cfg.enable_buy_seeds ?? true;
    const spanBuy = document.createElement('span');
    spanBuy.textContent = 'Kích hoạt Mua hạt';
    labelBuy.append(checkBuy, spanBuy);

    const labelHarvest = document.createElement('label');
    labelHarvest.style.cssText = 'display:flex;align-items:center;gap:8px;font-size:0.75rem;font-weight:700;color:var(--text-primary);cursor:pointer;';
    const checkHarvest = document.createElement('input');
    checkHarvest.type = 'checkbox';
    checkHarvest.checked = cfg.enable_harvest_sell ?? true;
    const spanHarvest = document.createElement('span');
    spanHarvest.textContent = 'Kích hoạt Thu hoạch & Bán';
    labelHarvest.append(checkHarvest, spanHarvest);

    const labelAutoLogin = document.createElement('label');
    labelAutoLogin.style.cssText = 'display:flex;align-items:center;gap:8px;font-size:0.75rem;font-weight:700;color:var(--text-primary);cursor:pointer;';
    const checkAutoLogin = document.createElement('input');
    checkAutoLogin.type = 'checkbox';
    checkAutoLogin.checked = cfg.enable_auto_login ?? true;
    const spanAutoLogin = document.createElement('span');
    spanAutoLogin.textContent = 'Tự động Đăng nhập';
    labelAutoLogin.append(checkAutoLogin, spanAutoLogin);

    rowScripts.append(labelBuy, labelHarvest, labelAutoLogin);
    secScripts.appendChild(rowScripts);
    content1.appendChild(secScripts);

    const secSettings = _makeSection('Tham số nhận diện & điều khiển');
    secSettings.style.marginTop = '14px';

    const gridParams = document.createElement('div');
    gridParams.style.cssText = 'display:grid;grid-template-columns:repeat(4,1fr);gap:12px;width:100%;box-sizing:border-box;margin-top:12px';

    const timeout     = _makeNumberInput('Timeout (s)',    cfg.button_timeout_secs ?? 5,    1,   60);
    const delay       = _makeNumberInput('Delay nút (ms)', cfg.click_delay_ms      ?? 1000, 100, 10000);
    const threshold   = _makeNumberInput('Độ nhạy khớp',  cfg.match_threshold     ?? 25,   10,  50);
    const launchDelay = _makeNumberInput('Chờ mở game (s)', cfg.game_launch_delay_secs ?? 60, 5, 300);

    gridParams.append(timeout.group, delay.group, threshold.group, launchDelay.group);
    secSettings.appendChild(gridParams);
    content1.appendChild(secSettings);

    // ── CÀI ĐẶT THU HOẠCH & BÁN ─────────────────────────────────────────────
    const secHarvest = _makeSection('Cấu hình Thu hoạch & Bán');
    secHarvest.style.marginTop = '14px';

    const gridHarvest = document.createElement('div');
    gridHarvest.style.cssText = 'display:grid;grid-template-columns:repeat(3,1fr);gap:12px;width:100%;box-sizing:border-box;margin-top:12px';

    const harvestInterval = _makeNumberInput('Chu kỳ thu hoạch (phút)', cfg.harvest_interval_mins ?? 30, 1, 1440);
    const harvestLoop     = _makeNumberInput('Lượt lặp thu hoạch',      cfg.harvest_loop_count     ?? 2,  1, 100);
    const sellLoop        = _makeNumberInput('Lượt lặp bán',            cfg.sell_loop_count        ?? 2,  1, 100);

    gridHarvest.append(harvestInterval.group, harvestLoop.group, sellLoop.group);
    secHarvest.appendChild(gridHarvest);
    content1.appendChild(secHarvest);

    // ── TAB 2 CONTENT: CHỌN HẠT GIỐNG ────────────────────────────────────────
    const gridSeeds = document.createElement('div');
    gridSeeds.className = 'seeds-grid';
    gridSeeds.style.marginTop = '8px';

    // Exclude digit templates (starting with seeds/digits/) from the seed selection grid
    const seedTemplates = availableTemplates.filter(t => t.startsWith('seeds/') && !t.startsWith('seeds/digits/'));
    seedTemplates.forEach(t => {
        const name  = t.replace('seeds/', '').replace('.png', '');
        const label = document.createElement('label');
        label.className = 'seed-checkbox-label';
        const input = document.createElement('input');
        input.type    = 'checkbox';
        input.value   = t;
        input.checked = !!(cfg.selected_seeds && cfg.selected_seeds.includes(t));
        const span = document.createElement('span');
        span.textContent = name;
        label.append(input, span);
        gridSeeds.appendChild(label);
    });

    content2.appendChild(gridSeeds);

    // ── TAB 3 CONTENT: LỊCH SỬ MUA ───────────────────────────────────────────
    const historyContainer = document.createElement('div');
    historyContainer.style.cssText = 'display:flex;flex-direction:column;gap:6px;margin-top:8px;';
    content3.appendChild(historyContainer);

    // Render History Function
    const renderHistory = (updatedHistory) => {
        const activeHistory = updatedHistory || inMemoryHistory;
        historyContainer.innerHTML = '';
        const entries = Object.entries(activeHistory);

        if (entries.length === 0) {
            const emptyEl = document.createElement('div');
            emptyEl.style.cssText = 'font-size:0.75rem;font-weight:600;color:var(--text-secondary);text-align:center;padding:24px 8px;border:1.5px dashed var(--border-color);border-radius:6px;background:var(--secondary-color);line-height:1.4;';
            emptyEl.textContent = 'Chưa mua hạt giống nào trong phiên làm việc này. Hãy khởi chạy kịch bản mua để tích lũy thống kê!';
            historyContainer.appendChild(emptyEl);
        } else {
            // Sort entries alphabetically
            entries.sort((a, b) => a[0].localeCompare(b[0]));
            entries.forEach(([seed, qty]) => {
                const name = seed.replace('seeds/', '').replace('.png', '');
                
                const item = document.createElement('div');
                item.style.cssText = 'display:flex;justify-content:space-between;align-items:center;padding:8px 12px;border:1.5px solid var(--border-color);border-radius:6px;background:var(--secondary-color);transition:all 0.2s;';
                
                const left = document.createElement('div');
                left.style.cssText = 'display:flex;align-items:center;gap:8px;font-size:0.75rem;font-weight:700;color:var(--text-primary);text-transform:capitalize;';
                left.innerHTML = `${SVG_SEED}<span>${name}</span>`;

                const right = document.createElement('div');
                right.style.cssText = 'background:var(--accent-color);color:#ffffff;padding:3px 10px;border-radius:12px;font-size:0.7rem;font-weight:700;letter-spacing:0.5px;';
                right.textContent = `${qty} hạt`;

                item.append(left, right);
                historyContainer.appendChild(item);
            });

            // Action row inside history tab
            const actionRow = document.createElement('div');
            actionRow.style.cssText = 'display:flex;justify-content:flex-end;margin-top:12px;';

            const btnClear = document.createElement('button');
            btnClear.style.cssText = 'display:flex;align-items:center;gap:6px;font-size:0.7rem;font-weight:700;padding:5px 12px;border-radius:4px;border:2px solid #ef4444;background:transparent;color:#ef4444;cursor:pointer;transition:all 0.2s ease-in-out;';
            btnClear.innerHTML = `${SVG_TRASH}<span>Xóa lịch sử</span>`;
            
            // Hover effect
            btnClear.onmouseenter = () => {
                btnClear.style.background = '#ef4444';
                btnClear.style.color = '#ffffff';
            };
            btnClear.onmouseleave = () => {
                btnClear.style.background = 'transparent';
                btnClear.style.color = '#ef4444';
            };

            btnClear.onclick = async () => {
                const confirmClear = confirm('Bạn có chắc muốn đặt lại lịch sử mua hạt giống của phiên này?');
                if (!confirmClear) return;
                
                try {
                    await invoke('clear_purchase_history');
                    inMemoryHistory = {}; // Reset local reference
                    log(`[${device.title}] Đã đặt lại lịch sử mua hạt giống!`, 'success');
                    renderHistory({});
                } catch (err) {
                    log(`[${device.title}] Lỗi đặt lại lịch sử: ${err}`, 'error');
                }
            };

            actionRow.appendChild(btnClear);
            historyContainer.appendChild(actionRow);
        }
    };

    // ── TAB SWITCHING LOGIC ──────────────────────────────────────────────────
    tab1.onclick = () => {
        // Tab 1 Active - Pill styling
        tab1.style.color = 'var(--accent-color)';
        tab1.style.background = 'var(--secondary-color)';
        tab1.style.borderColor = 'var(--border-color)';
        tab1.style.fontWeight = '700';

        tab2.style.color = 'var(--text-secondary)';
        tab2.style.background = 'transparent';
        tab2.style.borderColor = 'transparent';
        tab2.style.fontWeight = '600';

        tab3.style.color = 'var(--text-secondary)';
        tab3.style.background = 'transparent';
        tab3.style.borderColor = 'transparent';
        tab3.style.fontWeight = '600';

        // Animate visibility and opacity for stable container layout height
        content1.style.visibility = 'visible';
        content1.style.opacity = '1';
        content1.style.pointerEvents = 'auto';

        content2.style.visibility = 'hidden';
        content2.style.opacity = '0';
        content2.style.pointerEvents = 'none';

        content3.style.visibility = 'hidden';
        content3.style.opacity = '0';
        content3.style.pointerEvents = 'none';
    };

    tab2.onclick = () => {
        // Tab 2 Active - Pill styling
        tab2.style.color = 'var(--accent-color)';
        tab2.style.background = 'var(--secondary-color)';
        tab2.style.borderColor = 'var(--border-color)';
        tab2.style.fontWeight = '700';

        tab1.style.color = 'var(--text-secondary)';
        tab1.style.background = 'transparent';
        tab1.style.borderColor = 'transparent';
        tab1.style.fontWeight = '600';

        tab3.style.color = 'var(--text-secondary)';
        tab3.style.background = 'transparent';
        tab3.style.borderColor = 'transparent';
        tab3.style.fontWeight = '600';

        // Animate visibility and opacity for stable container layout height
        content2.style.visibility = 'visible';
        content2.style.opacity = '1';
        content2.style.pointerEvents = 'auto';

        content1.style.visibility = 'hidden';
        content1.style.opacity = '0';
        content1.style.pointerEvents = 'none';

        content3.style.visibility = 'hidden';
        content3.style.opacity = '0';
        content3.style.pointerEvents = 'none';
    };

    tab3.onclick = async () => {
        // Tab 3 Active - Pill styling
        tab3.style.color = 'var(--accent-color)';
        tab3.style.background = 'var(--secondary-color)';
        tab3.style.borderColor = 'var(--border-color)';
        tab3.style.fontWeight = '700';

        tab1.style.color = 'var(--text-secondary)';
        tab1.style.background = 'transparent';
        tab1.style.borderColor = 'transparent';
        tab1.style.fontWeight = '600';

        tab2.style.color = 'var(--text-secondary)';
        tab2.style.background = 'transparent';
        tab2.style.borderColor = 'transparent';
        tab2.style.fontWeight = '600';

        // Animate visibility and opacity for stable container layout height
        content3.style.visibility = 'visible';
        content3.style.opacity = '1';
        content3.style.pointerEvents = 'auto';

        content1.style.visibility = 'hidden';
        content1.style.opacity = '0';
        content1.style.pointerEvents = 'none';

        content2.style.visibility = 'hidden';
        content2.style.opacity = '0';
        content2.style.pointerEvents = 'none';

        // Fetch fresh history and render
        try {
            inMemoryHistory = await invoke('get_purchase_history');
        } catch (err) {
            console.error('Lỗi lấy lịch sử mua hạt:', err);
        }
        renderHistory();
    };

    // ── FOOTER: BUTTONS ──────────────────────────────────────────────────────
    footer.style.cssText = 'display:flex;gap:8px;justify-content:flex-end;align-items:center;';

    const btnCancel = document.createElement('button');
    btnCancel.style.cssText = 'font-size:0.75rem;font-weight:700;padding:8px 16px;border-radius:6px;border:2px solid var(--border-color);background:transparent;color:var(--text-secondary);cursor:pointer;transition:all 0.2s;';
    btnCancel.textContent = 'Đóng';
    btnCancel.onmouseenter = () => {
        btnCancel.style.color = 'var(--text-primary)';
        btnCancel.style.borderColor = 'var(--text-secondary)';
    };
    btnCancel.onmouseleave = () => {
        btnCancel.style.color = 'var(--text-secondary)';
        btnCancel.style.borderColor = 'var(--border-color)';
    };
    btnCancel.onclick = () => _closeModal(overlay);

    const btnSave = document.createElement('button');
    btnSave.className = 'accent-btn';
    btnSave.style.cssText = 'font-size:0.75rem;font-weight:700;padding:8px 16px;border-radius:6px;border:none;cursor:pointer;transition:all 0.2s;';
    btnSave.textContent = 'Lưu thiết lập';
    btnSave.onclick = async () => {
        const selected = Array.from(gridSeeds.querySelectorAll('input:checked')).map(i => i.value);
        const newConf  = { 
            ...cfg, 
            selected_seeds: selected,
            button_timeout_secs:   _parseSafe(timeout.input.value,   5,    1),
            click_delay_ms:         _parseSafe(delay.input.value,     1000, 100),
            match_threshold:        _parseSafe(threshold.input.value, 25,   10),
            game_launch_delay_secs: _parseSafe(launchDelay.input.value, 60,   5),
            harvest_interval_mins:  _parseSafe(harvestInterval.input.value, 30, 1),
            harvest_loop_count:     _parseSafe(harvestLoop.input.value,     2,  1),
            sell_loop_count:        _parseSafe(sellLoop.input.value,        2,  1),
            enable_buy_seeds:       checkBuy.checked,
            enable_harvest_sell:    checkHarvest.checked,
            enable_auto_login:      checkAutoLogin.checked,
        };
        try {
            await invoke('save_config', { config: newConf });
            onSaved(newConf);
            log(`[${device.title}] Đã lưu thiết lập cấu hình tổng hợp!`, 'success');
            _closeModal(overlay);
        } catch (err) {
            log(`[${device.title}] Lỗi lưu cấu hình: ${err}`, 'error');
        }
    };
    
    footer.append(btnCancel, btnSave);
    _showModal(overlay);
}
