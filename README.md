# SeeVideo (Tauri)

离线激活码 + 账号快速切换的 Lumina 视频生成桌面壳。

- 包装页面：`https://ai.byteplus.com/lumina/model-experience/video?mode=video`
- 隐藏左侧导航与右侧作品列表，仅保留中间区域
- 应用内导入激活码（支持 1~N 个账号）
- 显示邮箱列表并一键切换账号
- 本地加密存储账号（`~/.seevideo/accounts.enc`）

## 目录

- `src-tauri/` Tauri Rust 应用
- `tools/generate_activation_code.js` 激活码生成脚本（CSV -> code）
- `keys/` 激活码加密/解密密钥（示例）

## 运行

1) 先生成激活码（示例）

node tools/generate_activation_code.js   --csv sample_accounts.csv   --public-key keys/license_pub.pem   --out activation_code.txt

2) 运行桌面应用

cd src-tauri
cargo run

## 打包（建议）

本地手动构建：

cd src-tauri
cargo build --release

GitHub 自动化构建：

- 已新增 `.github/workflows/release.yml`
- 推送 `v*` tag（如 `v0.1.0`）后会自动在 GitHub Actions 上构建：
  - macOS：`aarch64-apple-darwin` → `.dmg`
  - Windows：`x86_64-pc-windows-msvc` → `.msi` / `NSIS .exe`
- 也支持在 Actions 页面手动触发 `Release` workflow
- tag 构建完成后会自动创建 GitHub Release 并上传安装包

并确保：
- 关闭调试符号（release）
- 使用 LTO 与 `panic=abort`（已在 Cargo.toml 配置）
- 前端注入脚本在构建流程中做混淆（可新增 npm obfuscate 步骤）

## 安全说明

- 激活码内容是加密密文，不是明文账号
- 本地存储是二次加密（AES-GCM）
- 纯离线场景无法做到“绝对不可逆向”，只能显著提高逆向成本
