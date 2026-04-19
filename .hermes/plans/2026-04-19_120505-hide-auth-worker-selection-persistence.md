# SeeVideo 登录体验优化方案

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** 在保持当前登录链路可用的前提下，隐藏 auth worker 窗口、在登录/切号期间用黄色状态图标闪烁提示，并让账号下拉框与自动登录都稳定记住“最后一次选中的账号”。

**Architecture:** 保持现有“main 窗口展示 Lumina + auth-worker WebView 执行登录自动化”的架构，不改登录流程本身，只补三层能力：Rust 侧 worker 可见性与状态同步、加密本地 store 持久化最后选中账号、inject.js 前端状态机与下拉框恢复逻辑。这样改动面最小，风险最低。

**Tech Stack:** Tauri v1, Rust, 注入式前端 JS, 本地 AES-GCM 账号存储。

**Current context / baseline**
- 当前 worker 仍是可见窗口：`src-tauri/src/main.rs:460-477` 明确写了 `.visible(true)` 与 `.skip_taskbar(false)`，并且日志就是 `creating VISIBLE auth worker`。
- 当前账号列表接口只返回排序后的邮箱：`src-tauri/src/main.rs:427-435`，没有“当前选中账号”概念。
- 当前 store 只保存 `accounts`：`src-tauri/src/store.rs:11-14`，没有 `last_selected_email`。
- 当前前端在启动未登录时默认选第一个账号自动登录：`src-tauri/src/inject.js:759-768`。
- 当前状态点颜色已有红/黄/绿，但黄色只是静态：`src-tauri/src/inject.js:281-283`。

---

## 方案总览

### 1. 隐藏 auth worker
- 保留独立 worker WebView，因为它已经把密码页与主窗口隔离开。
- 仅把 worker 改成真正隐藏：
  - `.visible(false)`
  - `.skip_taskbar(true)`
- 保留一个 **调试开关**（建议环境变量 `SEEVIDEO_SHOW_AUTH_WORKER=1` 或 debug_assertions 下可见），防止后续 BytePlus 登录 DOM 变化时无法排查。
- Rust 日志文案同步改掉，避免以后误判当前行为。

### 2. 黄色图标改为“闪烁提醒”
- 继续复用现有 `sv_auth_dot` / `setAuthStatus('loading')` 机制，不新增复杂 UI。
- 只给 `.status-loading` 增加 CSS animation，例如 0.8s ease-in-out 无限闪烁/呼吸。
- 触发时机保持现有逻辑：
  - `triggerSwitchAndWait(email)` 开始时置为 loading
  - invoke 失败 / poll timeout 时改 failed
  - 登录成功时改 success
- 这样视觉反馈统一，不需要再引入额外 toast 或遮罩。

### 3. 记住“最后一次选择的账号”并跨重启恢复
核心原则：**最后选中账号 = 用户最后一次手动选择，或激活后首次自动切入成功的账号。**

建议把它放到 Rust 加密 store 里，而不是只放前端 localStorage：
- localStorage 只属于远程 Lumina 页面上下文，不够稳定；站点清缓存/切域名/页面脚本异常时可能丢。
- store 本来就存放账号信息，追加一个轻量字段最稳。

建议新增字段：
- `AccountStore { accounts: Vec<LicenseAccount>, last_selected_email: Option<String> }`

新增 Rust 命令：
- `get_last_selected_account() -> Result<Option<String>, String>`
- `set_last_selected_account(email: String) -> Result<(), String>`

行为定义：
- 手动切换账号时：
  1. 先立即保存 `last_selected_email`
  2. 再触发 `switch_account`
  3. 若切换失败，下拉框仍保持该值，不自动回退
- 激活导入后自动切到首账号时：
  - 若此前没有 last_selected，则把首账号写入 last_selected
  - 若已有 last_selected 且仍存在于导入后的列表，优先恢复它
- 启动时：
  - `refreshAccountOptions()` 后先取 `get_last_selected_account()`
  - 若该邮箱仍存在于当前账号列表，下拉框选中它
  - 未登录时自动登录也优先用它，而不是 `emails[0]`
  - 只有当 last_selected 缺失或对应账号已不存在，才 fallback 到首账号

这样能满足：
- 切换后 UI 立即保持选中
- 关闭 app 再打开仍恢复同一账号
- 自动登录使用用户最后选中的账号，而不是字母排序后的第一个账号

---

## 详细改动点

### A. Rust：auth worker 隐藏 + 调试可见开关
**文件：** `src-tauri/src/main.rs`

建议改动：
- 提取一个布尔函数，例如：
  - `fn should_show_auth_worker() -> bool`
- 默认返回 false；当 `SEEVIDEO_SHOW_AUTH_WORKER=1` 或 debug 模式时返回 true。
- `WindowBuilder` 改为：
  - `.visible(should_show_auth_worker())`
  - `.skip_taskbar(!should_show_auth_worker())`
- 日志改成：
  - hidden 模式：`creating HIDDEN auth worker`
  - 调试模式：`creating VISIBLE auth worker (debug)`

**好处：**
- 生产态完全隐藏窗口
- 出问题时仍可通过环境变量打开调试

### B. Rust：store 持久化最后选中账号
**文件：** `src-tauri/src/store.rs`

建议改动：
- 扩展结构体：
  - `pub last_selected_email: Option<String>`
- 保持 `#[derive(Default)]` 兼容旧存储；旧文件反序列化时缺字段会走默认值。
- 新增辅助函数：
  - `pub fn get_last_selected_email() -> anyhow::Result<Option<String>>`
  - `pub fn set_last_selected_email(email: Option<String>) -> anyhow::Result<()>`
- `set_...` 时建议做一次归一化：trim + 非空判断 + 小写比较去重，但保存原 email 文本即可。
- 可选增强：如果设置的 email 不在 `accounts` 中，写日志但仍允许保存；启动恢复时再做存在性校验。

### C. Rust：暴露前端命令
**文件：** `src-tauri/src/main.rs`

建议新增命令：
- `get_last_selected_account`
- `set_last_selected_account`

最小接口建议：
- `get_last_selected_account() -> Result<Option<String>, String>`
- `set_last_selected_account(email: String) -> Result<(), String>`

同时更新：
- `invoke_handler(...)`
- Rust stderr 日志，方便看链路：
  - `set_last_selected_account called: ...`
  - `get_last_selected_account done: ...`

### D. 前端：黄色图标闪烁
**文件：** `src-tauri/src/inject.js`

建议改动：
- 在 `ensureStyle()` 里给 `.sv-auth-dot.status-loading` 增加动画：
  - `animation: sv-auth-pulse .8s ease-in-out infinite;`
- 新增 `@keyframes sv-auth-pulse`：
  - `0%,100%`：较亮 + 较强阴影
  - `50%`：透明度下降/缩小阴影
- failed / success 保持静态即可，只有 loading 闪烁。

### E. 前端：下拉框恢复最后选中账号
**文件：** `src-tauri/src/inject.js`

建议新增小工具函数：
- `loadLastSelectedAccount()` -> invoke `get_last_selected_account`
- `saveLastSelectedAccount(email)` -> invoke `set_last_selected_account`
- `selectAccountIfPresent(email)` -> 如果 option 存在则选中，否则返回 false

在以下位置接入：

1. `refreshAccountOptions(reason)`
- 刷新列表后不要默认停留空值
- 若有 last_selected 且列表中存在，直接 `select.value = last_selected`
- 若不存在，保持首项或空项

2. `boot()`
- 取 `emails` 后先恢复 last_selected
- 自动登录逻辑改为：
  - 优先 `last_selected`
  - 否则 `emails[0]`
- 不再硬编码 `const email = emails[0];`

3. 激活导入成功分支
- 导入后刷新账号列表
- 选中策略：
  - 优先旧的 last_selected（如果仍在）
  - 否则首账号
- 真正决定使用哪个 email 后，先保存，再 `triggerSwitchAndWait(email)`

4. 手动切换 `select.addEventListener('change', ...)`
- 用户选择后立刻 `await saveLastSelectedAccount(email)`
- 再进入 `triggerSwitchAndWait(email)`
- 切换失败时不要把下拉框改回去；保留用户选项，只更新状态点为 failed

---

## 推荐实现顺序

### Task 1: 先做 Rust store 扩展
**目标：** 先把“最后选中账号”落地，不碰 UI。

**文件：**
- 修改：`src-tauri/src/store.rs`
- 修改：`src-tauri/src/main.rs`

**完成标准：**
- 新命令可读写 `last_selected_email`
- 旧 `accounts.enc` 不报错

### Task 2: 隐藏 auth worker，并保留调试开关
**目标：** 不改登录逻辑，只改 worker 可见性策略。

**文件：**
- 修改：`src-tauri/src/main.rs`

**完成标准：**
- 默认切换账号时看不到独立窗口
- 打开调试开关后仍可看到 worker，便于排查

### Task 3: 前端恢复“最后选中账号”
**目标：** 下拉框与自动登录都优先使用 last_selected。

**文件：**
- 修改：`src-tauri/src/inject.js`

**完成标准：**
- 手动切号后，下拉框不回跳
- 重启 app 后仍选中上一次账号
- 未登录自动登录时优先最后一次账号

### Task 4: 黄色状态点闪烁
**目标：** loading 态明显提醒但不引入额外交互。

**文件：**
- 修改：`src-tauri/src/inject.js`

**完成标准：**
- 登录中黄色点闪烁
- 成功绿色静态，失败红色静态

### Task 5: 联调验证
**目标：** 确认隐藏 worker 后登录链路无倒退。

**验证场景：**
1. 已有多个账号时启动 app，确认下拉框选中 last_selected。
2. 当前未登录时启动，确认自动登录的是 last_selected，不是排序后的第一个。
3. 手动切到另一账号，确认黄色点闪烁，成功后变绿，下拉框保持新账号。
4. 关闭 app 再打开，确认仍是刚才选中的账号。
5. 调试模式打开 worker，确认必要时仍能肉眼排查登录流程。

---

## 风险与注意点

### 1. “选中账号”与“登录成功账号”可能暂时不一致
如果用户切到 B，但 B 登录失败，下拉框会保持 B，这是本方案有意保留的行为，因为用户要求“切换账号后保持选中状态”。

建议：
- 用红点表示失败，避免用户误以为已登录成功。
- 不自动回退到 A，否则会破坏“保持选中”的预期。

### 2. 旧 store 兼容性
`AccountStore` 新增字段后，要确认 serde 默认反序列化兼容旧 `accounts.enc`。

### 3. BytePlus 登录 DOM 变化
隐藏 worker 后若未来登录失效，调试会更难，所以必须保留“可见 worker 调试开关”。这不是可选项，建议一起做。

### 4. 现在 `list_accounts()` 会排序
目前邮箱列表字母排序（`emails.sort()`）没问题，但启动自动登录不能再依赖排序后的第一项，否则会覆盖用户偏好。

---

## 我建议的最终落地口径

如果按最稳方案推进，我建议这次只做三件事：
1. auth worker 默认隐藏，但保留 debug 可见开关；
2. 黄色状态点增加闪烁动画；
3. 在加密 store 中新增 `last_selected_email`，让 UI 和自动登录都优先它。

这样不需要重构登录链路，也不会动到已经“目前登录正常”的核心自动化逻辑，属于低风险增强。

---

## 验证命令

在项目根目录执行：

- `cd /Users/mapan/Desktop/SeeVideo-tauri`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- 如有前端/壳层启动命令，再按项目现有方式跑一遍 dev 模式做人工验证

---

## 当前仓库定位
- 检查基线 commit：`d6275e6`
- 本方案基于以下现状文件：
  - `src-tauri/src/main.rs`
  - `src-tauri/src/store.rs`
  - `src-tauri/src/inject.js`
  - `src-tauri/tauri.conf.json`
