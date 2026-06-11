# Root Treatment / R4-A40 Memory Knowledge Secretary Text Fixture Helper Extraction v1

日期：2026-06-12

状态：已完成，implementation / checkpoint hash 待回填。

Planning baseline commit：`dc3409686bd324bddfd5849a84ff0dc7c991896a`

Implementation commit：待回填

Review result：`STATUS: CLEAR`；P0/P1/P2 none；复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`

Checkpoint commit：待回填

本文是 Root Treatment / Stage R 的 R4-A40 任务包；R4-A40 继续对应官方计划 R4-6：离线测试拆分。R4-A40 只接受为记忆治理、记忆中心、知识库、秘书只读入口相关 expected / forbidden / class text fixture 抽离完成；不接受为 R4 完成、离线测试全部按域拆分完成、产品 UI 行为修改、真实 Tauri / 截图验收、页面真实数据来源迁移、R3 Level B、真实 Codex 执行或 backlog 功能解冻。

## 0. 全局主管理解

已知事实：

- R4-A39 已完成并回填 hash，`offline-permission-dialog.test.tsx` 当前 3,794 行。
- 主测试中记忆治理、记忆中心、知识库、秘书入口仍保留多组 inline expected / forbidden text arrays。
- 这些 arrays 是测试 fixture，不是产品行为；数据构造、summary derivation、按钮点击、action payload、render 和断言仍应留在主测试。

核心判断：

```text
R4-A40 只抽 memory / knowledge / secretary 文案 fixture；不抽 read model derivation、action payload、按钮行为、JSX render 或产品逻辑。
```

## 1. Execution Mode

Execution Mode：Supervisor-led offline text fixture helper extraction with review-line readback。

Multi-Agent Policy：

- 主管线负责任务包、实现、验证、evidence、handoff 和 checkpoint。
- 复核线继续复用只读复核线程 `019eb850-0698-7f70-a9b2-e7d0d668ccf5`，除非线程卡死或上下文污染。
- 入口文档只在验证和复核通过后的 checkpoint 同步。

## 2. Scope

允许修改：

- `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryKnowledgeTextFixtures.ts`
- `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`
- 本任务包、对应 evidence / handoff。

允许动作：

- 新增纯测试 helper，承载记忆治理、记忆中心、知识库、秘书入口的 expected / forbidden / class text arrays。
- 更新主测试引用 helper。
- 运行 offline test / typecheck / shape gate / diff check。

External changes not owned by R4-A40：

- `backlog.md`
- `docs/own-agent-and-company-vision-v1.md`

## 3. Prohibited

R4-A40 禁止：

- 修改产品代码、CSS、Rust、Tauri command、DB、sidecar schema、workflow state schema、真实执行路径或 UI 行为。
- 把 data fixture、summary derivation、JSX render、button 查找、click 流程、action payload、`assert` / `assertDeepEqual` 行为断言搬进 helper。
- 启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 执行真实 `codex exec` / `codex exec resume`，发送 prompt，读写 `/Users/yoyi/.codex`。
- 读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 纳入 `backlog.md` 或 `docs/own-agent-and-company-vision-v1.md`。

## 4. Expected Implementation

1. 新增 `offlineMemoryKnowledgeTextFixtures.ts`。
2. 抽离：
   - 工作流观察 / 正式记忆 / lint / 任务记忆包 UI expected / forbidden texts。
   - 记忆中心 expected / forbidden texts 与 expected class names。
   - lifecycle / relation / maintenance / mature pattern / quarantine dialog expected / forbidden texts。
   - 知识库 expected / forbidden texts 与候选确认弹层 expected texts。
   - 秘书摘要、右侧秘书入口 expected / forbidden texts。
3. 主测试继续保留：
   - `summarize*` / `derive*` read model 检查。
   - `MemoryCenterView` / `KnowledgeBaseView` / `SecretaryBrief` / `RightDetailPanel` render。
   - button 查找、click、pending action 和 payload 检查。
   - 所有 `assert` / `assertDeepEqual` 行为断言。

## 5. Verification

必须通过：

- `npm run typecheck`
- `npm run test:offline-interaction`
- `node scripts/harness/workbench-shape-gate.js --mode check`
- `git diff --check`

建议记录：

- `offline-permission-dialog.test.tsx` 行数变化。
- 新 helper 行数。
- shape gate 是否只有既有 warning。

## 6. Acceptance Boundary

可接受为：

- R4-6 memory / knowledge / secretary text fixture helper extraction 完成。
- 主测试继续瘦身，行为断言留在主测试。
- 复核线只读检查通过后可 checkpoint。

不可接受为：

- R4 完成。
- 离线测试全部按域拆分完成。
- 产品 UI 行为修改或视觉重做。
- 页面真实数据来源迁移完成。
- R3 Level B 或真实生产 DB read-cut。
- 真实 Codex 执行、`.codex` 读写、真实 Tauri / 截图验收。
- backlog 功能解冻。

## 7. Result

R4-A40 已完成并通过复核线 `STATUS: CLEAR`。

完成内容：

- 新增 `prototypes/productized-desktop-shell/tests/helpers/offlineMemoryKnowledgeTextFixtures.ts`，只承载 memory / knowledge / secretary expected / forbidden / class text arrays。
- 更新 `prototypes/productized-desktop-shell/tests/offline-permission-dialog.test.tsx`，将相关 inline arrays 替换为 `memoryKnowledgeTextFixtures.*`。
- 主测试仍保留 data fixture、summary derivation、JSX render、button 查找、click 流程、pending action、payload、`assert` / `assertDeepEqual` 行为断言。

验证通过：

- `npm run typecheck`
- `npm run test:offline-interaction`，`offline interaction tests passed: 14`
- `node scripts/harness/workbench-shape-gate.js --mode check`，0 errors，保留既有 warning `tauri_command_total_increased 97/96`
- `git diff --check`

行数：

- `offline-permission-dialog.test.tsx`：3,794 -> 3,576。
- `offlineMemoryKnowledgeTextFixtures.ts`：新增 269 行。

边界确认：

- 未修改产品代码、UI、CSS、Rust/Tauri、DB、sidecar schema、workflow state schema 或真实执行路径。
- 未启动 Tauri / Browser / Chrome / Vite dev / screenshot。
- 未执行真实 `codex exec` / `codex exec resume`，未发送 prompt，未读写 `/Users/yoyi/.codex`。
- 未读取 secret/token/`.env`/keychain/OAuth/provider credential/full transcript。
- `backlog.md` 和 `docs/own-agent-and-company-vision-v1.md` 是外部变更，不属于 R4-A40。
