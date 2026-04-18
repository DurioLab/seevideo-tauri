const statusEl = document.getElementById('status');
const emailEl = document.getElementById('email');

function setStatus(msg) { statusEl.textContent = msg; }

async function invoke(cmd, args = {}) {
  const api = window.__TAURI__ || window.__TAURI_INTERNALS__;
  const fn = api?.tauri?.invoke || api?.invoke;
  if (!fn) throw new Error('Tauri invoke 不可用');
  return fn(cmd, args);
}

async function refresh() {
  await invoke('cleanup_layout').catch(() => {});
  const emails = await invoke('list_accounts');
  emailEl.innerHTML = '';
  emails.forEach((e) => {
    const opt = document.createElement('option');
    opt.value = e;
    opt.textContent = e;
    emailEl.appendChild(opt);
  });
  setStatus(`账号数: ${emails.length}`);
}

document.getElementById('import').addEventListener('click', async () => {
  try {
    const code = document.getElementById('code').value.trim();
    if (!code) return setStatus('请先输入激活码');
    const ret = await invoke('import_activation_code', { code });
    setStatus(`导入成功：新增 ${ret.added}，总计 ${ret.total}`);
    await refresh();
  } catch (e) {
    setStatus(`导入失败: ${e}`);
  }
});

document.getElementById('switch').addEventListener('click', async () => {
  try {
    const email = emailEl.value;
    if (!email) return setStatus('没有可切换账号');
    await invoke('switch_account', { email });
    await invoke('cleanup_layout').catch(() => {});
    setStatus(`已触发切换: ${email}`);
  } catch (e) {
    setStatus(`切换失败: ${e}`);
  }
});

document.getElementById('refresh').addEventListener('click', async () => {
  try {
    await refresh();
  } catch (e) {
    setStatus(`刷新失败: ${e}`);
  }
});

refresh().catch((e) => setStatus(`初始化失败: ${e}`));
