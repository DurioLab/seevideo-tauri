#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod license;
mod store;

use serde::Serialize;
use std::fs;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tauri::{Manager, WindowUrl};

use crate::license::{decode_activation_code, LicenseAccount};
use crate::store::{
    get_last_selected_email, load_store, merge_accounts, set_last_selected_email,
};

const LUMINA_URL: &str = "https://ai.byteplus.com/lumina/model-experience/video?mode=video";
const BYTEPLUS_AUTH_LOGIN_URL: &str = "https://console.byteplus.com/auth/login/";
const AUTH_WORKER_LABEL: &str = "auth-worker";

static WORKER_LOGIN_DONE: AtomicBool = AtomicBool::new(false);

fn should_show_auth_worker() -> bool {
    matches!(
        std::env::var("SEEVIDEO_SHOW_AUTH_WORKER")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn private_key_pem() -> anyhow::Result<String> {
    let base = std::env::current_dir()?
        .join("..")
        .join("keys")
        .join("license_priv.pem");
    let txt = fs::read_to_string(base)?;
    Ok(txt)
}

#[derive(Serialize)]
struct ImportResult {
    added: usize,
    total: usize,
}

fn build_login_js(email: &str, password: &str) -> String {
    let email = serde_json::to_string(email).unwrap_or_else(|_| "\"\"".to_string());
    let password = serde_json::to_string(password).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        r#"(async function() {{
  const sleep = (ms)=>new Promise(r=>setTimeout(r,ms));

  const bridge = (msg) => {{
    try {{
      const api = window.__TAURI__ || window.__TAURI_INTERNALS__;
      const fn = (api && api.tauri && api.tauri.invoke) || (api && api.invoke);
      if (fn) fn('frontend_auth_log', {{ message: `[worker] ${{msg}}` }}).catch(() => {{}});
    }} catch (_) {{}}
  }};

  const log = (...args) => {{
    try {{
      console.log('[SeeVideoAuth][worker]', ...args);
      const s = args.map((x) => {{
        if (typeof x === 'string') return x;
        try {{ return JSON.stringify(x); }} catch (_) {{ return String(x); }}
      }}).join(' | ');
      bridge(s);
    }} catch (_) {{}}
  }};

  const isVisible = (el) => {{
    if (!el) return false;
    const r = el.getBoundingClientRect();
    const cs = window.getComputedStyle(el);
    return r.width > 0 && r.height > 0 && cs.visibility !== 'hidden' && cs.display !== 'none';
  }};

  const setValue = (el, value) => {{
    if (!el) return false;
    try {{
      el.focus();
      const proto = Object.getPrototypeOf(el);
      const desc = proto ? Object.getOwnPropertyDescriptor(proto, 'value') : null;
      const setter = desc && desc.set;
      if (setter) setter.call(el, value);
      else el.value = value;

      el.dispatchEvent(new InputEvent('input', {{ bubbles: true, inputType: 'insertText', data: value }}));
      el.dispatchEvent(new Event('change', {{ bubbles: true }}));

      const now = (el.value || '');
      return now.length > 0;
    }} catch (_) {{
      return false;
    }}
  }};

  const clickFirstVisibleText = (texts) => {{
    const ts = Array.isArray(texts) ? texts : [texts];
    const nodes = Array.from(document.querySelectorAll('button,a,span,div,p'));
    for (const t of ts) {{
      const target = nodes.find((x) => (x.innerText || '').trim() === t && isVisible(x));
      if (target) {{
        target.click();
        return t;
      }}
    }}
    return '';
  }};

  const clickVisibleButtonText = (texts) => {{
    const ts = Array.isArray(texts) ? texts : [texts];
    const nodes = Array.from(document.querySelectorAll('button,[role="button"],input[type="submit"]'));
    for (const t of ts) {{
      const target = nodes.find((x) => {{
        const text = (x.innerText || x.value || '').trim();
        return text === t && isVisible(x);
      }});
      if (target) {{
        target.click();
        return t;
      }}
    }}
    return '';
  }};

  const pickVisible = (selectors) => {{
    for (const s of selectors) {{
      const list = Array.from(document.querySelectorAll(s));
      for (const el of list) {{
        if (isVisible(el)) return el;
      }}
    }}
    return null;
  }};

  const findUsernameInput = () => pickVisible([
    '#Identity_input',
    '#Email_input',
    'input[autocomplete="username"]',
    'input[name="username"]',
    '#username:not([type="hidden"])',
    'input[type="email"]',
    'input[placeholder*="用户名"]',
    'input[placeholder*="电子邮件"]',
    'input[placeholder*="邮箱"]',
    'input[aria-label*="用户名"]',
    'input[aria-label*="电子邮件"]',
    'input[aria-label*="邮箱"]',
  ]);

  const findPasswordInput = () => pickVisible([
    '#Password_input',
    'input[autocomplete="current-password"]',
    'input[type="password"]',
    'input[name*="password" i]',
  ]);

  const findLoginButton = () => {{
    const byText = clickVisibleButtonText(['登录', 'Log in', 'Sign in', 'Sign In', 'Login']);
    if (byText) return {{ clicked: true, by: byText }};
    const submit = pickVisible(['button[type="submit"]', 'input[type="submit"]']);
    if (submit) {{
      submit.click();
      return {{ clicked: true, by: 'submit' }};
    }}
    return {{ clicked: false, by: 'none' }};
  }};

  const maybeDismissCookie = () => {{
    clickFirstVisibleText(['全部接受', 'Accept all', '接受']);
  }};

  const STATE_KEY = '__sv_worker_state__';
  const RESET_KEY = '__sv_worker_reset_done__';

  const getSession = (k, fallback='') => {{
    try {{
      const v = window.sessionStorage.getItem(k);
      return v == null ? fallback : v;
    }} catch (_) {{
      return fallback;
    }}
  }};
  const setSession = (k, v) => {{
    try {{ window.sessionStorage.setItem(k, v); }} catch (_) {{}}
  }};

  const getState = () => getSession(STATE_KEY, 'init');
  const setState = (v) => setSession(STATE_KEY, v);
  const getResetDone = () => getSession(RESET_KEY, '') === '1';
  const setResetDone = () => setSession(RESET_KEY, '1');

  const callLogoutEndpoint = async () => {{
    const logoutUrl = 'https://ai.byteplus.com/signin/logout';
    try {{
      await fetch(logoutUrl, {{ method: 'GET', credentials: 'include', mode: 'no-cors' }});
      log('logout_fetch_sent', logoutUrl);
    }} catch (e) {{
      log('logout_fetch_error', String(e));
    }}
    try {{
      const img = new Image();
      img.src = `${{logoutUrl}}?ts=${{Date.now()}}`;
      log('logout_beacon_sent');
    }} catch (_) {{}}
  }};

  const clearSessionArtifacts = async () => {{
    try {{ localStorage.clear(); }} catch (_) {{}}
    try {{ sessionStorage.clear(); }} catch (_) {{}}

    try {{
      if (window.indexedDB && typeof window.indexedDB.databases === 'function') {{
        const dbs = await window.indexedDB.databases();
        for (const db of dbs || []) {{
          if (db && db.name) {{
            try {{ window.indexedDB.deleteDatabase(db.name); }} catch (_) {{}}
          }}
        }}
      }}
    }} catch (_) {{}}

    try {{
      if (window.caches && typeof window.caches.keys === 'function') {{
        const keys = await window.caches.keys();
        for (const k of keys) {{
          try {{ await window.caches.delete(k); }} catch (_) {{}}
        }}
      }}
    }} catch (_) {{}}

    try {{
      if (navigator.serviceWorker && navigator.serviceWorker.getRegistrations) {{
        const regs = await navigator.serviceWorker.getRegistrations();
        for (const r of regs) {{
          try {{ await r.unregister(); }} catch (_) {{}}
        }}
      }}
    }} catch (_) {{}}

    try {{
      const cookies = document.cookie ? document.cookie.split(';') : [];
      for (const raw of cookies) {{
        const name = raw.split('=')[0]?.trim();
        if (!name) continue;
        const expires = 'expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/;';
        document.cookie = `${{name}}=; ${{expires}}`;
        document.cookie = `${{name}}=; ${{expires}} domain=.byteplus.com;`;
        document.cookie = `${{name}}=; ${{expires}} domain=console.byteplus.com;`;
      }}
    }} catch (_) {{}}
  }};

  log('worker_login_start', document.readyState, window.location.href);

  const authUrl = 'https://console.byteplus.com/auth/login/';
  const href = window.location.href;
  const onConsole = window.location.hostname.endsWith('console.byteplus.com');
  const onAuthLogin = href.includes('/auth/login');

  if (!getResetDone()) {{
    log('session_reset_begin', href);
    await callLogoutEndpoint();
    await sleep(600);
    await clearSessionArtifacts();
    setResetDone();
    setState('init');
    log('session_reset_done', window.location.href);
  }}

  const state = getState();
  if ((state === 'submitted' || state === 'success') && !onAuthLogin) {{
    setState('success');
    log('login_success_page_detected', window.location.href);
    bridge('worker_sync_main');
    await sleep(5000);
    bridge('worker_close_ok');
    return;
  }}

  if (!onConsole || !onAuthLogin) {{
    if (state === 'init' || state === 'filling') {{
      log('navigate_to_auth_login', window.location.href, '=>', authUrl);
      window.location.href = authUrl;
      return;
    }}
  }}

  if (state === 'submitted') {{
    log('awaiting_post_login_redirect', window.location.href);
    return;
  }}

  if (window.__sv_worker_login_started__) {{
    return;
  }}
  window.__sv_worker_login_started__ = true;
  setState('filling');

  maybeDismissCookie();
  log('auth_page_ready', window.location.href);

  let usernameFilled = false;
  let passwordFilled = false;

  for (let i = 0; i < 200; i += 1) {{
    const user = findUsernameInput();
    const pwd = findPasswordInput();

    if (!usernameFilled) {{
      if (user) {{
        const ok = setValue(user, {email});
        await sleep(120);
        const uv = (user.value || '').trim();
        usernameFilled = ok && uv.length > 0;
        log('username_filled', i + 1, user.id || user.name || user.type || 'unknown', usernameFilled ? 'ok' : 'empty');
      }}
      await sleep(250);
      continue;
    }}

    if (!passwordFilled) {{
      if (pwd) {{
        const ok = setValue(pwd, {password});
        await sleep(120);
        const pv = (pwd.value || '');
        passwordFilled = ok && pv.length > 0;
        log('password_filled', i + 1, pwd.id || pwd.name || pwd.type || 'unknown', passwordFilled ? 'ok' : 'empty');
      }}
      await sleep(250);
      continue;
    }}

    if (user && pwd && usernameFilled && passwordFilled) {{
      let ret = findLoginButton();
      if (!ret.clicked) {{
        try {{
          pwd.dispatchEvent(new KeyboardEvent('keydown', {{ key: 'Enter', code: 'Enter', bubbles: true }}));
          pwd.dispatchEvent(new KeyboardEvent('keyup', {{ key: 'Enter', code: 'Enter', bubbles: true }}));
          const form = pwd.closest('form');
          if (form) {{
            if (typeof form.requestSubmit === 'function') form.requestSubmit();
            else form.submit();
          }}
        }} catch (_) {{}}
        await sleep(250);
        const byText2 = clickFirstVisibleText(['登录', 'Log in', 'Sign in', 'Sign In', 'Login']);
        ret = {{ clicked: Boolean(byText2), by: byText2 || 'enter_or_form' }};
      }}

      if (!ret.clicked) {{
        log('login_submit_retry', i + 1, 'not_found', 'continue_loop');
        await sleep(300);
        continue;
      }}

      log('login_submit', i + 1, 'clicked', ret.by);
      setState('submitted');
      bridge('worker_login_submitted');
      await sleep(6000);
      log('post_submit_wait_done', window.location.href);
      bridge('worker_sync_main');
      bridge('worker_close_ok');
      return;
    }}

    if (i === 39 || i === 79 || i === 119 || i === 159 || i === 199) {{
      log(
        'waiting_form',
        i + 1,
        usernameFilled ? 'username_ok' : 'username_missing',
        passwordFilled ? 'pwd_ok' : 'pwd_missing',
        user ? 'user_el_yes' : 'user_el_no',
        pwd ? 'pwd_el_yes' : 'pwd_el_no'
      );
      maybeDismissCookie();
    }}

    await sleep(250);
  }}

  log('login_form_timeout');
  setState('failed');
  bridge('worker_login_form_timeout');
}})();"#
    )
}

fn sync_main_to_lumina(app: &tauri::AppHandle, reason: &str) {
    let Some(main) = app.get_window("main") else {
        eprintln!(
            "[SeeVideoAuth] sync_main_to_lumina skipped: main window missing ({})",
            reason
        );
        return;
    };

    let lumina = serde_json::to_string(LUMINA_URL).unwrap_or_else(|_| "\"\"".to_string());
    let js = format!("try {{ window.location.href = {}; }} catch (_) {{}}", lumina);
    match main.eval(&js) {
        Ok(_) => eprintln!("[SeeVideoAuth] sync_main_to_lumina done ({})", reason),
        Err(e) => eprintln!("[SeeVideoAuth] sync_main_to_lumina failed ({}): {}", reason, e),
    }

    let app_handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(2));
        if let Some(main) = app_handle.get_window("main") {
            let _ = main.eval(layout_cleanup_js());
            let _ = main.eval(
                r#"try { if (window.__sv_tick__) window.__sv_tick__('worker_sync_main'); } catch (_) {}"#,
            );
        }
    });
}

fn layout_cleanup_js() -> &'static str {
    include_str!("inject.js")
}

#[tauri::command]
fn import_activation_code(code: String) -> Result<ImportResult, String> {
    eprintln!("[SeeVideoAuth] import_activation_code called, code_len={}", code.len());
    let pem = private_key_pem().map_err(|e| e.to_string())?;
    let payload = decode_activation_code(&code, &pem).map_err(|e| e.to_string())?;

    let added = merge_accounts(&payload.accounts).map_err(|e| e.to_string())?;
    let total = load_store().map_err(|e| e.to_string())?.accounts.len();
    eprintln!("[SeeVideoAuth] import_activation_code done: added={}, total={}", added, total);

    Ok(ImportResult { added, total })
}

#[tauri::command]
fn list_accounts() -> Result<Vec<String>, String> {
    eprintln!("[SeeVideoAuth] list_accounts called");
    let st = load_store().map_err(|e| e.to_string())?;
    let mut emails: Vec<String> = st.accounts.into_iter().map(|a| a.email).collect();
    emails.sort();
    eprintln!("[SeeVideoAuth] list_accounts done: {} account(s)", emails.len());
    Ok(emails)
}

#[tauri::command]
fn get_last_selected_account() -> Result<Option<String>, String> {
    eprintln!("[SeeVideoAuth] get_last_selected_account called");
    let email = get_last_selected_email().map_err(|e| e.to_string())?;
    eprintln!("[SeeVideoAuth] get_last_selected_account done: {:?}", email);
    Ok(email)
}

#[tauri::command]
fn set_last_selected_account(email: String) -> Result<(), String> {
    eprintln!("[SeeVideoAuth] set_last_selected_account called: {}", email);
    set_last_selected_email(Some(email)).map_err(|e| e.to_string())
}

#[tauri::command]
fn switch_account(app: tauri::AppHandle, email: String) -> Result<(), String> {
    eprintln!("[SeeVideoAuth] switch_account called: {}", email);
    let st = load_store().map_err(|e| e.to_string())?;
    let total_accounts = st.accounts.len();
    eprintln!("[SeeVideoAuth] switch_account store loaded: {} account(s)", total_accounts);

    let acct: LicenseAccount = st
        .accounts
        .into_iter()
        .find(|a| a.email.eq_ignore_ascii_case(&email))
        .ok_or_else(|| {
            eprintln!("[SeeVideoAuth] switch_account account not found: {}", email);
            "account not found".to_string()
        })?;
    eprintln!("[SeeVideoAuth] switch_account account matched: {}", acct.email);

    // 不在主窗口执行登录，避免出现密码页面。
    if let Some(old_worker) = app.get_window(AUTH_WORKER_LABEL) {
        eprintln!("[SeeVideoAuth] switch_account closing old auth worker");
        let _ = old_worker.close();
    }

    let show_auth_worker = should_show_auth_worker();
    let worker_visibility_label = if show_auth_worker { "VISIBLE" } else { "HIDDEN" };
    eprintln!(
        "[SeeVideoAuth] switch_account creating {} auth worker: {}",
        worker_visibility_label,
        BYTEPLUS_AUTH_LOGIN_URL
    );
    let _auth_worker = tauri::WindowBuilder::new(
        &app,
        AUTH_WORKER_LABEL,
        WindowUrl::External(BYTEPLUS_AUTH_LOGIN_URL.parse().expect("valid url")),
    )
    .title("SeeVideoAuthWorker")
    .visible(show_auth_worker)
    .skip_taskbar(!show_auth_worker)
    .build()
    .map_err(|e| {
        eprintln!("[SeeVideoAuth] switch_account worker build failed: {}", e);
        e.to_string()
    })?;
    eprintln!(
        "[SeeVideoAuth] switch_account {} auth worker ready",
        if show_auth_worker { "visible" } else { "hidden" }
    );

    // 延迟+重试注入脚本，确保在 auth-worker 真正加载目标页面后执行。
    WORKER_LOGIN_DONE.store(false, Ordering::SeqCst);
    let js = build_login_js(&acct.email, &acct.password);
    let app_handle = app.clone();
    thread::spawn(move || {
        for attempt in 1..=15 {
            if WORKER_LOGIN_DONE.load(Ordering::SeqCst) {
                eprintln!("[SeeVideoAuth] switch_account stop eval retries: done flag set");
                break;
            }
            thread::sleep(Duration::from_millis(900));
            if WORKER_LOGIN_DONE.load(Ordering::SeqCst) {
                eprintln!("[SeeVideoAuth] switch_account stop eval retries after sleep: done flag set");
                break;
            }
            let Some(worker) = app_handle.get_window(AUTH_WORKER_LABEL) else {
                eprintln!("[SeeVideoAuth] switch_account worker missing before eval attempt={}", attempt);
                continue;
            };
            match worker.eval(&js) {
                Ok(_) => {
                    eprintln!("[SeeVideoAuth] switch_account eval attempt={} submitted", attempt);
                }
                Err(e) => {
                    eprintln!("[SeeVideoAuth] switch_account eval attempt={} failed: {}", attempt, e);
                }
            }
        }
    });

    // 后台兜底关闭 worker：给足调试时间，避免按钮未触发时被过早关闭。
    let app_handle = app.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(300));
        if let Some(worker) = app_handle.get_window(AUTH_WORKER_LABEL) {
            eprintln!("[SeeVideoAuth] switch_account closing auth worker after fallback delay");
            let _ = worker.close();
        }
    });

    eprintln!("[SeeVideoAuth] switch_account submitted to auth worker: {}", email);
    Ok(())
}

#[tauri::command]
fn cleanup_layout(app: tauri::AppHandle) -> Result<(), String> {
    let main = app
        .get_window("main")
        .ok_or_else(|| "main window not found".to_string())?;

    // 先走轻量 tick，避免重复注入导致 boot/list_accounts 循环日志。
    let tick_js = r#"try {
      if (window.__sv_tick__) {
        window.__sv_tick__('cleanup_layout_cmd');
      } else {
        throw new Error('sv_tick_missing');
      }
    } catch (_) {}"#;
    let _ = main.eval(tick_js);

    // 保底：若页面还未注入，再注入完整脚本。
    main.eval(layout_cleanup_js()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn frontend_auth_log(app: tauri::AppHandle, message: String) {
    eprintln!("[SeeVideoAuth][Front] {}", message);

    if message.contains("[worker] worker_login_submitted") {
        sync_main_to_lumina(&app, "worker_login_submitted");
    }
    if message.contains("[worker] worker_sync_main") {
        WORKER_LOGIN_DONE.store(true, Ordering::SeqCst);
        sync_main_to_lumina(&app, "worker_sync_main");
    }
    if message.contains("[worker] worker_login_form_timeout") {
        WORKER_LOGIN_DONE.store(true, Ordering::SeqCst);
    }
    if message.contains("[worker] worker_close_ok") {
        WORKER_LOGIN_DONE.store(true, Ordering::SeqCst);
        if let Some(worker) = app.get_window(AUTH_WORKER_LABEL) {
            eprintln!("[SeeVideoAuth] closing auth worker after worker_close_ok");
            let _ = worker.close();
        }
    }
}

fn main() {
    tauri::Builder::default()
        .on_page_load(|window, payload| {
            eprintln!(
                "[SeeVideoAuth] page_load: window={}, url={}",
                window.label(),
                payload.url()
            );
        })
        .invoke_handler(tauri::generate_handler![
            import_activation_code,
            list_accounts,
            get_last_selected_account,
            set_last_selected_account,
            switch_account,
            cleanup_layout,
            frontend_auth_log,
        ])
        .setup(|app| {
            let main_window = tauri::WindowBuilder::new(
                app,
                "main",
                WindowUrl::External(LUMINA_URL.parse().expect("valid url")),
            )
            .title("SeeVideo")
            .fullscreen(false)
            .inner_size(1440.0, 900.0)
            .build()?;

            let _ = main_window.eval(layout_cleanup_js());
            eprintln!("[SeeVideoAuth] inject script submitted on main window setup");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
