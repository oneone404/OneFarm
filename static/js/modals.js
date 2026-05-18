// ─── SVG ICONS (dùng chung toàn app) ─────────────────────────────────────────
const SVG_GEAR   = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></svg>`;
const SVG_SEED   = `<svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22V12"/><path d="M12 12C12 12 7 10 5 6c3-1 6 0 7 2"/><path d="M12 12c0 0 5-2 7-6-3-1-6 0-7 2"/></svg>`;
const SVG_DOLLAR = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2v20M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>`;
const SVG_CLOSE  = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"/><line x1="6" y1="6" x2="18" y2="18"/></svg>`;

// ─── MODAL FACTORY (dùng chung cho mọi modal) ─────────────────────────────────
function _createModal(titleHTML) {
    const overlay = document.createElement('div');
    overlay.className = 'modal-overlay';

    const card = document.createElement('div');
    card.className = 'modal-card';

    const header = document.createElement('div');
    header.className = 'modal-header';

    const titleEl = document.createElement('div');
    titleEl.className = 'modal-title';
    titleEl.innerHTML = titleHTML;

    const btnClose = document.createElement('button');
    btnClose.className = 'icon-btn';
    btnClose.innerHTML = SVG_CLOSE;
    btnClose.onclick = () => _closeModal(overlay);

    header.append(titleEl, btnClose);

    const body = document.createElement('div');
    body.className = 'modal-body';

    const footer = document.createElement('div');
    footer.className = 'modal-footer';

    card.append(header, body, footer);
    overlay.appendChild(card);
    return { overlay, body, footer };
}

function _closeModal(overlay) {
    overlay.classList.remove('active');
    setTimeout(() => overlay.remove(), 300);
}

function _showModal(overlay) {
    document.body.appendChild(overlay);
    setTimeout(() => overlay.classList.add('active'), 10);
}

function _makeSection(titleText) {
    const sec = document.createElement('div');
    sec.className = 'modal-section';
    const t = document.createElement('div');
    t.className = 'modal-section-title';
    t.textContent = titleText;
    sec.appendChild(t);
    return sec;
}

function _makeNumberInput(labelText, value, min, max) {
    const group = document.createElement('div');
    group.style.cssText = 'display:flex;flex-direction:column;gap:6px';
    const label = document.createElement('div');
    label.style.cssText = 'font-size:0.65rem;font-weight:700;text-transform:uppercase;letter-spacing:0.5px;color:var(--text-secondary);text-align:center';
    label.textContent = labelText;
    const input = document.createElement('input');
    input.type = 'number';
    input.style.cssText = 'width:100%;padding:8px 10px;border-radius:6px;border:2px solid var(--border-color);background-color:var(--secondary-color);color:var(--text-primary);text-align:center;font-weight:700;font-size:0.8rem;box-sizing:border-box';
    input.min = min; input.max = max; input.value = value;
    group.append(label, input);
    return { group, input };
}

function _parseSafe(val, fallback, min) {
    const n = parseInt(val, 10);
    return isNaN(n) || n < min ? fallback : n;
}
