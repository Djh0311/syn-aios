# 任务包：产品化桌面壳一期 UI 行为验证

## 任务名

验证产品化桌面壳一期的 Tauri 窗口、页面切换、本机动作确认弹层和安全边界。

## 所属开发线

验证线。

这是现有验证线任务，不新增常设开发线。

## 背景

当前阶段仍以治理 Codex 为主，不做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排或 release 发布版。

依据：

- `product-line/STAGE_PLAN.md`：阶段 2 通过标准要求桌面应用能运行、本机动作只能对索引内路径执行、UI 正文和按钮行为有稳定验证证据。
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-review.md`：总指导线已接受产品化桌面壳一期，但明确 UI 点击链条、剪贴板内容、Finder 打开/定位和 release 打包仍未完成稳定验证。
- `product-line/tasks/README.md`：候选下一步是产品化桌面壳一期 UI 行为验证，派给验证线。

## 目标

- 验证 `product-line/prototypes/productized-desktop-shell/` 能启动 Tauri 窗口。
- 验证窗口正文能读取静态索引，并显示项目、会话、skills、plugins 等摘要。
- 验证 6 个页面能切换：
  - 首页
  - 项目
  - 会话
  - Skills / Plugins
  - 任务线 / 证据
  - 诊断
- 验证打开项目目录、复制路径、定位 rollout 之前会出现确认弹层。
- 验证确认弹层至少显示动作类型、目标路径、路径来源。
- 验证后端拒绝非白名单路径。
- 验证页面不展示会话正文、工具输出、命令输出、输入历史、记忆正文、密钥、`.env`、授权文件内容。
- 验证后清理本轮启动的 dev server 和 Tauri 进程，并复核 5173 没有监听残留。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-27-productized-desktop-shell-v1.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-review.md`
- `product-line/evidence/2026-05-27-productized-desktop-shell-v1.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-result.md`
- `product-line/prototypes/productized-desktop-shell/`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/tauri-capability-probe/.tauri-cli/`
- `product-line/prototypes/tauri-capability-probe/.cargo-home/`
- `product-line/prototypes/tauri-capability-probe/.cargo-target/`

## 允许写入

- `product-line/evidence/`
- `product-line/handoffs/`
- 如必须新增验证脚本，可写入 `product-line/prototypes/productized-desktop-shell/tests/` 或 `product-line/prototypes/productized-desktop-shell/scripts/`。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不做个人知识库。
- 不做多 agent 接入。
- 不做向量搜索。
- 不做模型辅助调度。
- 不做复杂画布编排。
- 不做 release 打包、签名、自动更新、系统托盘、通知或登录项。
- 不接受任意用户输入路径执行本机动作。
- 不用真实敏感路径做打开、复制或定位动作；非白名单拒绝测试只允许走后端单测或安全桩，不要实际打开敏感文件。
- 不为验证剪贴板而读取系统剪贴板内容，除非用户另行明确允许。

## 建议验证方式

- 先复跑：
  - `npm run typecheck`
  - `npm run build`
  - `cargo test --offline`
- 再启动 Tauri dev。
- 优先使用可自动化的窗口或浏览器检查；如果当前环境无法稳定自动化 Tauri 窗口，要明确写出不能自动化的原因。
- 可以通过页面状态、可见文本、截图、DOM 检查或 Tauri 后端命令返回值建立证据。
- 本机动作验证以“确认弹层出现”和“后端白名单拒绝非白名单路径”为主；Finder 打开和剪贴板内容不是本任务必须独立核验项。

## 验收标准

- 有 evidence 和 handoff。
- 明确说明验证命令、结果和失败原因。
- `npm run typecheck` 通过，或说明失败原因。
- `npm run build` 通过，或说明失败原因。
- `cargo test --offline` 通过，或说明失败原因。
- 能证明 Tauri 窗口正文读取到了索引摘要，或说明为什么当前环境无法证明。
- 能证明 6 个页面可切换，或说明为什么当前环境无法证明。
- 能证明本机动作前有确认弹层，或说明为什么当前环境无法证明。
- 能证明后端拒绝非白名单路径。
- 验证结束后清理本轮启动的进程，并复核 5173 无监听残留。
- 不把结果包装成完整桌面发布版。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些测试或证据
4. 哪些 UI 行为已验证
5. 哪些本机动作已验证
6. 哪些能力仍不确定
7. 是否触碰任何禁止事项
8. 验证后是否有进程或端口残留
9. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否仍只治理 Codex。
- 是否补上产品化桌面壳一期 UI 行为证据。
- 是否保持路径白名单和权限确认。
- 是否没有读取或展示敏感内容。
- 是否没有引入知识库、多 agent、向量搜索、模型调度或 release 发布范围。
- 是否清理验证进程。
