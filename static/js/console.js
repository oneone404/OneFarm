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
