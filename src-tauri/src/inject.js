(() => {
  if (window.__sv_injected_v2__) {
    try {
      if (typeof window.__sv_tick__ === 'function') {
        window.__sv_tick__('reinject');
      }
      console.log('[SeeVideoAuth]', new Date().toISOString(), 'inject_reentry_skip_boot');
    } catch (_e) {}
    return;
  }
  window.__sv_injected_v2__ = true;

  const NAV_ID = '__sv_top_nav__';
  const MODAL_ID = '__sv_activate_modal__';
  const STYLE_ID = '__sv_top_nav_style__';
  const CONTENT_GATE_ID = '__sv_content_gate__';

  const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
  const authLog = (...args) => {
    try {
      const ts = new Date().toISOString();
      console.log('[SeeVideoAuth]', ts, ...args);
      const serialized = args.map((x) => {
        if (typeof x === 'string') return x;
        try { return JSON.stringify(x); } catch (_e) { return String(x); }
      }).join(' | ');
      try {
        tauriInvoke('frontend_auth_log', { message: `${ts} ${serialized}` }).catch(() => {});
      } catch (_e) {}
    } catch (_e) {}
  };

  const tauriInvoke = (cmd, args = {}) => {
    const api = window.__TAURI__ || window.__TAURI_INTERNALS__;
    const fn = (api && api.tauri && api.tauri.invoke) || (api && api.invoke);
    if (!fn) throw new Error('Tauri invoke unavailable');
    return fn(cmd, args);
  };

  const getMountRoot = () => document.body || document.documentElement;

  const invokeWithTimeout = (cmd, args = {}, timeoutMs = 12000) => {
    let timer = null;
    return Promise.race([
      tauriInvoke(cmd, args),
      new Promise((_, reject) => {
        timer = setTimeout(() => reject(new Error(`invoke timeout: ${cmd}`)), timeoutMs);
      })
    ]).finally(() => {
      if (timer) clearTimeout(timer);
    });
  };

  const hideLayout = () => {
    document.querySelectorAll('.ba-sider').forEach((el) => {
      el.style.width = '0';
    });

    document.querySelectorAll('.lumi-feedback-entry').forEach((el) => {
      el.classList.add('hidden');
    });

    const h1 = document.querySelector('.model-experience-main-content h1')
    if(h1){
      h1.textContent = '开启您的创作之旅'
    }

    const wrapper = document.querySelector('.ba-generate-framework-main-content-wrapper');
    if (wrapper) {
      wrapper.classList.remove('flex-none');
      wrapper.classList.add('flex-1');

      const first = wrapper.querySelector(':scope > div:first-child') || wrapper.firstElementChild;
      if (first) {
        first.style.width = '100%';
      }
    }

    const framework = document.querySelector('.ba-generate-framework');
    if (framework) {
      const second = framework.children[1];
      const third = framework.children[2];
      if (second) {
        second.classList.add('hidden');
      }
      if (third) {
        third.classList.add('hidden');
      }
    }

    const toHide = ['智能视频', '自由创作'];
    Array.from(document.querySelectorAll('button,[role="button"]')).forEach((el) => {
      const txt = (el.innerText || '').trim();
      if (!toHide.some((t) => txt.startsWith(t))) return;
      const r = el.getBoundingClientRect();
      if (r.top < window.innerHeight * 0.55) return;
      el.style.display = 'none';
    });

    if (document.body) {
      document.body.style.paddingTop = '0';
      document.body.style.overflowY = 'hidden';
    }
    document.documentElement.style.overflowY = 'hidden';
  };

  const ensureStyle = () => {
    if (document.getElementById(STYLE_ID)) return;
    const style = document.createElement('style');
    style.id = STYLE_ID;
    style.textContent = `
      #${NAV_ID}{
        position:fixed;left:0;right:0;top:0;height:48px;z-index:2147483647;
        display:flex;align-items:center;gap:10px;padding:0 12px;
        background:rgba(16,18,24,.95);border-bottom:1px solid rgba(255,255,255,.12);
        backdrop-filter:blur(6px);color:#fff;font:13px/1.2 -apple-system,BlinkMacSystemFont,Segoe UI,Arial;
      }
      #${NAV_ID}, #${NAV_ID} *{
        pointer-events:auto !important;
      }
      #${NAV_ID} .sv-title{font-weight:600;opacity:.95;margin-right:8px}
      #${NAV_ID} .sv-auth-status{display:flex;align-items:center;gap:6px;min-width:84px}
      #${NAV_ID} .sv-auth-dot{
        width:10px;height:10px;border-radius:50%;display:inline-block;
        border:1px solid rgba(255,255,255,.35);
        box-shadow:0 0 0 1px rgba(0,0,0,.2) inset;
      }
      #${NAV_ID} .sv-auth-dot.status-failed{background:#ef4444;box-shadow:0 0 10px rgba(239,68,68,.5)}
      #${NAV_ID} .sv-auth-dot.status-loading{background:#f59e0b;box-shadow:0 0 10px rgba(245,158,11,.5);animation:sv-auth-loading-pulse .8s ease-in-out infinite}
      #${NAV_ID} .sv-auth-dot.status-success{background:#22c55e;box-shadow:0 0 10px rgba(34,197,94,.5)}
      @keyframes sv-auth-loading-pulse{
        0%,100%{opacity:1;transform:scale(1);box-shadow:0 0 12px rgba(245,158,11,.6)}
        50%{opacity:.35;transform:scale(.82);box-shadow:0 0 3px rgba(245,158,11,.18)}
      }
      #${NAV_ID} .sv-auth-text{font-size:12px;opacity:.92;min-width:56px}
      #${NAV_ID} button,#${NAV_ID} select{
        height:30px;border:1px solid rgba(255,255,255,.18);border-radius:8px;
        background:rgba(255,255,255,.08);color:#fff;padding:0 10px;
        pointer-events:auto !important;
      }
      #${NAV_ID} select{min-width:280px}
      #${MODAL_ID}{
        position:fixed;inset:0;background:rgba(0,0,0,.45);z-index:2147483646;
        display:flex;align-items:center;justify-content:center;
      }
      #${MODAL_ID} .sv-box{
        position:relative;
        width:520px;
        max-width:92vw;
        background:#151923;
        border:1px solid rgba(255,255,255,.15);
        border-radius:12px;
        padding:46px 14px 14px;
      }
      #${MODAL_ID} textarea{
        width:100%;height:120px;box-sizing:border-box;border-radius:8px;
        border:1px solid #303849;background:#1d2230;color:#fff;padding:10px;resize:vertical;
      }
      #${MODAL_ID} .sv-actions{margin-top:10px;display:flex;justify-content:flex-end;gap:8px}
      #${MODAL_ID} .sv-close{
        position:absolute;right:10px;top:10px;width:28px;height:28px;
        display:flex;align-items:center;justify-content:center;
        border-radius:8px;border:1px solid rgba(255,255,255,.2);
        background:rgba(255,255,255,.08);color:#fff;cursor:pointer;
        z-index:2;padding:0;
      }
      #${MODAL_ID} .sv-close svg{width:14px;height:14px;display:block,margin-left:5px;margin-top:5px;}
      #${MODAL_ID} button{
        height:32px;padding:0 12px;border-radius:8px;border:1px solid rgba(255,255,255,.2);
        background:rgba(255,255,255,.08);color:#fff;
      }
      #${CONTENT_GATE_ID}{
        position:absolute;
        inset:0;
        display:none;
        align-items:center;
        justify-content:center;
        background:rgba(8,10,14,.38);
        backdrop-filter: blur(1px);
        z-index:120;
        pointer-events:none;
      }
      #${CONTENT_GATE_ID}.loading{
        display:flex;
        pointer-events:none;
      }
      #${CONTENT_GATE_ID}.blocked{
        display:flex;
        pointer-events:none;
      }
      #${CONTENT_GATE_ID} .sv-gate-box{
        min-width:260px;
        max-width:72%;
        border-radius:12px;
        border:1px solid rgba(255,255,255,.18);
        background:rgba(20,24,33,.88);
        color:#fff;
        text-align:center;
        padding:12px 16px;
        font-size:14px;
      }
      #${CONTENT_GATE_ID}.loading .sv-gate-box::before{
        content:'⏳ ';
      }
    `;
    document.documentElement.appendChild(style);
  };

  const ensureModal = () => {
    let modal = document.getElementById(MODAL_ID);
    if (modal) return modal;
    modal = document.createElement('div');
    modal.id = MODAL_ID;
    modal.style.display = 'none';
    modal.innerHTML = `
      <div class="sv-box">
        <button class="sv-close" id="sv_activate_close" title="关闭" aria-label="关闭">
          <svg viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
            <path d="M6 6L18 18M18 6L6 18" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
          </svg>
        </button>
        <textarea id="sv_code_input" placeholder="输入激活码"></textarea>
        <div class="sv-actions">
          <button id="sv_activate_confirm">确认</button>
        </div>
      </div>
    `;
    const mountRoot = getMountRoot();
    mountRoot.appendChild(modal);
    return modal;
  };

  // 遮罩逻辑已移除：保留空实现以兼容旧调用。
  const ensureContentGate = () => null;
  const setContentGate = (_msg, _opts = {}) => {};

  const AUTH_STATUS_TEXT = {
    failed: '失败',
    loading: '登录中',
    success: '成功',
  };

  const setAuthStatus = (status, reason = '') => {
    const normalized = ['failed', 'loading', 'success'].includes(status) ? status : 'failed';
    const prev = window.__sv_auth_status__;
    window.__sv_auth_status__ = normalized;

    const dot = document.getElementById('sv_auth_dot');
    const text = document.getElementById('sv_auth_text');
    if (dot) {
      dot.classList.remove('status-failed', 'status-loading', 'status-success');
      dot.classList.add(`status-${normalized}`);
      dot.title = `登录状态：${AUTH_STATUS_TEXT[normalized]}`;
    }
    if (text) {
      text.textContent = AUTH_STATUS_TEXT[normalized];
      text.title = dot ? dot.title : `登录状态：${AUTH_STATUS_TEXT[normalized]}`;
    }

    if (prev !== normalized) {
      authLog('auth_indicator', normalized, reason || 'no_reason');
    }
  };

  const isAuthPromptVisible = () => {
    const authSelectors = ['#Identity_input', '#Email_input', '#Password_input', '#verifyInput', 'input[type="password"]'];
    return authSelectors.some((s) => {
      const el = document.querySelector(s);
      if (!el) return false;
      const r = el.getBoundingClientRect();
      return r.width > 0 && r.height > 0;
    });
  };

  const isLikelyLoggedIn = () => {
    const isVisible = (el) => {
      if (!el) return false;
      const r = el.getBoundingClientRect();
      return r.width > 0 && r.height > 0;
    };

    const loggedInSelectors = [
      '.i18n-bd-user-user',
      '.bd-user-avatar',
      '[class*="user-avatar"]',
      '[class*="account-avatar"]',
      '[class*="user-menu"] [class*="avatar"]',
      '[class*="avatar"] img'
    ];
    if (loggedInSelectors.some((s) => isVisible(document.querySelector(s)))) return true;

    if (isAuthPromptVisible()) return false;

    const signInVisible = Array.from(document.querySelectorAll('button,a,span,div')).some((el) => {
      const t = (el.innerText || '').trim().toLowerCase();
      if (!(t === 'sign in' || t === '登录' || t === 'log in')) return false;
      const r = el.getBoundingClientRect();
      return r.width > 0 && r.height > 0;
    });
    if (signInVisible) return false;

    // 保守策略：没有明确“已登录”信号时，视为未登录，避免误判成功。
    return false;
  };

  const refreshContentGateByAuthState = () => {
    const loggedIn = isLikelyLoggedIn();
    const state = loggedIn ? 'logged_in' : 'need_login';
    if (window.__sv_last_auth_state__ !== state) {
      authLog('auth_state', state);
      window.__sv_last_auth_state__ = state;
    }
    if (!window.__sv_login_inflight__) {
      setAuthStatus(loggedIn ? 'success' : 'failed', `auth_state:${state}`);
    }
    // 遮罩已去掉：这里仅保留登录态日志，不再操作覆盖层。
  };



  // loading 逻辑已移除：保留同名函数以兼容既有调用。
  const clearLoadingLock = (_reason = 'manual') => {};

  const runWithLoadingGate = async (_message, fn, _timeoutMs = 0) => {
    return await fn();
  };

  const triggerSwitchAndWait = async (email) => {
    if (!email) {
      authLog('switch_skip_empty_email');
      setAuthStatus('failed', 'switch_empty_email');
      return false;
    }
    window.__sv_login_inflight__ = true;
    setAuthStatus('loading', `switch_begin:${email}`);
    authLog('switch_begin', email);

    try {
      authLog('switch_attempt', email, 1);
      await invokeWithTimeout('switch_account', { email }, 12000);
      authLog('switch_invoke_ok', email, 1);
    } catch (e) {
      window.__sv_login_inflight__ = false;
      setAuthStatus('failed', `switch_invoke_error:${email}`);
      authLog('switch_invoke_error', email, 1, String(e));
      authLog('switch_failed', email);
      return false;
    }

    // auth-worker 登录流程可能较慢，给足等待窗口，避免误判失败。
    await sleep(1000);

    for (let t = 0; t < 160; t += 1) {
      if (isLikelyLoggedIn()) {
        window.__sv_login_inflight__ = false;
        setAuthStatus('success', `switch_success:${email}`);
        authLog('switch_success', email, 'poll', t + 1);
        return true;
      }
      if (t === 0 || t === 19 || t === 39 || t === 79 || t === 119 || t === 159) {
        authLog('switch_poll_waiting', email, t + 1);
      }
      await sleep(250);
    }

    window.__sv_login_inflight__ = false;
    setAuthStatus('failed', `switch_poll_timeout:${email}`);
    authLog('switch_poll_timeout', email, 1);
    authLog('switch_failed', email);
    return false;
  };

  const ensureNav = () => {
    let nav = document.getElementById(NAV_ID);
    if (nav) {
      if (!document.getElementById('sv_auth_status')) {
        const statusWrap = document.createElement('div');
        statusWrap.className = 'sv-auth-status';
        statusWrap.id = 'sv_auth_status';
        statusWrap.title = '登录状态';
        statusWrap.innerHTML = '<span id="sv_auth_dot" class="sv-auth-dot status-failed"></span><span id="sv_auth_text" class="sv-auth-text">失败</span>';
        const firstBtn = nav.querySelector('#sv_btn_activate');
        if (firstBtn) nav.insertBefore(statusWrap, firstBtn);
        else nav.appendChild(statusWrap);
      }
      return nav;
    }
    nav = document.createElement('div');
    nav.id = NAV_ID;
    nav.innerHTML = `
      <div class="sv-title">SeeVideo</div>
      <div class="sv-auth-status" id="sv_auth_status" title="登录状态">
        <span id="sv_auth_dot" class="sv-auth-dot status-failed"></span>
        <span id="sv_auth_text" class="sv-auth-text">失败</span>
      </div>
      <button id="sv_btn_activate">激活</button>
      <select id="sv_account_select"><option value="">账号列表（空）</option></select>
    `;
    const mountRoot = getMountRoot();
    mountRoot.appendChild(nav);
    return nav;
  };

  const forceTopNavInteractive = () => {
    const nav = document.getElementById(NAV_ID);
    if (!nav) return;
    nav.style.zIndex = '2147483647';
    nav.style.pointerEvents = 'auto';
    const btn = document.getElementById('sv_btn_activate');
    const sel = document.getElementById('sv_account_select');
    [btn, sel].forEach((el) => {
      if (!el) return;
      el.style.pointerEvents = 'auto';
      el.style.position = 'relative';
      el.style.zIndex = '2147483647';
      try { el.disabled = false; } catch (_) {}
    });
  };

  const removeLegacyContentGate = () => {
    const old = document.getElementById(CONTENT_GATE_ID);
    if (old && old.parentNode) old.parentNode.removeChild(old);
  };

  const normalizeEmail = (email) => String(email || '').trim();

  const loadLastSelectedAccount = async () => {
    try {
      const email = await invokeWithTimeout('get_last_selected_account', {}, 6000);
      const normalized = normalizeEmail(email);
      authLog('last_selected_loaded', normalized || '(empty)');
      window.__sv_last_selected_email__ = normalized || '';
      return normalized;
    } catch (e) {
      authLog('last_selected_load_error', String(e));
      return '';
    }
  };

  const saveLastSelectedAccount = async (email, reason = 'unknown') => {
    const normalized = normalizeEmail(email);
    if (!normalized) return false;
    try {
      await invokeWithTimeout('set_last_selected_account', { email: normalized }, 6000);
      window.__sv_last_selected_email__ = normalized;
      authLog('last_selected_saved', reason, normalized);
      return true;
    } catch (e) {
      authLog('last_selected_save_error', reason, normalized, String(e));
      return false;
    }
  };

  const choosePreferredAccount = (emails, preferredEmail = '') => {
    const list = Array.isArray(emails) ? emails.map(normalizeEmail).filter(Boolean) : [];
    const preferred = normalizeEmail(preferredEmail);
    if (preferred && list.includes(preferred)) return preferred;
    return list[0] || '';
  };

  const refreshAccountOptions = async (reason = 'unknown') => {
    const select = document.getElementById('sv_account_select');
    if (!select) {
      authLog('accounts_refresh_skip_no_select', reason);
      return [];
    }
    authLog('accounts_refresh_begin', reason);
    try {
      const emails = await invokeWithTimeout('list_accounts', {}, 6000);
      authLog('accounts_refresh_result', reason, Array.isArray(emails) ? emails.length : -1, emails);
      select.innerHTML = '';
      if (!emails || emails.length === 0) {
        const opt = document.createElement('option');
        opt.value = '';
        opt.textContent = '账号列表（空）';
        select.appendChild(opt);
        window.__sv_last_selected_email__ = '';
        return [];
      }
      emails.forEach((email) => {
        const opt = document.createElement('option');
        opt.value = email;
        opt.textContent = email;
        select.appendChild(opt);
      });
      const preferred = choosePreferredAccount(emails, window.__sv_last_selected_email__ || '');
      if (preferred) {
        select.value = preferred;
        window.__sv_last_selected_email__ = preferred;
        authLog('accounts_refresh_selected', reason, preferred);
      }
      return emails;
    } catch (e) {
      authLog('accounts_refresh_error', reason, String(e));
      return [];
    }
  };

  const wireEvents = () => {
    const activateBtn = document.getElementById('sv_btn_activate');
    const select = document.getElementById('sv_account_select');
    const modal = ensureModal();
    const confirmBtn = document.getElementById('sv_activate_confirm');
    const closeBtn = document.getElementById('sv_activate_close');
    if (confirmBtn) confirmBtn.style.display = 'inline-block';
    if (closeBtn) closeBtn.style.display = 'inline-block';
    const closeModal = () => {
      modal.style.display = 'none';
    };

    if (activateBtn && !activateBtn.dataset.bound) {
      activateBtn.dataset.bound = '1';
      activateBtn.addEventListener('click', () => {
        // loading 中也允许随时打开激活框
        clearLoadingLock('activate_click');
        modal.style.display = 'flex';
      });
    }

    if (closeBtn && !closeBtn.dataset.bound) {
      closeBtn.dataset.bound = '1';
      closeBtn.addEventListener('click', closeModal);
    }

    if (!modal.dataset.boundClose) {
      modal.dataset.boundClose = '1';
      modal.addEventListener('click', (e) => {
        if (e.target === modal) closeModal();
      });
    }

    if (!window.__sv_modal_esc_bound__) {
      window.__sv_modal_esc_bound__ = true;
      window.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
          const m = document.getElementById(MODAL_ID);
          if (m && m.style.display !== 'none') m.style.display = 'none';
        }
      });
    }

    if (confirmBtn && !confirmBtn.dataset.bound) {
      confirmBtn.dataset.bound = '1';
      confirmBtn.addEventListener('click', async () => {
        const input = document.getElementById('sv_code_input');
        const code = (input && input.value || '').trim();
        if (!code) return;

        authLog('activate_submit');
        closeModal();
        try {
          await runWithLoadingGate('激活并登录中，请稍候…', async () => {
            const importResult = await invokeWithTimeout('import_activation_code', { code }, 12000);
            authLog('activate_import_ok', importResult);
            const existingPreferred = window.__sv_last_selected_email__ || await loadLastSelectedAccount();
            const emails = await refreshAccountOptions('activate_submit');
            authLog('activate_accounts', emails.length, emails);
            if (emails.length > 0) {
              const targetEmail = choosePreferredAccount(emails, existingPreferred);
              authLog('activate_switch_target', targetEmail, existingPreferred || '(none)');
              const sel = document.getElementById('sv_account_select');
              if (sel) sel.value = targetEmail;
              await saveLastSelectedAccount(targetEmail, 'activate_submit');
              authLog('activate_switch_first_begin', targetEmail);
              const switched = await triggerSwitchAndWait(targetEmail);
              authLog('activate_switch_first_done', targetEmail, switched);
              if (!switched) {
                authLog('activate_switch_first_failed', targetEmail);
              }
            } else {
              authLog('activate_no_accounts_after_import');
            }
          });

          input.value = '';
        } catch (e) {
          setAuthStatus('failed', 'activate_failed');
          authLog('activate_failed', String(e));
          clearLoadingLock('activate_failed');
          // keep modal open; user can retry input
          refreshContentGateByAuthState();
        }
      });
    }

    if (select && !select.dataset.bound) {
      select.dataset.bound = '1';
      select.addEventListener('change', async () => {
        const email = select.value;
        if (!email) return;
        authLog('manual_switch_select', email);
        window.__sv_last_selected_email__ = email;
        await saveLastSelectedAccount(email, 'manual_switch');
        try {
          await runWithLoadingGate('正在切换账号，请稍候…', async () => {
            await triggerSwitchAndWait(email);
          });
        } catch (e) {
          setAuthStatus('failed', `manual_switch_failed:${email}`);
          authLog('manual_switch_failed', email, String(e));
          clearLoadingLock('manual_switch_failed');
          refreshContentGateByAuthState();
        }
      });
    }
  };

  window.__sv_tick__ = (reason = 'external') => {
    try {
      ensureStyle();
      ensureNav();
      ensureModal();
      removeLegacyContentGate();
      ensureContentGate();
      wireEvents();
      forceTopNavInteractive();
      hideLayout();
      refreshContentGateByAuthState();
    } catch (e) {
      authLog('tick_error', reason, String(e));
    }
  };

  const boot = async () => {
    authLog('boot_start', document.readyState, window.location.href);
    ensureStyle();
    ensureNav();
    ensureModal();
    removeLegacyContentGate();
    ensureContentGate();
    wireEvents();
    forceTopNavInteractive();
    hideLayout();

    const lastSelected = await loadLastSelectedAccount();
    let emails = await refreshAccountOptions('boot');
    if ((!emails || emails.length === 0)) {
      authLog('boot_refresh_empty_retry_list');
      try {
        const fallback = await invokeWithTimeout('list_accounts', {}, 6000);
        emails = Array.isArray(fallback) ? fallback : [];
        authLog('boot_list_accounts_fallback', emails.length, emails);
        const sel = document.getElementById('sv_account_select');
        if (sel) {
          sel.innerHTML = '';
          if (emails.length === 0) {
            const opt = document.createElement('option');
            opt.value = '';
            opt.textContent = '账号列表（空）';
            sel.appendChild(opt);
          } else {
            emails.forEach((email) => {
              const opt = document.createElement('option');
              opt.value = email;
              opt.textContent = email;
              sel.appendChild(opt);
            });
            const preferred = choosePreferredAccount(emails, lastSelected || window.__sv_last_selected_email__ || '');
            if (preferred) {
              sel.value = preferred;
              window.__sv_last_selected_email__ = preferred;
              authLog('boot_fallback_selected', preferred);
            }
          }
        }
      } catch (e) {
        authLog('boot_list_accounts_fallback_error', String(e));
      }
    }

    refreshContentGateByAuthState();

    // 已有账号但未登录时，启动后自动发起一次登录，避免“有账号但无登录动作”。
    if (emails.length > 0 && !isLikelyLoggedIn()) {
      const email = choosePreferredAccount(emails, lastSelected || window.__sv_last_selected_email__ || '');
      authLog('boot_auto_switch', email);
      try {
        await runWithLoadingGate('检测到账号，正在自动登录…', async () => {
          const sel = document.getElementById('sv_account_select');
          if (sel) sel.value = email;
          await saveLastSelectedAccount(email, 'boot_auto_switch');
          await triggerSwitchAndWait(email);
        }, 12000);
      } catch (e) {
        setAuthStatus('failed', 'boot_auto_switch_failed');
        authLog('boot_auto_switch_failed', String(e));
        clearLoadingLock('boot_auto_switch_failed');
      }
      refreshContentGateByAuthState();
    }
  };

  const startRuntime = () => {
    if (window.__sv_started__) return;
    window.__sv_started__ = true;

    boot().catch((e) => {
      authLog('boot_failed', String(e));
    });

    // 轻量保活：维持导航、状态和布局修正。
    if (!window.__sv_timer__) {
      window.__sv_timer__ = setInterval(() => {
        try {
          ensureStyle();
          ensureNav();
          ensureModal();
          wireEvents();
          forceTopNavInteractive();
          hideLayout();
          refreshContentGateByAuthState();
        } catch (e) {
          authLog('tick_error', 'runtime_timer', String(e));
        }
      }, 1200);
    }
  };

  if (document.readyState === 'loading') {
    window.addEventListener('DOMContentLoaded', () => {
      startRuntime();
    }, { once: true });
  } else {
    startRuntime();
  }
})();

