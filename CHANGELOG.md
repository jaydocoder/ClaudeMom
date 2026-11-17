# 更新日志

## 未发布 · Linux 支持（jaydocoder fork）

- Linux：支持 Anthropic 官方 Claude Desktop 的 deb 安装（`/usr/lib/claude-desktop`，2026-06-30 起官方发布）。安装目录、可执行文件、`resources/`、版本号都按 Linux 布局解析，另带社区重打包版与 `~/.local/lib/claude-desktop` 的探测回退。
- Linux：Claude 桌面图标可转交 ClaudeMom 启动（`--launch-claude`），走当前选中的上游配置；用用户级 `.desktop` 覆盖，不改系统包文件，删除覆盖项即可恢复。ClaudeMom 已在托盘常驻时，新进程会通过单实例锁端口把启动请求转给常驻实例，不再静默退出。
- Linux：修复「本地模拟登录态不生效」。真实 profile 的 1p 切换原来只看状态文件 `real-profile-switch.json` 就返回，一旦该文件与实际配置不一致（例如停止后还原到 3p、但状态文件未清），就会永久卡在 3p，Claude 界面一直要求登录。现在会校验 `deploymentMode` 实际值，不一致就丢掉陈旧状态重新切换。
- Linux：本地 CA 需分别加入 Chromium 的 NSS 库（`~/.pki/nssdb`，`libnss3-tools`）和 Claude Code 所读的登录 shell 环境（`NODE_EXTRA_CA_CERTS` 写在 `~/.bash_profile`）。Node 不读 NSS 或系统 CA store；Claude 会剥离继承的同名变量，但启动 Claude Code 时会解析登录 shell 的环境。
- Linux：真实 profile 目录使用 XDG 的 `~/.config/Claude` / `~/.config/Claude-3p`；进程识别、停止和遗留代理回收使用 `pgrep -af` 与 `/bin/kill`。
- 打包：新增 `deb` 目标；fork 关闭 `createUpdaterArtifacts`。
- 新增诊断：`cargo run --example probe_linux -p claude-local`。

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
