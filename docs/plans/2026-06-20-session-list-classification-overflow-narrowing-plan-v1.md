# 会话列表分类过滤 + 标题/全局溢出 + 侧栏收窄 方案 v1

> 日期：2026-06-20 · 作者：主导线 · 状态：待执行线落地
> 一句话判据：① 会话列表读取层加一道 subagent 过滤；② 标题与全局文本不许溢出框；③ 两处侧栏收窄。纯前端 + 一处后端读查询，不碰安全闸/发送回路。

## 0. 缘起

用户真机：syn 会话列表"乱七八糟"、比 codex 多得多。主导线读 codex 自己的 `~/.codex/state_5.sqlite` 实测定因（下面数字均为 2026-06-20 实查）。

## 1. 会话列表分类过滤（subagent）——决定：**全部隐藏**

### 证据（实查 state_5.sqlite）
- 工作台口径（`has_user_event=1 AND archived=0`）共 **604** 条，散在 46 个项目目录。
- 按 `source` 列拆：**顶层会话 136**（vscode 125 / exec 10 / cli 1）+ **subagent 子线程 468**（`{"subagent":{"other":"guardian"}}` 156 + 一堆 `{"subagent":{"thread_spawn":...}}` 的 explorer/worker）。
- **syn 实际可见窗口（最近 100 条）里：subagent 噪声 89 条，真会话只有 11 条。** ← 这就是"乱七八糟"的真因。
- codex 主列表只显顶层会话，**不显 subagent**。工作台过滤没排除 subagent → 噪声铺满可见窗口。

### 决定（用户拍板）
**subagent 全部隐藏、不需要看到**（不收纳、不留入口；将来要看子 agent 是另立视图的事，本方案不做）。

### 修法
- **在读取层加过滤**：`source` 不含 subagent 才进列表。
- **落点（指针，执行线重核）**：后端 `src-tauri/src/codex_db.rs:177` 那条 `WHERE has_user_event = 1 {archived_clause}` → 加 `AND source NOT LIKE '%subagent%'`。
- **为什么放后端不放前端**：① 直接砍 604→136 行；② 顺带不再把那 468 条的标题（见 §2，标题巨大）拉进 payload，加载也快。
- **判据说明**：`source` 是 JSON（`{"subagent":...}`）或纯串（`vscode`/`exec`/`cli`）；观测到 subagent 全部带 `subagent` 键，真会话 source 不含该词，`NOT LIKE '%subagent%'` 安全。要更稳可解析 JSON 判 `subagent` 键，执行线定。
- 过滤后：列表 = 136 条真会话，页大小 100（`codex_db.rs:48`）首页装 100、其余分页。项目 thread_count 也随之归正（不再被子线程灌水）。

## 2. 标题 / 全局溢出

### 证据
- `threads.title` 存的是**整条首条消息原文**：平均 **10,989 字符**、最长 **76,429 字符**，604 条里 382 条超 120 字。未重命名就铺满屏。

### 修法（三层）
1. **显示截断**：左栏会话标题 + 对话页顶栏标题 → `text-overflow:ellipsis` / `line-clamp` + `max-width`，超出显 `···`。落点 `src/views/agent/AgentSessionList.tsx:300-304`（`sc-title`）+ 对话页顶栏标题元素。
2. **全局兜底**（用户："所有信息不应超出框架"）：扫一遍框架级容器，统一 `min-width:0` + `overflow` 约束，所有平铺文本（标题/路径/标签）都不许撑破框。已有不少 `min-width:0`，但标题这类没截断——本层补齐，并定一条全局规则：列表项/卡片内文本默认截断、不撑容器。
3. **读模型截短 title**（治本+省 payload）：后端把 `title` 映射成 SessionRecord 时截短（取首行 / 前 N 字，保留完整原文另存或不传）。落点 `codex_db.rs` 的行→SessionRecord 映射（约 :237-262）。否则 136×可能上万字标题仍是几 MB 负载。

## 3. 侧栏收窄

- **app 左导航栏**继续收窄；**对话页左侧会话栏**也收窄。
- 纯 CSS 宽度，落点 `src/styles.css` 对应 `width`（执行线按现状定位具体类，别误伤别的栏）。
- 收窄后注意 §2 的标题截断要同步生效（栏更窄、更易溢出）。

## 4. 边界 / 高危

- **轻档**。#1 是 codex sqlite 的**只读查询**加一个 WHERE 子句——**不是安全闸 / 不碰高危清单**；#2/#3 纯前端。
- **不碰发送回路 / manual_relay / 任何 `.rs` 安全逻辑**；#1 那处 `codex_db.rs` 仅读查询过滤。
- 范围只限：会话列表读取 + 标题/溢出 CSS + 两处侧栏宽度。别顺手动别的视图。

## 5. 验证（完成必附真证据）
- **后端**：加过滤后重跑 codex_db 相关测试；或直接对 `state_5.sqlite` 跑加了 `AND source NOT LIKE '%subagent%'` 的同款查询，确认计数 604→136、最近 100 里 0 subagent。
- typecheck + `test:offline-interaction` 绿；加断言：① 列表读模型不含 source 带 subagent 的项；② 超长标题渲染后 DOM 文本截断（含 `···`/`line-clamp` 类）、不撑破容器宽度。
- **真机**（执行线/用户）：syn 列表只剩真会话、无 subagent；超长标题左栏/顶栏都截断不铺屏；两侧栏更窄且不溢出。
- 扫 diff：除 `codex_db.rs` 那一处读查询过滤外无后端改动；没碰发送回路。

---

## v1.1 真机回归修订（2026-06-20 · 主导线 · 主机验后）— 本节优先级高于上文

> 用户真机：#1 subagent 过滤已生效 ✓。新增两处：无项目会话统一 + 侧栏再收窄。

### A. 无项目会话统一（新功能）

- **现状 bug**：syn 在 `codex_db.rs:262` 一律**按 cwd 推 `project_root`**，每条会话都安到一个项目；codex 把"没指定项目"的直接聊天**统一成一个列表**。syn 没有这个统一桶。
- **实查数据（已用真实样本定信号，非猜）**：用户指认 `/Users/yoyi/Documents/Codex/` 下基本都是直接聊天。实查：该目录下 **24 条**——全是 `2026-05-xx/new-chat`、`ai`、`agent` 这类**日期戳暂存目录**（codex 默认开聊处）；其余 **112 条**是具名真项目（蚊子 / gamework / crazytown / kt-erp / workspace …，**workspace 用户已确认是真项目**）。
- **用户决定（拍板）**：**按路径区分**——`/Users/yoyi/Documents/Codex/` 底下 = 无项目；其余各自成项目。**不按 git**（会错并非 git 真项目）；**不并 workspace**（真项目）。
- **规则（执行线落地照此）**：cwd 判为「无项目」当且仅当 `cwd == /Users/yoyi/Documents/Codex` **或** `cwd` 以 `/Users/yoyi/Documents/Codex/` 开头。命中 → `project_root = None` → 落进 `group_by_project` **已有的 None 桶**（:336），前端渲成一个「直接聊天 / 无项目」列表；其余 cwd 照常各自成项目。
- **实现注**：这是**用户特定路径**、且用户说"**先**按这个区分"——做成一个**命名常量 / 前缀列表**（如 `NO_PROJECT_PATH_PREFIXES`），别内联散写，方便日后加路径或换更通用信号。home(1)/tmp-probe(1) 那两条用户没点名，本规则**不含**。
- **落点**：后端 `codex_db.rs:262` 的 `project_root` 推导处加「无项目」判定；前端核 `ProjectsView` / 会话列表对 `project_root = None` 的渲染（是否已有 None 桶 UI，没有就加一个「直接聊天」分组）。

### B. 两处侧栏再收窄（CSS）

- app 左导航 `--sidebar-w`：196 → **176px**。
- 对话页会话栏 `.agent-session-shell` 列宽：256 → **224px**。
- 收窄后 §2 的标题截断要仍生效（栏更窄、更易溢出，重点复验 `.sc-title` 链）。

### 验证（本节）

- **后端**：加「无项目」判定后重跑 codex_db 测试 + 加断言（`/Users/yoyi/Documents/Codex/` 下的 thread → `project_root` None；具名项目目录（含 workspace）→ 各自 `project_root`）。对真实库核：无项目桶 = **24** 条（Documents/Codex 下），其余 **112** 条成项目。
- typecheck + offline 绿；**真机**：syn 出现「直接聊天」统一列表、真项目仍各自分组；两栏更窄且标题不溢出。
- 扫 diff：后端仅 `codex_db.rs`；没碰发送回路。
