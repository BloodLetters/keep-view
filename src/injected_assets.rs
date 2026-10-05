pub const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36";

pub const WIDGET_SCRIPT: &str = r##"
(function() {
    if (window.__KEEP_WIDGET_INITIALIZED__) return;
    window.__KEEP_WIDGET_INITIALIZED__ = true;

    // --- State ---
    let isAlwaysOnTop = false;
    let currentZoom = 1.0;
    let isOnline = navigator.onLine;

    // --- Create and Inject Floating Minimal Widget Pill ---
    function initPill() {
        if (document.getElementById('keep-widget-pill')) return;

        const pill = document.createElement('div');
        pill.id = 'keep-widget-pill';
        pill.innerHTML = `
            <div class="kw-pill-collapsed">
                <span class="kw-status-dot ${isOnline ? 'online' : 'offline'}" id="kw-status-dot" title="${isOnline ? 'Online (Tersinkron)' : 'Offline (Tersimpan Lokal)'}"></span>
                <button class="kw-pill-btn ${isAlwaysOnTop ? 'active' : ''}" id="kw-pill-pin" title="Pin Widget (Always on Top)">📌</button>
            </div>
            <div class="kw-pill-expanded">
                <span class="kw-pill-text" id="kw-pill-status">${isOnline ? 'Online' : 'Offline'}</span>
                <span class="kw-sep">|</span>
                <button class="kw-pill-btn" id="kw-pill-drag" title="Geser / Pindahkan Widget">✥</button>
                <button class="kw-pill-btn" id="kw-btn-search" title="Cari Catatan (Ctrl+F)">🔍</button>
                <button class="kw-pill-btn" id="kw-zoom-out" title="Perkecil Zoom">-</button>
                <span class="kw-zoom-label" id="kw-zoom-val">100%</span>
                <button class="kw-pill-btn" id="kw-zoom-in" title="Perbesar Zoom">+</button>
                <span class="kw-sep">|</span>
                <button class="kw-pill-btn" id="kw-btn-sync" title="Sinkronkan / Muat Ulang">🔄</button>
                <button class="kw-pill-btn" id="kw-btn-help" title="Info & Bantuan">ℹ️</button>
            </div>
        `;

        // Floating search input bar
        const searchBar = document.createElement('div');
        searchBar.id = 'keep-floating-search';
        searchBar.innerHTML = `
            <input type="text" id="keep-search-input" placeholder="Cari catatan..." autocomplete="off">
            <button id="keep-search-close" title="Tutup Pencarian">&times;</button>
        `;
        searchBar.style.display = 'none';

        // Floating Top Drag Bar for frameless window
        if (!document.getElementById('keep-window-drag-bar')) {
            const dragBar = document.createElement('div');
            dragBar.id = 'keep-window-drag-bar';
            dragBar.title = 'Drag untuk memindahkan jendela widget';
            document.documentElement.appendChild(dragBar);

            dragBar.addEventListener('mousedown', (e) => {
                if (e.button === 0) {
                    sendIpc({ type: 'drag_window' });
                }
            });
        }

        document.documentElement.appendChild(pill);
        document.documentElement.appendChild(searchBar);

        // --- Event Listeners for Pill ---
        const pinBtn = document.getElementById('kw-pill-pin');
        pinBtn.addEventListener('click', (e) => {
            e.stopPropagation();
            sendIpc({ type: 'toggle_pin' });
        });

        const dragBtn = document.getElementById('kw-pill-drag');
        if (dragBtn) {
            dragBtn.addEventListener('mousedown', (e) => {
                if (e.button === 0) {
                    e.stopPropagation();
                    sendIpc({ type: 'drag_window' });
                }
            });
        }


        document.getElementById('kw-btn-sync').addEventListener('click', (e) => {
            e.stopPropagation();
            triggerSync();
        });

        document.getElementById('kw-zoom-in').addEventListener('click', (e) => {
            e.stopPropagation();
            adjustZoom(0.05);
        });

        document.getElementById('kw-zoom-out').addEventListener('click', (e) => {
            e.stopPropagation();
            adjustZoom(-0.05);
        });

        document.getElementById('kw-btn-help').addEventListener('click', (e) => {
            e.stopPropagation();
            showHelpModal();
        });

        // Search bar toggle
        document.getElementById('kw-btn-search').addEventListener('click', (e) => {
            e.stopPropagation();
            toggleSearchBar();
        });

        document.getElementById('keep-search-close').addEventListener('click', (e) => {
            e.stopPropagation();
            toggleSearchBar(false);
        });

        const searchInput = document.getElementById('keep-search-input');
        searchInput.addEventListener('input', (e) => {
            const query = e.target.value;
            applySearchQuery(query);
        });

        searchInput.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                toggleSearchBar(false);
            }
        });

        // Drag window from empty background area
        document.addEventListener('mousedown', (e) => {
            if (e.button === 0 && (e.target === document.documentElement || e.target === document.body)) {
                sendIpc({ type: 'drag_window' });
            }
        });
    }

    // --- Clean and Safe Sidebar Rail Collapse ---
    function hideSidebarRail() {
        const nav = document.querySelector('[role="navigation"], [class*="PvRhvb"], nav');
        if (!nav) return;

        nav.style.setProperty('display', 'none', 'important');

        // Check if there is an outer rail wrapper (must be left-aligned narrow element not containing main)
        const main = document.querySelector('div[role="main"], main');
        let parent = nav.parentElement;
        while (parent && parent !== document.body && (!main || !parent.contains(main))) {
            const rect = parent.getBoundingClientRect();
            if (rect.width > 0 && rect.width <= 320 && rect.left < 50) {
                parent.style.setProperty('display', 'none', 'important');
                break;
            }
            parent = parent.parentElement;
        }
    }

    // --- Search Helper ---
    function toggleSearchBar(force) {
        const bar = document.getElementById('keep-floating-search');
        if (!bar) return;
        const shouldShow = force !== undefined ? force : bar.style.display === 'none';
        bar.style.display = shouldShow ? 'flex' : 'none';
        if (shouldShow) {
            const input = document.getElementById('keep-search-input');
            if (input) input.focus();
        } else {
            applySearchQuery('');
            const input = document.getElementById('keep-search-input');
            if (input) input.value = '';
        }
    }

    function applySearchQuery(query) {
        const nativeInput = document.querySelector('header input[role="combobox"], input[name="q"], input[aria-label*="Search"], input[aria-label*="Telusuri"]');
        if (nativeInput) {
            nativeInput.value = query;
            nativeInput.dispatchEvent(new Event('input', { bubbles: true }));
            nativeInput.dispatchEvent(new Event('change', { bubbles: true }));
            return;
        }

        const cards = document.querySelectorAll('div[role="main"] div[role="button"][tabindex="0"]');
        const q = query.toLowerCase().trim();
        cards.forEach(card => {
            if (!q) {
                card.style.display = '';
            } else {
                const text = card.textContent.toLowerCase();
                card.style.display = text.includes(q) ? '' : 'none';
            }
        });
    }

    // --- Zoom Management ---
    function adjustZoom(delta) {
        currentZoom = Math.min(Math.max(currentZoom + delta, 0.7), 1.5);
        applyZoom();
        sendIpc({ type: 'set_zoom', value: currentZoom });
    }

    function applyZoom() {
        if (Math.abs(currentZoom - 1.0) < 0.01) {
            document.body.style.removeProperty('zoom');
        } else {
            document.body.style.zoom = currentZoom;
        }
        const valElem = document.getElementById('kw-zoom-val');
        if (valElem) {
            valElem.textContent = Math.round(currentZoom * 100) + '%';
        }
    }

    // --- Online/Offline & Sync Tracking ---
    function updateStatus(status, text) {
        const dot = document.getElementById('kw-status-dot');
        const label = document.getElementById('kw-pill-status');
        if (dot) {
            dot.className = 'kw-status-dot ' + status;
            dot.title = text;
        }
        if (label) {
            label.textContent = text;
        }

        sendIpc({ type: 'status_changed', status: status, text: text });
    }

    function triggerSync() {
        const btn = document.getElementById('kw-btn-sync');
        if (btn) btn.classList.add('kw-spin');
        updateStatus('syncing', 'Menyinkronkan...');

        fetch('https://keep.google.com/u/0/', { method: 'HEAD', cache: 'no-store' })
            .then(() => {
                setTimeout(() => {
                    if (btn) btn.classList.remove('kw-spin');
                    updateStatus('online', 'Tersinkron');
                }, 800);
            })
            .catch(() => {
                setTimeout(() => {
                    if (btn) btn.classList.remove('kw-spin');
                    updateStatus('offline', 'Offline');
                }, 800);
            });
    }

    window.addEventListener('online', () => {
        isOnline = true;
        updateStatus('syncing', 'Reconnecting...');
        setTimeout(() => {
            updateStatus('online', 'Online');
        }, 1200);
    });

    window.addEventListener('offline', () => {
        isOnline = false;
        updateStatus('offline', 'Offline (Lokal)');
    });

    setInterval(() => {
        if (!navigator.onLine) {
            if (isOnline) {
                isOnline = false;
                updateStatus('offline', 'Offline (Lokal)');
            }
            return;
        }
        fetch('https://www.google.com/generate_204', { mode: 'no-cors', cache: 'no-store' })
            .then(() => {
                if (!isOnline) {
                    isOnline = true;
                    updateStatus('online', 'Online');
                }
            })
            .catch(() => {
                if (isOnline) {
                    isOnline = false;
                    updateStatus('offline', 'Offline (Lokal)');
                }
            });
    }, 15000);

    // --- Hotkeys ---
    document.addEventListener('keydown', (e) => {
        if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') {
            e.preventDefault();
            toggleSearchBar(true);
        } else if (e.key === 'Escape') {
            toggleSearchBar(false);
            let existing = document.getElementById('kw-help-modal');
            if (existing) existing.remove();
        }
    });

    // --- Help Modal ---
    function showHelpModal() {
        let existing = document.getElementById('kw-help-modal');
        if (existing) {
            existing.remove();
            return;
        }

        const modal = document.createElement('div');
        modal.id = 'kw-help-modal';
        modal.innerHTML = `
            <div class="kw-modal-box">
                <div class="kw-modal-header">
                    <h3>💡 Google Keep Widget Desktop</h3>
                    <button class="kw-modal-close" id="kw-modal-close-btn">&times;</button>
                </div>
                <div class="kw-modal-body">
                    <p><strong>Tampilan Bersih (Clean Widget):</strong></p>
                    <ul>
                        <li>Header Google dan menu navigasi samping otomatis disembunyikan agar Anda fokus pada catatan.</li>
                        <li>Gunakan tombol di pojok kanan bawah untuk Pin, Zoom, Pencarian, dan Sinkronisasi.</li>
                    </ul>
                    <p><strong>Pintasan Keyboard (Hotkeys):</strong></p>
                    <ul>
                        <li><kbd>Alt + Shift + K</kbd> : Buka / Sembunyikan Widget dari mana saja</li>
                        <li><kbd>Ctrl + F</kbd> : Cari catatan cepat</li>
                        <li><kbd>Ctrl + R</kbd> / <kbd>F5</kbd> : Segarkan & Sinkronkan</li>
                        <li><kbd>Esc</kbd> : Tutup pencarian / jendela bantuan</li>
                    </ul>
                    <p><strong>Mode Offline:</strong></p>
                    <ul>
                        <li>Catatan Anda tersimpan di disk lokal. Begitu online kembali, Google Keep otomatis menyinkronkan seluruh perubahan.</li>
                    </ul>
                </div>
            </div>
        `;
        document.documentElement.appendChild(modal);

        document.getElementById('kw-modal-close-btn').addEventListener('click', () => {
            modal.remove();
        });
        modal.addEventListener('click', (e) => {
            if (e.target === modal) modal.remove();
        });
    }

    // --- Rust IPC Helper ---
    function sendIpc(data) {
        const payload = JSON.stringify(data);
        if (window.ipc && window.ipc.postMessage) {
            window.ipc.postMessage(payload);
        } else if (window.chrome && window.chrome.webview && window.chrome.webview.postMessage) {
            window.chrome.webview.postMessage(payload);
        }
    }

    // --- Messages from Rust to JS ---
    window.__KEEP_SET_PIN__ = function(pinned) {
        isAlwaysOnTop = pinned;
        const btn = document.getElementById('kw-pill-pin');
        if (btn) {
            if (pinned) {
                btn.classList.add('active');
                btn.title = 'Pin Aktif (Widget selalu di atas)';
            } else {
                btn.classList.remove('active');
                btn.title = 'Pin Widget (Always on Top)';
            }
        }
    };

    window.__KEEP_SET_ZOOM__ = function(zoom) {
        currentZoom = zoom;
        applyZoom();
    };

    window.__KEEP_SET_COMPACT__ = function(_compact) {
        // Tampilan catatan tengah selalu rapi dan terpusat
    };

    // --- Inject Styles ---
    function injectStyles() {
        if (document.getElementById('keep-widget-custom-styles')) return;

        const style = document.createElement('style');
        style.id = 'keep-widget-custom-styles';
        style.textContent = `
            /* ========================================================
               1. HIDE TOP BAR (HEADER) & SIDEBAR NAVIGATION
               ======================================================== */
            header,
            #gb,
            div[role="banner"],
            [role="navigation"],
            [class*="PvRhvb"],
            nav,
            aside {
                display: none !important;
            }

            /* ========================================================
               2. FOCUS CENTER VIEW (MAIN NOTES CONTAINER)
               ======================================================== */
            html, body {
                padding: 0 !important;
                margin: 0 !important;
                background-color: #202124 !important;
            }

            div[role="main"],
            main {
                position: relative !important;
                margin-left: auto !important;
                margin-right: auto !important;
                margin-top: 8px !important;
                padding: 8px 16px 64px 16px !important;
                width: 100% !important;
                max-width: 100% !important;
                box-sizing: border-box !important;
            }

            /* ========================================================
               3. NOTE CREATION BAR & ACTION BUTTONS PRESERVATION
               ======================================================== */
            div[role="main"] [role="region"] {
                margin-top: 6px !important;
                margin-bottom: 16px !important;
                max-width: 100% !important;
            }

            /* Pinned / Others Section Headers */
            div[role="heading"] {
                font-size: 11px !important;
                font-weight: 600 !important;
                letter-spacing: 0.8px !important;
                color: #9aa0a6 !important;
                margin: 12px 4px 6px 4px !important;
                text-transform: uppercase !important;
            }

            /* ========================================================
               4. HIDE SCROLLBAR / SLIDER COMPLETELY
               ======================================================== */
            ::-webkit-scrollbar {
                display: none !important;
                width: 0 !important;
                height: 0 !important;
            }
            * {
                -ms-overflow-style: none !important;
                scrollbar-width: none !important;
            }

            /* ========================================================
               5. TOP WINDOW DRAG BAR (FRAMELESS WIDGET)
               ======================================================== */
            #keep-window-drag-bar {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                height: 14px;
                z-index: 2147483640;
                cursor: grab;
                background: transparent;
            }
            #keep-window-drag-bar:active {
                cursor: grabbing;
            }

            /* ========================================================
               5. MINIMAL FLOATING CONTROL PILL (CORNER WIDGET)
               ======================================================== */
            #keep-widget-pill {
                position: fixed;
                bottom: 12px;
                right: 12px;
                z-index: 2147483647;
                background: rgba(32, 33, 36, 0.7);
                backdrop-filter: blur(16px);
                -webkit-backdrop-filter: blur(16px);
                border: 1px solid rgba(255, 255, 255, 0.12);
                border-radius: 20px;
                padding: 4px 8px;
                display: flex;
                align-items: center;
                gap: 6px;
                box-shadow: 0 4px 16px rgba(0, 0, 0, 0.45);
                opacity: 0.35;
                transition: all 0.25s cubic-bezier(0.16, 1, 0.3, 1);
                user-select: none;
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
                font-size: 11px;
                color: #e8eaed;
            }

            #keep-widget-pill:hover,
            #keep-widget-pill:focus-within {
                opacity: 1;
                background: rgba(32, 33, 36, 0.95);
                border-color: rgba(251, 188, 4, 0.5);
                box-shadow: 0 6px 24px rgba(0, 0, 0, 0.65);
                transform: translateY(-2px);
            }

            .kw-pill-collapsed {
                display: flex;
                align-items: center;
                gap: 6px;
            }

            .kw-pill-expanded {
                display: none;
                align-items: center;
                gap: 5px;
            }

            #keep-widget-pill:hover .kw-pill-expanded {
                display: flex;
            }

            .kw-status-dot {
                width: 7px;
                height: 7px;
                border-radius: 50%;
                display: inline-block;
            }
            .kw-status-dot.online {
                background-color: #34c759;
                box-shadow: 0 0 6px rgba(52, 199, 89, 0.6);
            }
            .kw-status-dot.offline {
                background-color: #ff9500;
                box-shadow: 0 0 6px rgba(255, 149, 0, 0.6);
            }
            .kw-status-dot.syncing {
                background-color: #0a84ff;
                animation: kwPulse 1s infinite alternate;
            }

            @keyframes kwPulse {
                0% { opacity: 0.4; }
                100% { opacity: 1; }
            }

            .kw-pill-btn {
                background: rgba(255, 255, 255, 0.08);
                border: 1px solid rgba(255, 255, 255, 0.1);
                color: #e8eaed;
                padding: 3px 6px;
                border-radius: 6px;
                cursor: pointer;
                font-size: 11px;
                line-height: 1;
                transition: all 0.15s ease;
                display: flex;
                align-items: center;
                justify-content: center;
            }
            .kw-pill-btn:hover {
                background: rgba(255, 255, 255, 0.2);
                border-color: rgba(255, 255, 255, 0.3);
            }
            .kw-pill-btn:active {
                transform: scale(0.95);
            }
            .kw-pill-btn.active {
                background: #fbbc04;
                color: #121212;
                border-color: #fbbc04;
                font-weight: bold;
            }

            .kw-sep {
                color: rgba(255, 255, 255, 0.2);
                font-size: 10px;
            }

            .kw-zoom-label {
                font-size: 10px;
                color: #9aa0a6;
                min-width: 28px;
                text-align: center;
            }

            .kw-spin {
                animation: kwSpinAnim 1s linear infinite;
            }
            @keyframes kwSpinAnim {
                100% { transform: rotate(360deg); }
            }

            /* ========================================================
               6. FLOATING MINIMAL SEARCH BAR (CTRL+F)
               ======================================================== */
            #keep-floating-search {
                position: fixed;
                top: 10px;
                left: 12px;
                right: 12px;
                z-index: 2147483646;
                background: #2d2e31;
                border: 1px solid rgba(255, 255, 255, 0.18);
                border-radius: 8px;
                padding: 6px 12px;
                display: flex;
                align-items: center;
                gap: 8px;
                box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
            }
            #keep-search-input {
                flex: 1;
                background: transparent;
                border: none;
                color: #ffffff;
                font-size: 13px;
                outline: none;
                font-family: inherit;
            }
            #keep-search-close {
                background: transparent;
                border: none;
                color: #9aa0a6;
                font-size: 18px;
                cursor: pointer;
                line-height: 1;
            }
            #keep-search-close:hover {
                color: #ffffff;
            }

            /* ========================================================
               7. MODERN HELP MODAL
               ======================================================== */
            #kw-help-modal {
                position: fixed;
                top: 0;
                left: 0;
                right: 0;
                bottom: 0;
                background: rgba(0, 0, 0, 0.65);
                backdrop-filter: blur(8px);
                display: flex;
                align-items: center;
                justify-content: center;
                z-index: 2147483647;
                padding: 16px;
            }
            .kw-modal-box {
                background: #252528;
                color: #f2f2f7;
                border: 1px solid rgba(255, 255, 255, 0.15);
                border-radius: 12px;
                width: 100%;
                max-width: 380px;
                box-shadow: 0 16px 32px rgba(0, 0, 0, 0.45);
                overflow: hidden;
                font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            }
            .kw-modal-header {
                display: flex;
                align-items: center;
                justify-content: space-between;
                padding: 12px 16px;
                background: rgba(255, 255, 255, 0.05);
                border-bottom: 1px solid rgba(255, 255, 255, 0.1);
            }
            .kw-modal-header h3 {
                margin: 0;
                font-size: 14px;
                font-weight: 600;
                color: #fbbc04;
            }
            .kw-modal-close {
                background: none;
                border: none;
                color: #a1a1a6;
                font-size: 20px;
                cursor: pointer;
            }
            .kw-modal-close:hover {
                color: #ffffff;
            }
            .kw-modal-body {
                padding: 14px 16px;
                font-size: 12px;
                line-height: 1.5;
                color: #d1d1d6;
            }
            .kw-modal-body p {
                margin: 8px 0 4px 0;
            }
            .kw-modal-body ul {
                margin: 4px 0 10px 0;
                padding-left: 18px;
            }
            .kw-modal-body kbd {
                background: rgba(255, 255, 255, 0.12);
                border: 1px solid rgba(255, 255, 255, 0.2);
                border-radius: 4px;
                padding: 2px 5px;
                font-size: 11px;
                font-family: monospace;
                color: #ffffff;
            }
        `;
        document.head.appendChild(style);
    }

    function init() {
        injectStyles();
        initPill();
        hideSidebarRail();

        // Safely observe DOM to collapse sidebar rail when lazily loaded
        let sidebarTimer = null;
        const observer = new MutationObserver(() => {
            if (sidebarTimer) clearTimeout(sidebarTimer);
            sidebarTimer = setTimeout(hideSidebarRail, 150);
        });
        observer.observe(document.body || document.documentElement, {
            childList: true,
        });

        // Disconnect observer after 8s so it never causes background churn or interference during note dragging
        setTimeout(() => {
            hideSidebarRail();
            observer.disconnect();
        }, 8000);
    }

    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', init);
    } else {
        init();
    }
})();
"##;

pub const OFFLINE_FALLBACK_HTML: &str = r##"<!DOCTYPE html>
<html lang="id">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Google Keep Widget - Mode Offline</title>
    <style>
        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
            background: #1e1e24;
            color: #f2f2f7;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            min-height: 100vh;
            padding: 24px;
            text-align: center;
        }
        .card {
            background: #282830;
            border: 1px solid rgba(255, 255, 255, 0.1);
            border-radius: 16px;
            padding: 32px 24px;
            max-width: 360px;
            box-shadow: 0 12px 30px rgba(0, 0, 0, 0.35);
        }
        .icon {
            font-size: 48px;
            margin-bottom: 16px;
        }
        h2 {
            font-size: 18px;
            font-weight: 600;
            color: #fbbc04;
            margin-bottom: 10px;
        }
        p {
            font-size: 13px;
            color: #a1a1a6;
            line-height: 1.5;
            margin-bottom: 20px;
        }
        .badge {
            display: inline-flex;
            align-items: center;
            gap: 6px;
            background: rgba(255, 149, 0, 0.15);
            color: #ff9500;
            border: 1px solid rgba(255, 149, 0, 0.3);
            border-radius: 20px;
            padding: 4px 12px;
            font-size: 12px;
            font-weight: 500;
            margin-bottom: 20px;
        }
        .pulse {
            width: 8px;
            height: 8px;
            background: #ff9500;
            border-radius: 50%;
            animation: pulseAnim 1.5s infinite;
        }
        @keyframes pulseAnim {
            0% { transform: scale(0.9); opacity: 0.7; }
            50% { transform: scale(1.3); opacity: 1; }
            100% { transform: scale(0.9); opacity: 0.7; }
        }
        .btn {
            background: #fbbc04;
            color: #1a1a1a;
            border: none;
            border-radius: 8px;
            padding: 10px 20px;
            font-size: 13px;
            font-weight: 600;
            cursor: pointer;
            transition: all 0.2s ease;
            width: 100%;
        }
        .btn:hover {
            background: #e0a700;
            transform: translateY(-1px);
        }
        .btn:active {
            transform: translateY(1px);
        }
        .info {
            margin-top: 18px;
            font-size: 11px;
            color: #6e6e73;
        }
    </style>
</head>
<body>
    <div class="card">
        <div class="icon">📴</div>
        <div class="badge">
            <span class="pulse"></span>
            <span>Menunggu Koneksi</span>
        </div>
        <h2>Koneksi Internet Terputus</h2>
        <p>Aplikasi widget ini akan otomatis memuat dan menyinkronkan Google Keep begitu komputer Anda terhubung kembali ke internet.</p>
        <button class="btn" id="retry-btn">Coba Hubungkan Ulang</button>
        <div class="info">Catatan yang sebelumnya sudah dibuka tetap tersimpan aman di disk lokal PC Anda.</div>
    </div>

    <script>
        function checkAndReload() {
            const btn = document.getElementById('retry-btn');
            btn.textContent = 'Mengecek koneksi...';
            fetch('https://keep.google.com/u/0/', { method: 'HEAD', mode: 'no-cors' })
                .then(() => {
                    window.location.href = 'https://keep.google.com/u/0/';
                })
                .catch(() => {
                    setTimeout(() => {
                        btn.textContent = 'Coba Hubungkan Ulang';
                    }, 1000);
                });
        }

        document.getElementById('retry-btn').addEventListener('click', checkAndReload);

        window.addEventListener('online', () => {
            window.location.href = 'https://keep.google.com/u/0/';
        });

        // Polling reconnection
        setInterval(() => {
            if (navigator.onLine) {
                checkAndReload();
            }
        }, 5000);
    </script>
</body>
</html>
"##;
