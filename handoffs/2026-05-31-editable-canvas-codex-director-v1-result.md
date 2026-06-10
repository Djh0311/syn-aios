# 2026-05-31 可编辑画布 v1 实施结果

对应决策：[2026-05-31-editable-canvas-codex-as-director-v1.md](../decisions/2026-05-31-editable-canvas-codex-as-director-v1.md)

## 做完了什么

### 后端（Rust，src-tauri/src/mcp/）

- `protocol.rs` — JSON-RPC 2.0 over stdio 最小实现
- `tools.rs` — MCP 工具：主管侧 6 个 + 子侧 2 个，按身份分流
  - 主管：list_team / dispatch / read_outbox / recycle / stop / finish
  - 子：submit_outbox / report_blocked
- `storage.rs` — canvas / run-state / audit / outbox 文件读写
- `codex_runner.rs` — 用 `codex exec resume -c mcp_servers.canvas.command=...` 把工作台二进制注入为 codex 的 MCP server
- `orchestrator.rs` — 后台线程跑 dispatch loop，按 state.json 决定下一步派给主管还是子
- `commands.rs` — Tauri 命令：canvas_load / canvas_save / canvas_start_run / canvas_abort_run / canvas_run_status / canvas_tick_run

### 前端（TypeScript / React）

- `lib/types.ts` — 加 CanvasDefinition / CanvasRunState / CanvasAuditEvent 等类型
- `lib/tauri.ts` — 6 个 canvas 命令的包装
- `views/CanvasView.tsx` — React Flow 画布编辑器，支持加节点 / 拉边 / 编辑角色和挂会话 / 保存 / 开工 / 拍停 / 状态轮询
- `App.tsx` — 加「画布」一栏到左侧导航
- `package.json` — 装了 `@xyflow/react`

### 子命令入口

`codex-governance-workbench __mcp_server --role director|subagent --run-id <id> [--node-id <id>]`

每次 codex 会话起来时由 codex 自己 spawn 一个本 binary 子进程，stdio 通信，会话结束 server 也死。

## 文件层布局

```
~/Library/Application Support/CodexGovernanceWorkbench/canvas-v1/
  canvas/<canvas_id>.json       画布定义
  runs/<run_id>/state.json      本次跑的状态
  runs/<run_id>/audit.jsonl     事件流
  runs/<run_id>/outbox/<node_id>.md  子的交付物
```

## 已验证

- MCP server 身份分流：subagent 调 dispatch 被拒（"工具 dispatch 不在 Subagent 的可用集合里"）。
- 端到端 stdio 通路：dispatch → submit_outbox → read_outbox → recycle → 二次 dispatch → report_blocked → finish 全跑通，状态正确翻转，6 条 audit 全记录。
- 单线锁：`busy=front` 时再 dispatch 抛 "v1 单线"。
- typecheck（npm）+ cargo build 干净。

## 还没真实跑过的

第 8 步「真起 codex 会话端到端跑一次」未做，因为：

- 需要画布上至少 2 个节点都挂上**真实**的 codex 会话 id
- 需要在 Tauri 桌面壳里点开「画布」按钮触发——CLI 测不出来
- 跑真 codex 会消耗 token，应在用户在场监督下做

## 怎么真实试一次

### 准备会话

1. 在终端起 3 个 codex 会话当成员工。每个会话至少跑一次让它有 thread_id：

   ```sh
   codex exec --skip-git-repo-check "你是车间项目主管，等待派活指令。"
   codex exec --skip-git-repo-check "你是前端子 agent，等待派活。"
   codex exec --skip-git-repo-check "你是后端子 agent，等待派活。"
   ```

2. 跑完每条会回到列表，记下 thread_id（或用工作台的 Agent 页看）。

### 在工作台搭车间

1. 起工作台：

   ```sh
   cd /Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell
   npm run tauri:dev
   ```

2. 左侧点「画布」。第一次会自动建一个空画布 `default`。
3. 左侧栏：
   - 点「+ 项目主管」加一个 director 节点。
   - 点「+ 子 agent」加 1–2 个 subagent 节点。
4. 拉边：在画布上从主管节点的边缘拉到子节点。
5. 点中节点，在「节点编辑」里：
   - 改显示名 / 技能。
   - 在「codex 会话」下拉里挑刚才记下的 session id 挂上。
6. 点「保存」。

### 开工

1. 在「开工」框里写本次目标（例：「在 README 里加一行 hello」）。
2. 点「开工」。后台会自动起主管 codex 会话，主管会调 list_team 看现状，然后调 dispatch 派活给某个子；子干完调 submit_outbox；主管再被唤醒调 read_outbox + recycle；目标达成调 finish。
3. 「拍停」按钮可以中途叫停。
4. 状态会每 2 秒轮询一次显示。

### 看发生了什么

```sh
RUN_DIR="$HOME/Library/Application Support/CodexGovernanceWorkbench/canvas-v1/runs/<run_id>"
cat "$RUN_DIR/state.json"
cat "$RUN_DIR/audit.jsonl"
ls "$RUN_DIR/outbox/"
```

## v1 留下的口子（已知，按计划留到 v2）

- 单线：busy=Some 时再 dispatch 拒绝。
- 主管视野只到 audit + state，没接 memory 层。
- 全局主管 / 项目主管两层未做，只有单层主管。
- 画布内不能新建 codex 会话，必须先在 CLI 起好再在画布里挂上。
- B 模式（流水线）未做。
- 主管会话每次 dispatch 后都靠重新唤醒同一个 codex 会话来做下一个决定——长时间跑后 transcript 会变长，可能影响主管效率（决策本身仍然依赖文件层做事实，但 codex 会话内部 context 会涨）。

## 风险 / 注意

- **主管 prompt 控制力**：主管会被告知「这次唤醒只做一个决定」，但模型实际行为仍可能多调几次工具。当前没有强制单步限制。如果发现主管乱调一通，第一招是把 prompt 收得更狠；第二招是在 server 端硬性拒绝同一次 stdio session 的第 2 个 tool/call。
- **codex MCP 注入**：`-c mcp_servers.canvas.command=<binary>` 会让 codex spawn 我们的 server。如果 codex 的 MCP client 对 server 有未声明的协议要求（资源订阅 / sampling / 进度通知），第一跑可能会出怪行为。出问题先看 `codex exec --json` 的 stderr 里有没有 mcp 相关报错。
- **会话和画布的耦合**：节点只持有 thread_id，不重新登记会话本体的元数据。如果 codex 那边把会话 archive 或删了，画布节点会变成挂着死引用。v2 应该加校验。
- **没接现有 workflow_state.json**：本系统的 canvas/run/audit 是独立的新文件层，没复用 [2026-05-28-codex-workflow-min-model.md](../decisions/2026-05-28-codex-workflow-min-model.md) 的 schema。v2 决定要不要合一。

## 下一步建议

1. 用一个**很小**的真实任务验一遍（"改个 README 标点"级别），证明完整 spawn 链能跑。
2. 如果第 1 步成立，做画布内创建 codex 会话的入口（按一下就 codex exec 一次种子 prompt + 把新 thread_id 挂回节点）。
3. 加 audit 在画布右侧栏可视化（现在只能用 cat 看）。
4. 解开单线：从 1 个 slot 改成 N 个，主管 prompt 增加并发提示。
