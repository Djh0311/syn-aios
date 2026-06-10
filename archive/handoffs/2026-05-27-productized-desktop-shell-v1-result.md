# 产品化桌面壳一期交接

## 回收对象

- 任务包：`product-line/tasks/2026-05-27-productized-desktop-shell-v1.md`
- 开发线：桌面应用线
- Evidence：`product-line/evidence/2026-05-27-productized-desktop-shell-v1.md`
- 产物目录：`product-line/prototypes/productized-desktop-shell/`

## 结论

产品化桌面壳一期已完成到可回收状态。

它可以接受为“产品化桌面壳一期”，但不能接受为“完整桌面应用发布版”。

## 先说薄弱点

- UI 点击链条没有做稳定自动化复核。依据：本轮只验证 Tauri dev 启动、构建和后端测试。
- 剪贴板内容没有独立核验。依据：读取系统剪贴板可能带出无关敏感内容，本轮没有做。
- Finder 打开目录和定位文件没有本轮重新做按钮点击验证。依据：本轮重点是产品壳结构、权限弹层和构建验证。
- release bundle 关闭。依据：`src-tauri/tauri.conf.json` 中 `bundle.active=false`。
- 依赖体积大。依据：`node_modules` 70M，复用 cargo 缓存 `.cargo-home` 478M、`.cargo-target` 3.3G。

## 已完成

- 建立 `product-line/prototypes/productized-desktop-shell/`，不再把新壳叫 probe。
- 使用 Tauri 2 + Rust + React + TypeScript + Vite。
- 正式应用名：`CodexGovernanceWorkbench`。
- 窗口标题：`Codex 治理工作台`。
- 实现 6 个一期页面：
  - 首页总览
  - 项目页
  - 会话页
  - Skills / Plugins 页
  - 任务线 / evidence / handoff 页
  - 诊断页
- Rust 后端保留路径白名单。
- 前端实现本机动作权限确认弹层。
- 后端实现打开索引内项目目录、复制索引内路径、定位索引内 rollout 文件。

## 技术栈是否按任务包落地

已按任务包落地：

- Tauri 2
- Rust
- React
- TypeScript
- Vite

未进入一期：

- React Flow
- SQLite / FTS
- 个人知识库
- 多 agent 接入
- 向量搜索
- 模型调度
- 复杂画布编排

## 验证

通过：

- `npm run typecheck`
- `npm run build`
- `cargo test --offline`，复用探针 cargo 缓存，3 tests OK
- `npm run tauri:dev`，放行本地 dev server 后能启动 Tauri dev，运行二进制 `codex-governance-workbench`

说明：

- 首次沙箱内 `tauri:dev` 失败，原因是 Vite 无法绑定 `127.0.0.1:5173`。
- 放行后 Tauri dev 可运行。
- 验证后已停止 dev 进程。

## 桌面能力状态

已实现并有后端测试支撑：

- 路径白名单。
- 复制索引内路径。
- 打开索引内项目目录。
- 定位索引内 rollout 文件。

已实现但未做本轮稳定 UI 点击复核：

- 权限确认弹层后触发本机动作。

仍不确定：

- 剪贴板内容是否确实变更。
- Finder 打开/定位在所有项目路径上的表现。
- release 打包后的权限提示和系统行为。

## 安全边界

符合当前任务包：

- 仍只治理 Codex。
- 不写 `/Users/yoyi/.codex`。
- 不改 Codex 状态库。
- 不读取或展示密钥、令牌、授权文件、`.env`。
- 不展示会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不接受任意用户输入路径执行本机动作。
- 不做知识库、多 agent、向量搜索和模型调度。

## 依赖和构建产物

新壳目录：

- 总体：71M
- `node_modules`：70M
- `dist`：220K
- `src-tauri`：804K

复用探针缓存：

- `.cargo-home`：478M
- `.cargo-target`：3.3G

本轮曾误生成嵌套缓存：

- 路径：`product-line/prototypes/productized-desktop-shell/tauri-capability-probe/`
- 体积：约 2.0G
- 状态：已删除

## 建议回收判断

建议接受为产品化桌面壳一期。

不要接受为完整发布版。

下一步建议单独派验证或 release 任务：

- UI 点击验证本机动作。
- 剪贴板核验是否允许。
- release 打包、签名、图标和系统权限提示。
- 是否清理或外置 `node_modules`、cargo target 缓存。
