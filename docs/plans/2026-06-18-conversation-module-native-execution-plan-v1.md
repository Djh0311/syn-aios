# 对话模块 · 原生体验 · 完整执行计划 v1（交开发线）

日期：2026-06-18
出处：咨询线（Claude，咨询秘书）。
依据：① 已拍板的[阶段计划](2026-06-18-conversation-module-native-experience-phased-plan-v1.md)（战略/范围/分阶段）；② [codex 桌面 app 行为基线](2026-06-18-codex-native-conversation-behavior-baseline-v1.md)（UX 规格）；③ **codex/claude 公开仓库源码研究**（本文 §0 的真实契约，研究实证）。
**状态：待用户拍板 → 拍板后交开发线（执行角色）逐阶段拆任务包实现，咨询秘书复核把关。**

## 拍板摘要（先读这段）

- **这是什么**：给开发线的**完整执行计划**——把对话模块照搬 codex 做到原生。比阶段计划更细：含真实契约、关键架构决策、每阶段任务分解 + 技术做法 + 触及模块 + 验收判据。
- **范围（用户已拍）**：对话模块 · Codex-only · 回复整段（真流式 = P4）· 任意已有对话可发 + 能新建对话 · 设计取舍一律**照搬 codex**。
- **关键架构决策（源码研究后，强烈建议）**：把 relay 从「`codex exec` fire-and-forget + 读 `--output-last-message` 文件」**升级为「`codex exec --json` 的 ThreadEvent 事件流驱动」**——一举解决回复回显 + 完成检测，并为 P4 流式铺路（详见 §1·D1）。
- **代价**：5 阶段（P0 地基 → P1 核心回路 → P2 新建对话 → P3 原生渲染 → P4 流式〔本轮止于 P3〕）。每阶段 1+ 任务包，走治理。
- **谁干**：**开发线实现**；咨询秘书统筹拆包 + 复核 + 真机把关，不亲自写产品码。
- **不批 / 不碰**：不解锁 relay 以外真实执行；不动 sandbox/审批/限项目**命根子**；不碰记忆/工作流/底座/旧闸。
- **本轮交付线**：P0–P3 真机验通（任意会话可发 + 回复回显 + 新建对话 + codex 渲染范式）。P4 流式另排。

## 一句话判据

做任一步——**①照搬 codex（先查 §0 契约/基线，不自创）②没达本阶段「验收判据」不进下一阶段 ③每步真机当面验通才算完**。

---

## 0. 架构地基：codex 的真实契约（源码实证，照搬依据）

> 全部来自 `github.com/openai/codex`（Rust，`codex-rs/`）源码研究。**开发线开工前须用「当前安装的 codex 版本」实测复核**这些点（codex 迭代快，schema 可能漂移）。

### 0.1 codex 有三层集成契约（关键认知）

| 层 | 是什么 | 工作台怎么用 |
|---|---|---|
| **A. rollout JSONL** | codex 把会话逐条落盘：`~/.codex/sessions/YYYY/MM/DD/rollout-<ts>-<threadId>.jsonl`；索引在 `~/.codex/state_5.sqlite` + `session_index.jsonl` | **读历史**（工作台已在读，§0.2 是渲染依据） |
| **B. app-server JSON-RPC** | `codex exec` 内部起的 app-server，`thread/start`·`thread/resume`·`turn/start` + 流式通知 | 备选驱动面（最稳但最重） |
| **C. `codex exec --json`** | 公开 **ThreadEvent** 事件流（stdout JSONL）：`thread.started` / `turn.started` / `item.started\|updated\|completed` / `turn.completed{usage}` / `turn.failed` | **驱动 + 实时读**（本计划主用，见 D1） |

### 0.2 rollout JSONL schema（渲染依据）

- 每行：`{ "timestamp", "type": <RolloutItem>, "payload": <body> }`。顶层 `type` ∈ `session_meta` / `response_item` / `event_msg` / `turn_context` / `compacted` / `inter_agent_communication`。
- **⚠️ 双层 tag 坑**：`event_msg` 行是 `{"type":"event_msg","payload":{"type":"agent_message","message":"…"}}`（外层 + 内层 snake_case 双 tag）。**开发线务必拿真实 `~/.codex/sessions/**.jsonl` 验证后再写 parser。**
- **渲染要用的记录**：
  - `response_item` → `message`(role/content) · `agent_message` · `reasoning`(思考) · `local_shell_call`/`function_call`(+`function_call_output`，命令与输出) · `web_search_call` 等。
  - `event_msg` → `user_message` · `agent_message` · `agent_reasoning` · `patch_apply_end`(文件改动 `changes`/diff) · `token_count` · `turn_started`/`turn_complete` 等。
  - 命令的**渲染标签**照 codex 用 `parsed_cmd`（read/search/list_files…）出"已读取/已运行 N 个文件"，而非裸 argv。
- **持久化过滤**：exec begin/end、各种 delta **不落盘**；命令执行靠 `response_item` 的 `local_shell_call`/`function_call(+output)` 重建，文件改动靠 `patch_apply_end`。→ **静态历史里这些富信息齐全、可渲染（P3 能做）；实时进度/思考流要走 C 的事件流（P4）。**

### 0.3 exec / resume 机制（发送依据）

- `codex exec [OPTIONS] [PROMPT]`（PROMPT 可 stdin）。`codex exec resume <SESSION_ID|名字|--last> [PROMPT]` → 续到同一 rollout、追加写。
- 安全相关 flag（**命根子，照搬且不放松**）：`--sandbox read-only|workspace-write|danger-full-access`；`--ask-for-approval untrusted|on-failure|on-request|never`；`--dangerously-bypass-approvals-and-sandbox`(=`--yolo`，**禁用**)；`-C/--cd <DIR>`(cwd)；`--add-dir`。注：`--full-auto` 已废弃（陷阱 arg，别用）。
- `--json` → 输出 ThreadEvent 流（C）。`--output-last-message <FILE>` → 只覆盖写最终助手消息（工作台 spike 用的就是这条，信息太少，D1 要换掉）。
- **新会话**：cwd 来自 `-C/--cd`；id = UUID v7（时序）；`SessionMeta` 记 cwd/source/git{branch,commit}；每轮 model/sandbox/approval 记在 `turn_context`。

### 0.4 claude-code 旁证（次要 UX 参考）

append-only DAG（记录靠 `parentUuid` 串）；工具调用与结果分两条记录靠 id 关联；**渲染靠字形+颜色+缩进区分消息类型（非气泡）**；两层渲染（默认紧凑一行/工具结果折叠 + 展开详情视图）；权限模式 + Esc 打断 + rewind 是核心操舵。→ 印证 §0.2 的 codex 范式方向一致。

---

## 1. 关键架构决策（开发线照此实现）

- **D1 · 驱动方式换成事件流**：relay 真发改用 **`codex exec --json`**，消费 ThreadEvent：`item.completed{agent_message}` = codex 回复（**回显**）；`turn.completed` = **完成**（解锁）；`turn.failed`/`error` = 失败内联。**替代** spike 的"读 last-message 文件 + 轮询进程"。同一套事件流加细就是 P4 流式（item.started/updated 增量）。
  - 风险：须确认安装的 codex 支持 `exec --json` 且 ThreadEvent 字段稳定 → **P0 实测复核**。不支持则退回 B（app-server）或 C 不可用时临时沿用 last-message（降级，记缺口）。
- **D2 · 历史渲染照 codex 范式**：复用工作台既有 rollout 读取（`codex_transcript`），**按 §0.2 把 `response_item`/`event_msg` 映射成「平铺 markdown 文本 + 灰色工具状态行 + 思考块 + 压缩分隔线」**，弃聊天气泡。两层渲染（默认紧凑 + 展开详情）照 claude/codex。
- **D3 · 会话/新建模型照 codex**：续聊=resume by threadId（每条已有对话都可发，不靠"有没有 cwd"置灰——cwd 从该会话 rollout 的 `SessionMeta.cwd` 拿）；新建=内联选「项目 cwd · 执行模式 · 分支」（基线 §3），`-C/--cd` 起新 thread。
- **D4 · 命根子不动**：无论走 C/B，codex argv 仍 `--sandbox workspace-write` + `--add-dir 项目根` + 不传 `--yolo`/`--full-auto`/危险 `--ask-for-approval`；真发不可逆性质不变，沿用现有 relay 安全壳与回执审计。

---

## 2. 分阶段执行（每阶段：目标 / 任务分解 / 技术做法 / 触及 / 验收 / 治理）

### P0 · 地基：契约实测 + 既有资产盘点〔咨询秘书可参与，开发线主力实测〕

- **目标**：把 §0 契约在「当前安装的 codex」上实测钉死，盘清工作台既有可复用件，产出落地接口约定。
- **任务**：
  1. 实测 codex：跑 `codex exec --json` 看真实 ThreadEvent；取一条真实 rollout `.jsonl` 验证 §0.2 双层 tag 与字段；确认 resume / `--sandbox` / `--add-dir` 行为。
  2. 盘点工作台既有：`codex_transcript`（rollout 读取到哪步）、`manual_relay.rs`（安全壳/argv 构造）、会话列表（已按项目分组）、`AgentConversationShell`/`AgentChatComposer`。
  3. 产出「对话回路接口约定」：读(rollout 事件→视图模型)/发(`exec --json` argv + stdin)/回(ThreadEvent→消息)/状态机(草稿→发出→运行→完成/失败/停止 + 锁解时机)。
- **验收**：契约文档 + 一份"codex 实测纪要"（真实事件样本/字段确认/版本号）。
- **治理**：契约经咨询秘书 + 用户过目（接口是"先定接口"防劈两半的关键）。

### P1 · 核心回路：任意会话可发 + 回复回显 + 发完解锁〔最高优先〕

- **目标**：杀三条硬伤（只一条能发 / 发完不回显 / 发完锁死）。
- **任务分解**（可拆 2 个任务包：后端驱动 / 前端回路）：
  - 后端：① 放宽绑定——发送目标 = 选中会话的 `thread_id` + 其 rollout `SessionMeta.cwd`，**每条已有 codex 对话都可发**（去掉"无 project_root 即置灰"）。② 按 D1 用 `codex exec --json` resume 发送，消费 ThreadEvent，回执含 codex 回复文本 + 终态；保留 stop（D4 安全壳/argv 不动）。
  - 前端：发送 → 乐观显示用户消息 → "codex 运行中（可停）" → 收 `turn.completed` 把回复落进对话 → **自动解锁** → 可再发；撰写区去开发者字段墙；错误内联。
- **技术做法**：驱动走 D1；回复来自 `item.completed{agent_message}`；完成来自 `turn.completed`。（若 P0 判定 `--json` 不可用 → 降级方案另记。）
- **触及**：`manual_relay.rs`（+ 命令）、`commands.rs`/`command_registry.rs`、`tauri.ts`、`AgentConversationShell.tsx`、`AgentChatComposer.tsx`、相关 types。
- **验收（真机当面）**：任挑一条 codex 对话 → 发一句 → 看见自己消息 + codex 回复落进对话 → 解锁 → 连发第二句。
- **治理**：TDD（后端驱动/状态机）；独立复核 + 咨询审实物 + 真机验收；用户拍板才 commit。

### P2 · 新建对话（照搬 codex 内联模型）

- **目标**：像 codex 点"新建"起一条全新会话并能连聊。
- **任务**：① 后端 new-session 真发（`codex exec --json` 不带 resume、`-C/--cd 选定项目`，复用 D4 安全壳）。② 前端"新建对话"入口 + 内联选择器「项目 cwd ▾ · 执行模式 ▾ · 分支 ▾」（照基线 §3），新 thread 起好后成为当前线程入列表。
- **技术做法**：新 thread_id 由 codex 生成（UUID v7），首条 `thread.started` 拿到 id 后绑定该会话。
- **触及**：同 P1 + 会话列表/新建入口。
- **验收（真机）**：点新建 → 选项目 → 发 → 得回复 → 该会话落盘、列表可再找到并续聊。
- **治理**：同 P1。

### P3 · 原生渲染范式（照搬 codex，差距最大）

- **目标**：把对话渲染从"气泡 + 开发者字段"换成 codex 范式。
- **任务**：① 复用 `codex_transcript` 读 rollout，按 §0.2 映射渲染：**平铺 markdown 文本 + 灰色工具状态行（已读取/已运行/正在编辑 N + 命令，用 `parsed_cmd` 标签）+ 思考块 + "上下文已自动压缩"分隔线**。② 两层渲染（默认紧凑/工具结果折叠 + 展开详情）。③ 发完重读该会话 rollout 取"规范线程"（去重乐观气泡）。④ 会话列表/切换/停止/重发/错误态/空态/加载态打磨；inline code/代码块/diff 渲染。
- **技术做法**：渲染只吃**已落盘事件**（D2）；diff `+/-` 从 `patch_apply_end.changes` 派生。
- **触及**：`TranscriptViews.tsx`、`AgentConversationShell.tsx`、`conversationEngine.ts`、`codex_transcript`（按需补事件类型）、样式。
- **验收（真机 + 清单）**：与 codex 并排，对话渲染"像原生"（一份逐条清单：文本/工具行/思考/压缩/diff/折叠）。
- **治理**：UI 浏览器验证 + 真机；独立复核 + 咨询审。

### P4 ·〔本轮不做，下一阶段〕真流式

- 逐字流式 + 实时步数/计时/思考/工具进度。走 D1 同一事件流的 `item.started/updated` + `*_delta` + `turn_started`(步数/计时)。运行态富信息（步数药丸/目标条/思考流）照基线 §5。**按用户拍板本轮止于 P3，P4 另排。**

---

## 3. 横切关注点

- **安全（贯穿）**：D4 命根子不动；真发不可逆，沿用现有授权与回执审计；不解锁其他真实执行；不碰记忆/工作流/底座/旧闸。每任务包重申。
- **治理**：每阶段任务包 → 开发线实现 → 独立复核线 → 咨询线审实物 → 用户拍板 commit；子线不 commit。
- **真机硬规矩**：每阶段"验收判据"必须真桌面 app 当面验通才算完——不接受 offline 测完就报。
- **与总路线图**：本计划 = master-roadmap A线③b-2「GUI 真用」+ B线「对话」聚焦细化；推进回写 master-roadmap/CURRENT。
- **照搬纪律**：拿不准就查 §0 契约 / 基线 / 现看 codex；不自创。

## 4. 风险与缺口（诚实）

- **R1·codex 版本漂移**：§0 schema/flag 取自 `main`，安装版可能不同 → P0 必实测；`--json` 不支持则退 B 或降级（记缺口）。
- **R2·双层 tag parser 坑**：§0.2 务必真实文件验证后再写。
- **R3·照搬工作量**：codex 对话 UX 厚，P3 是实打实重做，非贴皮——别低估。
- **R4·实时富信息属 P4**：本轮（P1–P3）静态历史富信息可达，运行态实时（步数/计时/思考流）留 P4。

---

*本文为待拍板的完整执行计划。拍板后交开发线按 P0→P3 逐阶段拆任务包实现，咨询秘书复核把关、每阶段真机验收。P4 流式另排。设计总则：照搬 codex。*
