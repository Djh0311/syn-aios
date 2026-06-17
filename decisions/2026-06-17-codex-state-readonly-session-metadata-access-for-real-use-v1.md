# 决策：真实使用允许产品只读 Codex 会话元数据（窄化 RU「不碰 .codex」封印）v1

日期：2026-06-17

状态：**accepted（用户 2026-06-17 拍板）**

## 拍板摘要

- **要批的事**：把 RU 开发期定的「绝不碰 `.codex`」一刀切，**窄化**为三档——
  1. **真实使用（你在场、产品 GUI）允许只读列你自己的 Codex 会话元数据**：会话 id / 标题 / 工作目录 cwd / 更新时间 / model / reasoning effort / 来源 / rollout **路径**。
  2. **transcript 正文查看、auth、token、secret 仍是单独的严格门**（本决策不放行）。
  3. **开发期的 agent（Codex / Claude）仍不得擅自读 `.codex`**，需逐次取得你明确授权（与现状一致）。
- **代价**：几乎为零——这本就是产品已发布的设计（`codex_db.rs` 只读 `~/.codex/state_*.sqlite` 列你的会话），是你自己机器上你自己的数据、只读、你在场。
- **不批的后果**：工作台默认一打开就会读 `state_*.sqlite`，在「不碰 .codex」下 **GUI 真用永远没法做**，你最初要的「驾驶舱顺不顺手」永远验不了；且一个「Codex 工作台」被禁止读你自己的 Codex 会话，与产品命根子自相矛盾。

## 一句话判据

判某次 `.codex` 访问是否在本决策放行内——问：**「是不是产品在你在场时、只读列你自己会话的元数据（标题 / 目录 / 时间 / 模型 / rollout 路径）？」** 是 → 放行；只要涉及**读 transcript 正文 / auth / token / secret**，或是 **dev agent 擅自读**，→ 不放行，走单独门 / 单独授权。

## 背景与实物依据

- RU 去险撞墙：默认 snapshot / 页面读模型 `load_workbench_snapshot` → `build_snapshot`（`RealWithSqliteFallback`）→ `load_sessions_from_sqlite_or_index` → `codex_db::default_state_db_path()` → 读 `~/.codex/state_*.sqlite`。
- 咨询线核源码 `codex_db.rs` `read_threads()`：以 `SQLITE_OPEN_READ_ONLY` 打开，`SELECT id, title, cwd, updated_at_ms, archived, rollout_path, model, reasoning_effort, thread_source FROM threads WHERE has_user_event=1`；另读 `session_index.jsonl`（id→标题）。
- **不读**：auth.json / token / 凭据；transcript（rollout）正文；prompt body。仅取**会话元数据 + rollout 路径字符串**。
- 敏感性提示：会话**标题**可能由首条消息自动生成、**cwd** 暴露工作目录——属元数据但非零敏感；「看某条 transcript 正文」是另一条带 `viewer_boundary` 遮挡层的深路，**开 app 不触发**。

## 决定（待拍）

1. **放行**：真实使用时，产品可 read-only 读 `~/.codex/state_*.sqlite`（threads 表上述列）与 `session_index.jsonl`，用于列出 / 分组你自己的会话。
2. **不放行（维持严格门）**：transcript 正文读取、auth / token / secret / `.env` / keychain / OAuth / 凭据 / prompt body；任何写 `.codex`；真实 Codex 执行（K3-B1 / B2）。
3. **dev agent 边界不变**：Codex / Claude 在开发任务中**仍不得擅自读 `.codex`**；需要时逐次取得用户明确授权。
4. **触发动作**：本决策一旦拍板，解锁「真·GUI 真用复核」窄任务（RU1 的真正版本）——你在场、`tauri dev` 真打开、真用 mario test、看驾驶舱体感、GUI 内亲手走记忆闭环。

## 不接受为

- 不接受为允许读 transcript 正文 / auth / token / secret。
- 不接受为 dev agent 可随意读 `.codex`。
- 不接受为允许写 `.codex` 或解锁真实 Codex 执行 / K3-B1 / B2 / 真库 C 切换。
- 不接受为 RU 已整段完成或驾驶舱体感已验。
