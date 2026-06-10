# Stage K / K6 Final Tauri Dogfood Core Path Screenshot Acceptance v1

日期：2026-06-10

结论：`accepted_with_deferred_items`

本轮在 K6.2 恢复 ScreenCaptureKit window-only 截图链路之后，回到 K6 主任务继续真实 Tauri dogfood。通过 `VITE_STAGE_K_INITIAL_VIEW` 受控初始页 harness，逐页启动真实 Tauri 桌面壳，按窗口 id 捕获核心入口截图，并重新运行前端验证和 Stage K architecture gate。K6 可接受为真实 Tauri 核心入口 dogfood、截图证据和 Stage K acceptance freeze 完成；Stage K 当前冻结为 `accepted_with_deferred_items`，不能冒领为严格无缺口完成。

本轮没有执行真实 `codex exec` / `codex exec resume`，没有发送 prompt，没有读写 `/Users/yoyi/.codex`，没有读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript/rollout，没有启动 K3-B1 retry，没有启动 K3-B2，没有触发真实 retry / stop / restart / resume。

## 本轮补充实现

低风险前端 harness：

- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/vite-env.d.ts`

新增 `VITE_STAGE_K_INITIAL_VIEW`，仅用于 K6 真实 Tauri 截图验收时把桌面壳直接打开到指定一级入口。默认行为不变：未设置环境变量时仍进入首页。该 harness 不新增产品入口、不执行 Codex、不写 workflow state、不改变 sidecar schema。

## 真实 Tauri 截图

截图目录：

`evidence/tauri-verification/2026-06-10-stage-k-k6/`

核心截图均为 ScreenCaptureKit window-only 捕获，不是普通浏览器 smoke，不是全屏截图。

| 编号 | 页面 | 文件 | sha256 | 结论 |
| --- | --- | --- | --- | --- |
| 06 | 首页 | `06-home-screencapturekit-fresh-dev.png` | `9305700ec6a3651811406f24917d3e788b399f7b9c27901d714a49abe9b0ace2` | 可见首页入口和普通 UI 层级 |
| 12 | 智能体 | `12-agent-initial-view-env.png` | `b575292d555cbdaefc169f2b973ee93019c6ec168b07ae9f9e6f598147677dac` | 可见项目选择、对话选择、消息区、输入框和发送预览入口 |
| 13 | 运行中工作流 | `13-running-workflows-initial-view-env.png` | `659f1717190ef19649bfa543c5e2b0603b7e2f0f2303eed4282406b1589489d5` | 可见运行队列、失败控制、操作控制、记忆待处理摘要 |
| 14 | 项目 | `14-projects-initial-view-env.png` | `c07db2870dc44ab5850ff46e319e63ff45c8198ff646fb95019220efe00becb0` | 可见项目入口和项目列表 |
| 15 | 记忆层 | `15-memory-initial-view-env.png` | `ccb562a0fa5864b11d6338d519b477e6349146bf64c1505c7f99c98db4fdfdef` | 可见正式记忆、候选、观察、检查、维护和捕获/任务记忆包摘要 |
| 16 | 知识库 | `16-knowledge-initial-view-env.png` | `8a41e1ff93290a914e80615a0f2804ca8d7b568477ded7c1eb9ed08a2c3b380c` | 可见知识库资料、正式记忆关联、候选边界和 Obsidian-compatible 占位 |
| 17 | 设置 | `17-settings-initial-view-env.png` | `89654b50653c7ac6264b5c66c90d71f05c91473db4f58b30b1548033fd852bd7` | 可见开发者/内部边界后撤到设置页 |
| 18 | 想法箱 | `18-ideas-initial-view-env.png` | `363aad6819403b9a25d4b3eb13ad4d655fc31820773b3119ff6e16ff6eb9b573` | 可见想法入口和未接真实能力边界 |
| 19 | Skill | `19-skills-initial-view-env.png` | `e9a4c872704c9003fd59af4972a1f271f800d6e4ab543752391c27484003a18a` | 可见能力对象视角，不把内部字段铺成控制中心 |
| 20 | Harness | `20-harness-initial-view-env.png` | `4c722cee4687bc948f7e63663b972268d475d1f8c85c85485af0bfd39c6fb9a9` | 可见运行器能力、可运行范围、最近运行和待配置原因 |

视觉复核确认：

- `12-agent-initial-view-env.png` 显示智能体页是对话工作区，不是控制中心。
- `13-running-workflows-initial-view-env.png` 显示 `未知 / 不可用` 读回边界，没有把 readback unavailable 显示成真实 0 条结果。
- `17-settings-initial-view-env.png` 显示开发者入口位于设置页普通层之后。
- `18/19/20` 显示想法箱、Skill、Harness 能作为普通入口展示，并保留能力/占位边界。

## 验证

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 14`。

```text
npm run build
```

结果：通过，仅保留既有 Vite chunk-size warning。

```text
node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict
```

结果：

```text
Status: pass
Errors: 0
Warnings: 0
Info: 36
```

收尾窗口复核：

- `pgrep -laf "cargo-tauri dev|vite --host 127.0.0.1|codex-governance-workbench|npm run tauri:dev"` 无 Tauri 工作台残留。
- `ScreenCaptureKit --list --title "Codex 治理工作台"` 无 Tauri 工作台窗口残留。

## 2026-06-10 Post-Freeze Supervisor Alignment

K6 final 后，主管线做了一次只读完成态复核并发现 K3 主任务包 / K3-Level-B 字段冻结任务包仍保留中间状态措辞：

- `tasks/2026-06-10-stage-k-k3-project-workflow-real-automation-orchestration-v1.md` 原写法仍是 `Level B 待执行` / `K3 整体仍未完成`。
- `tasks/2026-06-10-stage-k-k3-level-b-real-workflow-execution-field-freeze-v1.md` 原写法仍是 `K3-Level-B 真实执行点未执行` / `K3-B 专用 bridge / harness 尚未完成`。

本次只做文档口径校准，不改产品代码、不启动 Tauri、不执行真实 Codex、不读写 `/Users/yoyi/.codex`：

- K3 主任务包已同步为：随 Stage K final freeze 收口为 `accepted_with_deferred_items`；Level A、字段冻结、K3-B0、K3-B1.0、K3-B1 失败分类、K3-B1.1 均已完成；K3-B1 retry 仍被安全审查拒绝；K3-B2 仍不得启动。
- K3-Level-B 字段冻结任务包已同步为：K3-B0 / K3-B1.0 / K3-B1.1 已完成，K3-B1 retry blocked，K3-B2 仍被 K3-B1 未成功复核阻断。

Fresh verify：

```text
node scripts/harness/stage-k-architecture-gate.js --target /Users/yoyi/workspace/product-line --strict
```

结果：通过，`Status: pass`，`Errors: 0`，`Warnings: 0`。

```text
npm run typecheck
```

结果：通过。

```text
npm run test:offline-interaction
```

结果：通过，`offline interaction tests passed: 14`。

```text
npm run build
```

结果：通过，仅保留既有 Vite chunk-size warning。

## Deferred Items

本轮不冒领以下项：

- K3-B1 retry 仍因安全审查拒绝，K3-B2 仍不得启动。
- 项目页深层 workflow canvas / node detail / task memory packet detail 未用真实 Tauri 子视图截图展开。
- 运行中工作流页的操作控制详情未逐项展开截图。
- 权限弹层未用稳定 fixture 触发截图；未为了截图触发真实 Codex 或敏感写入。
- 不接受为完整自动 UI 测试、自动 retry / stop / restart、planned adapters 真实接入、provider credential / model verification、GraphRAG / 向量库 / 图数据库或 Obsidian 原生同步。

## 接受口径

接受为：

- K6 真实 Tauri dogfood 核心入口截图验收完成。
- ScreenCaptureKit window-only 截图链路可用于真实 Tauri 验收。
- Stage K 当前完成项 / deferred 项已冻结为 `accepted_with_deferred_items`。
- 普通 UI 信息层级在真实 Tauri 壳中可见：智能体对话页、运行中工作流、项目、记忆层、知识库、设置、想法箱、Skill、Harness 均有窗口级截图证据。

不接受为：

- Stage K 严格无缺口完成。
- K3-B1 retry 成功或 K3-B2 可开始。
- 任意项目无限制自由控制台完成。
- 真实 retry / stop / restart / resume 已实现。
- planned adapters 真实接入。
- provider credential / model verification 完成。
- 自动写 FormalMemory 或自动技能化完成。
