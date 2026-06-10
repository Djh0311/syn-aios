# 任务包：产品化桌面壳一期

## 任务名

把 Tauri 最小能力探针推进为 Codex 治理工作台的产品化桌面壳一期。

## 所属开发线

桌面应用线。

这是阶段 2 桌面应用壳任务，不新增常设开发线。

## 背景

当前阶段仍以治理 Codex 为主，不做个人知识库、多 agent、向量搜索、模型调度和复杂画布编排。

依据：

- `product-line/STAGE_PLAN.md`：阶段 2 目标是本地桌面应用壳，展示项目、会话、skills、任务线状态，并支持打开文件夹、复制路径、定位日志。
- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`：阶段 2 / 第一版推荐 Tauri 2 + Rust + React + TypeScript + Vite，React Flow、SQLite、FTS 是后续稳定化方向。
- `product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-review.md`：Tauri 最小能力探针已阶段性通过，但不是完整桌面应用。
- `product-line/handoffs/2026-05-27-tauri-ui-action-validation-recovery-result.md`：Tauri 探针已覆盖窗口、索引读取、路径白名单、复制调用、打开目录、定位文件的阶段性验证。
- `product-line/tasks/README.md`：当前暂无阻塞任务；下一步应派产品化桌面壳任务包，明确权限提示、路径展示策略、正式应用名和 release 打包边界。

## 目标

- 建立产品化桌面壳一期目录，不继续把原型叫 probe。
- 使用 Tauri 2 + Rust + React + TypeScript + Vite。
- 继续读取当前只读索引：`product-line/prototypes/index-kernel/codex-index.json`。
- 保留 Rust 后端路径白名单策略。
- 实现 Codex 治理工作台的一期页面：
  - 首页总览
  - 项目页
  - 会话页
  - Skills / Plugins 页
  - 任务线 / evidence / handoff 页
  - 诊断页
- 实现正式权限提示：
  - 打开项目目录前显示目标路径。
  - 定位 rollout 文件前显示目标路径。
  - 复制路径后显示明确反馈。
  - 所有本机动作都必须由用户点击触发。
- 明确正式应用名、窗口标题和原型运行方式。
- 输出 evidence 和 handoff。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/README.md`
- `product-line/PROTOTYPE_WORK_LINES.md`
- `product-line/DEV_LINES.md`
- `product-line/decisions/2026-05-27-technical-stack-and-expansion-architecture.md`
- `product-line/decisions/2026-05-27-desktop-container-route.md`
- `product-line/prototypes/index-kernel/codex-index.json`
- `product-line/prototypes/desktop-app/`
- `product-line/prototypes/tauri-capability-probe/`
- `product-line/tasks/README.md`
- `product-line/handoffs/2026-05-27-tauri-min-prototype-after-cli-review.md`
- `product-line/handoffs/2026-05-27-tauri-ui-action-validation-recovery-result.md`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/`
- `product-line/evidence/`
- `product-line/handoffs/`

如必须复用 Tauri CLI 缓存或本地依赖，可以读取 `product-line/prototypes/tauri-capability-probe/.tauri-cli/` 和 `.cargo-home/`，但不要把缓存复制进新目录，除非 evidence 说明原因和体积影响。

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件。
- 不展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不自动运行 harness。
- 不做个人知识库。
- 不做多 agent 接入。
- 不做向量搜索。
- 不做模型辅助调度。
- 不做复杂画布编排。
- 不把当前结果包装成“完整桌面应用发布版”。
- 不做自动更新、系统托盘、通知、登录项。
- 不接受任意用户输入路径执行本机动作；动作路径必须来自索引内已有路径。

## 建议实现边界

一期可以继续使用 `codex-index.json`，不强行改 SQLite。

一期不要求 React Flow。可以预留“关系视图”入口，但不要实现复杂画布。

推荐结构：

- `src-tauri/`：Tauri + Rust 后端命令。
- `src/`：React + TypeScript 前端。
- `src/lib/`：索引类型、路径动作封装、格式化工具。
- `src/views/`：页面。
- `src/components/`：通用组件。

后端命令至少包括：

- 读取索引摘要。
- 读取项目列表。
- 读取会话列表。
- 读取 skills / plugins。
- 读取任务线入口元数据。
- 复制索引内路径。
- 打开索引内项目目录。
- 定位索引内 rollout 文件。

权限提示至少覆盖：

- 显示动作类型。
- 显示目标路径。
- 显示路径来源是索引。
- 用户确认后才执行。

## 验收标准

- 有可运行命令。
- 有正式应用名和窗口标题，不再显示为调试探针名称。
- React + TypeScript + Vite 前端能构建或通过类型/语法检查。
- Rust 后端测试通过。
- Tauri dev 能启动窗口。
- 首页能显示项目数、会话数、skills 数、plugins 数。
- 项目页和会话页能显示索引元数据，不显示正文。
- 本机动作只允许索引内路径。
- 权限提示存在，并且执行动作前可见。
- 不写 `.codex`。
- 不展示密钥。
- 不自动运行 harness。
- 有 evidence 和 handoff。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些测试或证据
4. 技术栈是否按任务包落地
5. 哪些桌面能力已验证
6. 哪些能力仍不确定
7. 是否触碰任何禁止事项
8. 依赖和构建产物体积变化
9. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否仍只治理 Codex。
- 是否把探针升级成产品壳，而不是继续堆 probe。
- 是否保持路径白名单。
- 是否有权限提示。
- 是否有稳定验证。
- 是否没有引入知识库、多 agent、向量搜索和模型调度。
