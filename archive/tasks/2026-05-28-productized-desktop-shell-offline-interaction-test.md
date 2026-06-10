# 任务包：产品化桌面壳离线前端交互测试

## 任务名

补产品化桌面壳一期权限确认弹层的离线前端交互测试。

## 所属开发线

验证线。

这是现有验证线任务，不新增常设开发线。

## 背景

当前阶段仍以治理 Codex 为主，不做个人知识库、多 agent、向量搜索、模型调度、复杂画布编排或 release 发布版。

依据：

- `product-line/STAGE_PLAN.md`：阶段 2 通过标准要求 UI 正文和按钮行为有稳定验证证据。
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-review.md`：产品化桌面壳一期 UI 行为验证已接受，但权限弹窗端到端点击证据不足。
- `product-line/tasks/README.md`：候选下一步是离线前端交互测试，验证 `PermissionDialog` 在点击动作按钮后显示动作、目标路径、路径来源。

## 目标

- 增加不依赖外网的前端交互测试。
- 覆盖点击项目页 `打开目录` 后出现 `PermissionDialog`。
- 覆盖点击项目页 `复制路径` 后出现 `PermissionDialog`。
- 覆盖点击会话页 `定位` 后出现 `PermissionDialog`。
- 覆盖点击会话页 `复制` 后出现 `PermissionDialog`。
- 验证弹层至少显示：
  - `本机动作确认`
  - 动作名称
  - `目标路径`
  - 具体路径
  - `路径来源`
  - 具体来源
  - `取消`
  - `确认执行`
- 测试只验证确认弹层出现，不实际调用后端 `open`、`pbcopy` 或 Finder 动作。
- 保持现有 Tauri 后端白名单测试继续通过。

## 允许读取

- `product-line/STAGE_PLAN.md`
- `product-line/tasks/README.md`
- `product-line/tasks/2026-05-27-productized-desktop-shell-v1.md`
- `product-line/tasks/2026-05-27-productized-desktop-shell-v1-validation.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-review.md`
- `product-line/handoffs/2026-05-27-productized-desktop-shell-v1-validation-review.md`
- `product-line/evidence/2026-05-27-productized-desktop-shell-v1-validation.md`
- `product-line/prototypes/productized-desktop-shell/`

## 允许写入

- `product-line/prototypes/productized-desktop-shell/`
- `product-line/evidence/`
- `product-line/handoffs/`

如果需要新增测试脚本或测试夹具，优先放在：

- `product-line/prototypes/productized-desktop-shell/tests/`
- `product-line/prototypes/productized-desktop-shell/scripts/`

## 禁止事项

- 不写 `/Users/yoyi/.codex`。
- 不改真实 Codex 状态库。
- 不读取或展示 `auth.json`、`.env`、密钥、令牌、授权文件内容。
- 不读取或展示 Codex 会话正文、工具输出、命令输出、输入历史或记忆正文。
- 不运行真实 harness。
- 不实际执行 Finder 打开、Finder 定位、`open`、`pbcopy` 或系统剪贴板读取。
- 不做个人知识库。
- 不做多 agent 接入。
- 不做向量搜索。
- 不做模型辅助调度。
- 不做复杂画布编排。
- 不做 release 打包、签名、自动更新、系统托盘、通知或登录项。
- 不为了测试拉取外网依赖；如果本地依赖不足，要说明缺口，不要绕过。
- 不把结果包装成完整端到端桌面 UI 自动化通过。

## 建议实现方式

优先选择不依赖外网的方式。

可选方案：

- 如果现有 `node_modules` 中已有可用测试库，使用本地依赖写组件交互测试。
- 如果没有测试库，写一个轻量离线测试脚本，使用 React / React DOM 本地依赖渲染相关组件，并模拟 `onRequestAction` 后验证 `PermissionDialog` 输出文本。
- 如果组件结构不方便测试，可以做小幅可测性改造，例如导出纯组件或增加稳定 `data-testid`，但不要改业务行为。

测试重点是前端状态链路：

- `ProjectsView` 按钮触发 `onRequestAction`。
- `SessionsView` 按钮触发 `onRequestAction`。
- `PermissionDialog` 接收 `PendingAction` 后显示正确文本。
- 点击取消后弹层可关闭。
- 不点击确认执行，不调用后端命令。

## 验收标准

- 有 evidence 和 handoff。
- 新增测试可在本机离线运行。
- 测试命令写入 evidence。
- `npm run typecheck` 通过。
- `npm run build` 通过。
- `cargo test --offline` 通过。
- 新增交互测试通过，或明确说明本地依赖不足导致无法完成。
- 测试覆盖项目页打开目录、项目页复制路径、会话页定位 rollout、会话页复制 rollout 的确认弹层。
- 不读取或展示敏感内容。
- 不实际执行系统级打开、定位、复制或剪贴板读取。
- 验证后无 dev server 或 Tauri 进程残留。

## 必须回传

1. 做了什么
2. 改了哪些文件
3. 新增了哪些测试或证据
4. 测试怎么运行
5. 哪些交互已经被离线测试覆盖
6. 哪些仍不确定
7. 是否触碰任何禁止事项
8. 验证后是否有进程或端口残留
9. 风险和下一步建议

## 总指导回收重点

回收时必须判断：

- 是否仍只治理 Codex。
- 是否真的补上权限弹窗前端交互证据。
- 是否没有实际执行系统打开、定位、复制或剪贴板读取。
- 是否没有读取或展示敏感内容。
- 是否没有引入知识库、多 agent、向量搜索、模型调度或 release 范围。
- 是否保持任务线数量不膨胀。
