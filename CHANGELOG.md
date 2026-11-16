# 更新日志

## 未发布 · Linux 支持（jaydocoder fork）

- Linux：支持 Anthropic 官方 Claude Desktop 的 deb 安装（`/usr/lib/claude-desktop`，2026-06-30 起官方发布）。安装目录、可执行文件、`resources/`、版本号都按 Linux 布局解析，另带社区重打包版与 `~/.local/lib/claude-desktop` 的探测回退。
- Linux：本地 CA 必须写进 Electron/Chromium 自己的 NSS 证书库（`~/.pki/nssdb`，经 `certutil`）。系统 openssl store 对它无效——只装系统 CA 的现象是 `curl` 正常、Claude 报 `-202 ERR_CERT_AUTHORITY_INVALID`。本项替代 macOS 的 `security add-trusted-cert`。依赖 `libnss3-tools`。
- Linux：NSS 只覆盖 Chromium 一侧。Electron 的 **Node 侧**（内置 Claude Code、主进程的 fetch）用 Node 自带的 145 个根证书，既不读 NSS 也不读系统 store，因此仍会报
  `unable to verify the first certificate`，界面上表现为「API Error: SSL certificate verification failed. Check your proxy or corporate SSL certificates」。
  启动 Claude 时追加 `NODE_EXTRA_CA_CERTS` 指向本地 CA，交给 Node 侧（子进程 Claude Code 一并继承）。实测：不设时 TLS 握手失败，设置后 `authorized=true`。
- Linux：真实 profile 目录改用 XDG 的 `~/.config/Claude`（和 `~/.config/Claude-3p`），不再是 `~/Library/Application Support`。
- Linux：进程识别、停止、遗留代理回收改用 `pgrep` 与 `/bin/kill`。
- 打包：新增 `deb` 目标；`createUpdaterArtifacts` 关闭（fork 没有自己的签名密钥，开着只会让构建报错）。
- Linux：真正让内置 Claude Code 可用的是**配置文件**而不是环境变量。Claude 有网络覆盖保护，会剥离继承来的
  `ANTHROPIC_*`，Claude Code 于是回落到 `https://api.anthropic.com`，经本地代理就是 SSL 报错。
  启动前把上游写进 Claude Code 的 `<code_dir>/settings.json` 的 `env`（`ANTHROPIC_BASE_URL` 指向本地
  明文端口），它就不再走 TLS，证书问题随之消失。共用 `~/.claude` 时不写，避免改动用户自己的配置。
- 新增诊断：`cargo run --example probe_linux -p claude-local` 打印解析到的安装路径与版本。

## 0.1.3 - 2026-11-15

- Windows：新版 Claude 桌面版主页没有「对话 | Cowork」切换的问题修好了，Cowork 默认打开；不想要可以在 设置 → 高级 → Cowork 关掉。
- 部分 Claude 版本没有 `ion-dist.asar`、而是把界面解包成目录，本地服务会启动失败、一直显示「本地服务未运行」；现在两种形式都支持。

## 0.1.2 - 2026-11-13

- 从 cc-switch 导入：也读取给 Claude 桌面版单独配的供应商（cc-switch 新版的「Claude Desktop」页），同一组地址和 Key 只导入一次。
- 从 cc-switch 导入：按 cc-switch 自己的规则找数据库，设置里改过配置目录的也能找到。
- cc-switch 里只有 OpenAI 格式的供应商时，说明原因，不再只提示「没有找到」。

## 0.1.1 - 2026-11-12

- macOS：改由系统（LaunchServices）启动 Claude，屏幕录制、辅助功能等隐私权限按 Claude 自己计算，不再显示「未请求」。
- 设置 → 高级 新增「新系统也开后台模式」（默认关）：Claude 在较新的 macOS 上提示「暂不支持后台模式」时，可打开它解除系统版本限制；未经 Claude 验证，可能不稳定。

## 0.1.0 - 2026-11-12

首个版本：从 CodexPlusPlus 拆出的独立程序。

- 用原版 Claude Desktop 的完整功能模式（Chat / Code / Cowork、界面汉化），直连自己的上游（Base URL + Key）。
- 多组上游随时切换；可从 cc-switch 导入 Claude 供应商，或把 Magpie / Codex++ 的本机网关当上游。
- 本地听写（SenseVoice，离线转写）。
- 自动更新。
