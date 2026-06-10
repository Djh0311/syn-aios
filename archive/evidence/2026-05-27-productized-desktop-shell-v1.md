# 产品化桌面壳一期 evidence

## 对象

- 任务包：`product-line/tasks/2026-05-27-productized-desktop-shell-v1.md`
- 开发线：桌面应用线
- 产物目录：`product-line/prototypes/productized-desktop-shell/`

## 这轮做了什么

- 新建产品化桌面壳一期目录，不继续把新壳命名为 probe。
- 使用 Tauri 2 + Rust + React + TypeScript + Vite 搭建应用。
- 继续读取 `product-line/prototypes/index-kernel/codex-index.json`。
- Rust 后端实现索引读取、任务队列轻量解析、路径白名单、本机路径动作。
- React 前端实现六个页面：
  - 首页总览
  - 项目页
  - 会话页
  - Skills / Plugins 页
  - 任务线 / evidence / handoff 页
  - 诊断页
- 前端实现本机动作权限确认弹层。
- 明确应用名和窗口标题。
- 输出运行说明。

## 改了哪些文件

主要新增：

- `product-line/prototypes/productized-desktop-shell/package.json`
- `product-line/prototypes/productized-desktop-shell/package-lock.json`
- `product-line/prototypes/productized-desktop-shell/README.md`
- `product-line/prototypes/productized-desktop-shell/index.html`
- `product-line/prototypes/productized-desktop-shell/vite.config.ts`
- `product-line/prototypes/productized-desktop-shell/tsconfig.json`
- `product-line/prototypes/productized-desktop-shell/src/`
- `product-line/prototypes/productized-desktop-shell/src-tauri/`
- `product-line/prototypes/productized-desktop-shell/dist/`
- `product-line/evidence/2026-05-27-productized-desktop-shell-v1.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-result.md`

运行生成：

- `product-line/prototypes/productized-desktop-shell/node_modules/`
- `product-line/prototypes/productized-desktop-shell/.cargo-home/`

说明：

- 本轮曾因相对 `CARGO_HOME` / `CARGO_TARGET_DIR` 传递不当，在新目录下误生成 `productized-desktop-shell/tauri-capability-probe/`，体积约 2.0G。已删除。
- 正式验证改用绝对路径复用 `product-line/prototypes/tauri-capability-probe/.cargo-home` 和 `.cargo-target`，没有把该缓存复制进新目录。

## 技术栈是否按任务包落地

已落地：

- Tauri 2：`tauri = 2.11.2`
- Rust：后端命令和单测在 `src-tauri/src/lib.rs`
- React：前端在 `src/`
- TypeScript：`tsconfig.json` 和 `.tsx` 视图
- Vite：`vite.config.ts`

未落地：

- React Flow：任务包说一期不要求，未实现。
- SQLite / FTS：任务包允许一期继续用 JSON，未实现。
- release 打包：任务包禁止包装成完整发布版，本轮未做。

## 桌面能力实现

后端命令：

- `load_workbench_snapshot`
- `copy_indexed_path`
- `open_indexed_project`
- `reveal_indexed_rollout`

路径限制：

- 打开目录只允许索引内 `projects[].project_root`。
- 定位 rollout 只允许索引内 `threads[].rollout_path`。
- 复制路径只允许索引内项目路径或 rollout 路径。
- 前端没有任意路径输入框。

权限提示：

- 执行动作前显示动作类型。
- 执行动作前显示目标路径。
- 执行动作前显示路径来源是索引。
- 用户确认后才调用 Rust 命令。
- Rust 命令层再次检查白名单。

## 验证结果

前端类型检查：

```bash
npm run typecheck
```

结果：通过。

前端构建：

```bash
npm run build
```

结果：通过。

构建输出：

- `dist/index.html`：0.41 kB
- `dist/assets/index-CWBKuO92.css`：6.98 kB
- `dist/assets/index-C-Jz40GM.js`：211.46 kB

Rust 后端测试：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
cargo test --offline
```

结果：通过，3 tests OK。

测试覆盖：

- 只接受索引内项目路径和 rollout 路径。
- 拒绝非白名单路径，例如 `/Users/yoyi/.codex/auth.json`。
- 构建 snapshot 时只使用索引元数据，不读取会话正文。
- 能读取真实静态索引摘要。

Tauri dev：

首次尝试：

- 沙箱内启动失败，原因：Vite 绑定 `127.0.0.1:5173` 被拒绝。

放行后：

```bash
CARGO_HOME=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-home \
CARGO_TARGET_DIR=/Users/yoyi/workspace/product-line/prototypes/tauri-capability-probe/.cargo-target \
npm run tauri:dev
```

结果：

- Vite ready：`http://127.0.0.1:5173/`
- Tauri dev 编译通过。
- 运行二进制：`codex-governance-workbench`
- 验证后已停止 dev 进程。

补充：

- 浏览器直接打开 Vite 页面会显示“不在 Tauri 窗口中运行”的错误提示，这是预期边界；真实索引读取依赖 Tauri 后端命令。
- 没有做自动 UI 点击验证打开 Finder、定位文件或剪贴板内容独立核验。

## 安全扫描

扫描范围：

- `product-line/prototypes/productized-desktop-shell/src`
- `product-line/prototypes/productized-desktop-shell/src-tauri/src`
- `product-line/prototypes/productized-desktop-shell/src-tauri/tauri.conf.json`

命中：

- `Command::new("pbcopy")`：用于复制索引内路径。
- `Command::new("open")`：用于打开项目目录和定位 rollout 文件。
- 测试里出现 `/Users/yoyi/.codex/auth.json`：用于证明非白名单敏感路径会被拒绝。

未命中：

- `auth.json`
- `.env`
- `first_user_message`
- `preview`
- `payload.content`
- rollout 正文读取

## 依赖和体积变化

新增前端依赖：

- React / React DOM
- TypeScript
- Vite
- `@vitejs/plugin-react`
- `@tauri-apps/api`
- React 类型包

体积：

- `product-line/prototypes/productized-desktop-shell/`：71M
- `node_modules/`：70M
- `dist/`：220K
- `src-tauri/`：804K
- 复用的探针 cargo 缓存：
  - `.cargo-home`：478M
  - `.cargo-target`：3.3G

说明：

- 新壳目录没有保留误生成的 2.0G 嵌套 cargo 缓存。
- release bundle 当前关闭，未生成发布包。

## 禁止事项复核

- 未写 `/Users/yoyi/.codex`。
- 未改真实 Codex 状态库。
- 未读取或展示密钥、令牌、授权文件、`.env`。
- 未展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 未自动运行 harness。
- 未实现个人知识库。
- 未实现多 agent 接入。
- 未实现向量搜索。
- 未实现模型辅助调度。
- 未实现复杂画布编排。
- 未做自动更新、系统托盘、通知、登录项。
- 未接受任意用户输入路径执行本机动作。

## 仍不确定

- Tauri 窗口内 UI 点击链条没有用自动化稳定复核。
- 剪贴板内容没有独立读取核验，避免读取系统剪贴板带出无关敏感内容。
- Finder 打开/定位未在本轮再次做 UI 点击验证。
- release 打包、签名、权限提示系统层表现未验证。

## 下一步建议

- 回收线重点检查路径白名单和权限确认弹层。
- 后续若要验收本机动作，应单独做 UI 点击验证，并明确是否允许读取剪贴板内容。
- release 打包应单独开任务，先定签名、图标、bundle 体积和权限提示标准。
