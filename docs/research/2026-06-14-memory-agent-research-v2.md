# 第二轮深挖报告：编排候选对决 + SQLite 落地

> 核实强度：12 条最吃重的可证伪论断，每条 3 票对抗式核实 → **10 确认 / 2 推翻**。所有结论按核实结果写，被推翻的不采信。

---

## Part 1 — agency-agents vs agency-swarm（编排轴）

### 一句话结论

**两个都不是你要的"调度器"，而且 agency-swarm 经核实确认无法驱动本地 Codex/Claude Code（硬绑 OpenAI Responses API）。** 正确的用法是**三层**，不是"角色源 + 调度器"两层配对：

| 层 | 谁来做 | 判定 |
|---|---|---|
| L1 角色内容（~232 个 persona） | **agency-agents** | 🟢 借 |
| L2 角色→本地 CLI 编译（写进 ~/.codex、~/.claude） | **agency-agents 的 convert.sh/lib.sh** | 🟢 借（代码可直接 vendor） |
| L3 调度器（决定谁何时跑、起 CLI 子进程、传上下文、存运行态） | **你自建**（两个仓库都没有） | agency-swarm 只贡献**设计模式** 🟡 学 |

### 头对头（已剔除被推翻的论断）

| 维度 | agency-agents | agency-swarm |
|---|---|---|
| 本质 | 静态 Markdown 角色库 + bash 安装器（**零运行时逻辑**，100% Shell） | OpenAI Agents SDK 之上的多 agent 框架（Python ~98%） |
| 角色定义 | persona.md，YAML frontmatter。**CI 硬性只要 `{name, description, color}`**；Identity/Mission/Critical-Rules 等 7 段是 WARN-only 约定 | 无角色库，"角色"=`Agent(name, description, instructions, tools)` 元组 |
| 派发 | **无程序化派发**：人跑 install.sh 复制文件 → 在宿主 CLI 里自然语言"激活" | 真有运行时派发：每条边自动生成 `SendMessage` 工具，recipient 是**枚举约束**的允许接收者，运行时由发送方 LLM 决定调谁 |
| 真调度? | **否**（1908 行 bash 里无 scheduler/daemon/queue/run-loop） | **否**（无条件分支/排序/并行/重试；顺序是 LLM 工具调用涌现出来的，非确定性） |
| 驱动本地 CLI? | ✅ 但仅作为**配置安装器**：写 `~/.codex/agents/<slug>.toml`（`convert_codex` 只出 name/description/developer_instructions）+ `~/.claude/agents/*.md`（**原样拷贝、不转换**）。它不 *启动* CLI，只填配置目录 | ❌ **不能**（核实 3-0）。它的 "Codex" 是 OpenAI 托管后端 `chatgpt.com/backend-api/codex`；唯一的 `subprocess.Popen` 起的是 Node openclaw 网关，再用 HTTP 当 OpenAI 模型访问——**从不 exec 本地 codex/claude 二进制** |
| 成熟度 | 社交声量极高（**112,842 star / 18,398 fork**）但工程薄（~339 commits 的 markdown + 3 个 bash）、**0 release/0 tag**（必须 pin commit SHA），2025-10-13 建库 | 框架级成熟（4,445 star、2,545 commits、SemVer 勤发，最新 v1.10.1 / 2026-06-11）但**版本锁死** `openai-agents==0.14.8` 精确 pin |
| License | MIT | MIT |

来源：[agency-agents](https://github.com/msitarzewski/agency-agents)（`scripts/lib.sh`、`scripts/lint-agents.sh:33`、`convert.sh:133-153`）· [agency-swarm](https://github.com/VRSEN/agency-swarm)（`src/agency_swarm/messages/codex_input.py:11`、`integrations/openclaw_model.py:55`、`tools/send_message.py:95`、`pyproject.toml`）

### 互补性真相

它们在**内容轴上互补**（一个是角色内容 + 编译器，一个是角色机制 + 静态权限图 + 委派工具 + 线程持久化）。但你设想的"**agency-agents persona + agency-swarm 调度器**"这个具体配对**不成立**，败在两条已确认的事实上：
1. **agency-swarm 不是调度器**——它是 LLM 在固定拓扑上的非确定性工具调用，没有确定性的节点排序/分支/并行/重试。"角色当工作流节点按职能调度并按序/按条件执行"恰恰是它**不提供**的。
2. **agency-swarm 绑死 OpenAI**——要让它接本地 CLI，得给每个 CLI 套一层 OpenAI-Responses 兼容 HTTP shim（OpenClaw 那套），还得被迫接受 LLM 驱动的非确定性委派。两个重适配器 + 一个你没要的范式。

### 可以拿走的具体东西

**从 agency-agents 直接 vendor（代码）**：
- `lib.sh` 的 `get_field / get_body / slugify / agent_slug / is_agent_file` → 当你"无依赖的角色数据模型"。`agent_slug()` 是文件名的**单一真相源**，保证 role→Codex 和 role→Claude 两个编译器不打架。
- `convert_codex()`（convert.sh ~L133-153）→ 你的 "role → `~/.codex/agents/<slug>.toml`" 编译器（⚠️ 转换有损：丢 color/emoji/vibe，body 全压进一个 `developer_instructions` 字符串；上线前对一下你装的 Codex 版本的 TOML schema）。
- `install_claude_code` 的直拷 .md → 你的 "role → `~/.claude/agents/*.md`" 步骤（无需转换）。
- "Agents Orchestrator" persona 的正文 → 当你调度器的**种子 prompt**（PM→architect→dev↔QA→reality-checker 流水线），但它**自己执行不了**。

**从 agency-swarm 借设计（不是代码/运行时）**：
- `>` 运算符 → `AgentFlow` 边列表（`agent_flow.py`）：用代码声明角色-权限图。
- `configure_agents()` 把图编译成"每个发送方一个工具、recipient 参数是允许目标的 ENUM"——约束每个角色能调用哪些节点。
- `load_threads_callback / save_threads_callback`（`hooks.py`）→ 你可恢复运行态的模板（agency-agents 什么都不持久化，这块全是你的）。

### 风险（务必记住）

- ⚠️ **两个都不调度**——确定性的"角色即节点"派发（含 order/branch/parallel/retry）是你的净新增工程量。把任一当调度器是头号失败模式。
- ⚠️ **agency-swarm 无法驱动本地 CLI**（已确认）——任何"它能原生跑本地 agent"的方案都是错的。
- ⚠️ **agency-agents 无版本**（0 release/0 tag）、README 自相矛盾（自称 232/16，但目录里能 glob 到非 agent 目录）；agent 集合会逐 commit 漂移 → **pin SHA，每次升级重新校验目录**。
- ⚠️ **schema 弱保证**——只有 `{name,description,color}` 是 CI 硬要求，7 段正文是 WARN-only，社区 persona 完整度参差 → 摄取时按防御式写，别假设每个角色都有 Workflow/Success-Metrics 段。
- ⚠️ **Token 重量**——好几个 persona 正文塞了大段代码（React/ArcPy），编译进 prompt 前先剥掉，否则每个角色 token 成本暴涨。

---

## Part 2 — SimpleMem & mem0 的 SQLite 落地（记忆轴）

### ⚠️ 先纠偏两处（我上一轮说得不够准）

1. **SimpleMem 的 SQLite 支持比我上轮讲的强。** 上轮我强调它"用 LanceDB"，给人"和 SQLite 无关"的印象。**实测纠正**：它的核心向量库确实是 LanceDB 写死（不可换，确认 3-0），**但 SimpleMem 自带两个一等公民的纯 SQLite 子包**——`cross/storage_sqlite.py`（`SQLiteStorage`，stdlib sqlite3，WAL + foreign_keys + 6 表会话时间线）和 `evolver/store.py`（`MemoryStore`，1798 行，`memories` 表把 embedding 以 `embedding_json TEXT` **内联**存、带 FTS5 全文索引 + 损坏恢复 + ALTER-TABLE 在线迁移）。**这恰恰是你这次迁移最贴的先例。**（上轮那条"它不算支持 SQLite"的论断，这轮被 1-2 推翻。）

2. **mem0 真正值得抄的不是核心库，是 OpenMemory 子项目。** mem0 核心库的 SQLite 只是 `history.db` 审计日志（向量在 Qdrant，外部）；但它的 **OpenMemory 子项目**（`openmemory/api/`）就是一个 **SQLite（SQLAlchemy）+ 状态枚举 + 身份外键**的关系库——**~80% 就是你要的目标形态**。

### 两库各自的 SQLite 结论

| | SimpleMem | mem0 |
|---|---|---|
| 核心存储 | 向量 = LanceDB **写死不可换**；但 evolver/cross 子包是**纯 sqlite3** | 核心库：facts/向量在 Qdrant（外部），SQLite 仅 `~/.mem0/history.db` 审计日志 |
| "整栈一个 SQLite 文件"? | evolver 已证明**可行**（content+scoping+status+embedding+FTS 全在一个 .db） | ❌ 核心库做不到（24 个向量后端无一是 sqlite-vec）；OpenMemory 子项目 ✅（`sqlite:///./openmemory.db`） |
| 候选→正式门? | ❌ 无（状态只有 active/superseded/archived；`candidate.py` 是**检索策略**的 decoy，不是记录门） | ❌ 核心库无（ADD/UPDATE/DELETE 即时提交）；**OpenMemory 有真状态机**（`MemoryState` + `memory_status_history`） |
| 身份/作用域 | 核心**无**（单平表，多租户靠换文件）；cross 子包有 tenant/project | 核心库塞在向量 payload metadata（**非 SQLite 真相源**，不对称）；OpenMemory 用 FK 列（users/apps）✅ |
| License | MIT | **Apache-2.0** |
| 成熟度 | 研究单体仓库，成熟度不均；evolver/cross 是较新子包；**无 pyproject** | 高成熟（PyPI `mem0ai` v2.0.6、>3400 行 main.py、有测试树、Alembic 迁移） |
| 判定 | 🟢 **借**（移植 evolver schema 当参考，别想换核心 LanceDB） | 🟡 **改**（抄 OpenMemory 的表/状态/身份骨架） |

来源：[SimpleMem](https://github.com/aiming-lab/SimpleMem)（`core/database/vector_store.py:10,46`、`cross/storage_sqlite.py:62-81`、`evolver/store.py`、`evolver/models.py:21`）· [mem0](https://github.com/mem0ai/mem0)（`mem0/memory/storage.py:2,11,14`、`configs/base.py:42`、`openmemory/api/app/database.py:10`、`openmemory/api/app/models.py:30-34,161`）

### 三句话设计结论

- **抄谁的 schema**：OpenMemory（唯一自带 per-record 状态机 + 状态转移审计表 + FK 身份作用域 + 复合索引）。
- **谁证明"纯 SQLite + 向量"可行**：SimpleMem/evolver（内联 `embedding_json` + FTS5）。
- **抄谁的字段**：evolver 的 `MemoryUnit` 最丰富（memory_type/importance/confidence/access_count/reinforcement_score/supersedes/expires_at），再加核心 `MemoryEntry` 的 coref 消解后 `lossless_restatement`；mem0 只借它的 **content-hash 去重**思路。

### ⭐ SQLite 表结构草案（这是本轮最值钱的交付物）

融合 OpenMemory 的状态+身份骨架 + evolver 的内联向量/FTS5/富字段。注释里标了每个设计的来源文件。

```sql
-- Workbench memory layer. 纯本地 SQLite (CPython stdlib sqlite3). 单文件.
-- 连接设置 (从 SimpleMem 两个库都验证过):
PRAGMA journal_mode = WAL;        -- cross/storage_sqlite.py:62
PRAGMA foreign_keys = ON;         -- cross/storage_sqlite.py:63
PRAGMA synchronous = NORMAL;      -- cross/storage_sqlite.py:64

-- ===== 身份 (OpenMemory users/apps -> 泛化为 user/agent) =====
CREATE TABLE IF NOT EXISTS users (
    user_id      TEXT PRIMARY KEY,        -- 稳定外部 id ("别当陌生人"的锚)
    display_name TEXT,
    created_at   TEXT NOT NULL,
    metadata_json TEXT DEFAULT '{}'
);
CREATE TABLE IF NOT EXISTS agents (       -- 哪个工作台 agent 写/拥有该记忆
    agent_id   TEXT PRIMARY KEY,
    name       TEXT,
    created_at TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (     -- dual-id 借自 cross
    session_id          TEXT PRIMARY KEY,        -- 内部 uuid
    external_session_id TEXT UNIQUE,             -- 客户端/run id, 可空
    user_id   TEXT REFERENCES users(user_id),
    agent_id  TEXT REFERENCES agents(agent_id),
    project   TEXT,                              -- 项目作用域 (cross)
    started_at TEXT NOT NULL, ended_at TEXT,
    status    TEXT NOT NULL DEFAULT 'active'
                 CHECK(status IN ('active','completed','failed')),  -- cross:81
    metadata_json TEXT DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_sessions_user    ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_project ON sessions(project);

-- ===== 核心记忆 (OpenMemory.Memory + evolver.MemoryUnit + core MemoryEntry) =====
CREATE TABLE IF NOT EXISTS memories (
    memory_id  TEXT PRIMARY KEY,                 -- uuid4
    -- 身份作用域: 提升为索引列 (OpenMemory 做法; mem0 核心塞 payload — 别学):
    user_id    TEXT REFERENCES users(user_id),
    agent_id   TEXT REFERENCES agents(agent_id),
    session_id TEXT REFERENCES sessions(session_id),   -- 来源 session
    tenant_id  TEXT NOT NULL DEFAULT 'default',         -- cross 多租户
    -- 内容 (core lossless_restatement = coref 消解+绝对时间戳的自包含事实):
    content      TEXT NOT NULL,
    summary      TEXT DEFAULT '',                       -- evolver
    content_hash TEXT,                                  -- mem0 md5(content) 去重/幂等
    memory_type  TEXT DEFAULT 'fact',                   -- evolver
    -- 候选->正式 生命周期 (新增 — 任何库都没有; 最近的是 OpenMemory 状态机):
    status TEXT NOT NULL DEFAULT 'candidate'
             CHECK(status IN ('candidate','formal','superseded','archived','rejected')),
    -- 出处/列表字段 *_json (cross+evolver 约定):
    keywords_json TEXT DEFAULT '[]', persons_json TEXT DEFAULT '[]',
    entities_json TEXT DEFAULT '[]', topics_json  TEXT DEFAULT '[]',
    location TEXT, event_timestamp TEXT,                -- 事实发生时间 (ISO-8601)
    -- 排序信号 (evolver):
    importance REAL DEFAULT 0.5, confidence REAL DEFAULT 0.7,
    access_count INTEGER DEFAULT 0, reinforcement_score REAL DEFAULT 0.0,
    -- 取代关系 (evolver+cross):
    supersedes_json TEXT DEFAULT '[]', superseded_by TEXT,
    -- 向量: 默认内联 (evolver) 或外部 ANN (cross vector_ref) — 二选一, 见风险:
    embedding_json TEXT DEFAULT '[]', vector_ref TEXT, embedding_model TEXT,
    -- 时间戳 + TTL:
    created_at TEXT NOT NULL, updated_at TEXT NOT NULL,
    last_accessed_at TEXT, formalized_at TEXT,          -- 候选->正式的时刻
    expires_at TEXT, tags_json TEXT DEFAULT '[]'
);
CREATE INDEX IF NOT EXISTS idx_mem_user_status   ON memories(user_id, status);
CREATE INDEX IF NOT EXISTS idx_mem_agent_status  ON memories(agent_id, status);
CREATE INDEX IF NOT EXISTS idx_mem_tenant_status ON memories(tenant_id, status);
CREATE INDEX IF NOT EXISTS idx_mem_status        ON memories(status);        -- 快速列候选
CREATE INDEX IF NOT EXISTS idx_mem_hash          ON memories(content_hash);   -- 去重
CREATE INDEX IF NOT EXISTS idx_mem_superseded    ON memories(superseded_by);

-- ===== 状态转移审计 (OpenMemory memory_status_history, 原样借) =====
CREATE TABLE IF NOT EXISTS memory_status_history (
    id TEXT PRIMARY KEY,
    memory_id TEXT NOT NULL REFERENCES memories(memory_id) ON DELETE CASCADE,
    old_status TEXT NOT NULL, new_status TEXT NOT NULL,
    changed_by TEXT,          -- user_id / 'system' / agent_id 谁确认的
    reason TEXT,              -- 确认时的可选人工备注
    changed_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_msh_memory ON memory_status_history(memory_id, new_status);

-- ===== 分类 M2M (OpenMemory categories) =====
CREATE TABLE IF NOT EXISTS categories (
    category_id TEXT PRIMARY KEY, name TEXT UNIQUE NOT NULL
);
CREATE TABLE IF NOT EXISTS memory_categories (
    memory_id   TEXT NOT NULL REFERENCES memories(memory_id) ON DELETE CASCADE,
    category_id TEXT NOT NULL REFERENCES categories(category_id) ON DELETE CASCADE,
    PRIMARY KEY (memory_id, category_id)
);

-- ===== 全文检索 (evolver FTS5, unicode61, 缺失时优雅降级) =====
CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
    memory_id, content, summary, entities_text, topics_text, tokenize='unicode61'
);
-- 用触发器或 app 侧 _index_fts() 保持同步 (evolver 是 app 侧做的)

-- ===== 可选: 原生向量 ANN (两个库都没有, 需自己加扩展, 别假设存在) =====
-- CREATE VIRTUAL TABLE memories_vec USING vec0(memory_id TEXT, embedding FLOAT[768]);

-- ===== schema 版本 (优先版本表, 比 evolver 的临时 ALTER 干净, 比 Alembic 轻) =====
CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
```

### 候选→正式确认门设计（建在上面 schema 上）

**没有任何库现成提供这个门**（已确认）——OpenMemory 的状态机是唯一真先例，按它建：

1. **一律以候选写入**：新抽取的记忆 `status='candidate'`（列默认值）。这是相对 mem0/SimpleMem 的**关键反转**——它俩都即时提交；agent 永远不静默生成正式记忆。同时写一行 `memory_status_history(old='', new='candidate', changed_by='agent:<id>')`。
2. **检索默认尊重门**：正常读 `WHERE status='formal'`（这就是 `idx_mem_user_status` 的用途）；单独的"待审队列"查 `WHERE status='candidate'`；可选允许 agent **只在同一 session 内**读自己的候选。
3. **晋升 = 确认门**：一次事务 `UPDATE … SET status='formal', formalized_at=? WHERE memory_id=? AND status='candidate'` + 审计行。`WHERE status='candidate'` 守卫使晋升**幂等**（rowcount 0 = 已晋升）；`changed_by` 记录是谁确认（人 vs 自动规则）。
4. **拒绝/过期**：人工拒 → `status='rejected'`（留痕不删）；加个清道夫按 TTL 把陈旧候选转 rejected。
5. **省钱点**（mem0 因急切 embedding 而错过的）：候选的 `embedding_json/vector_ref` 留 NULL，**只在晋升时**才算 embedding + 建 FTS 索引 → 候选写入很便宜，只有确认的记忆才付向量成本。
6. **取代 vs 门 正交**：`superseded/archived` 是从 `formal` 走到的终态，`candidate→formal→rejected` 是入口门，全塞进一个 CHECK 约束的 status 列 + `memory_status_history` 记录每条边 = 完整可查的状态机。

### 跨会话身份连续性设计（"别当陌生人"）

OpenMemory 的**关系式 FK 身份**是对的模型（SimpleMem 核心无身份，mem0 核心把身份埋在向量 payload 里、不对称）。要点：
- **稳定身份锚**：每个人一个持久 `user_id`（工作台账号/OS 用户/email 派生），与 session 无关。记忆带 `user_id` **索引列**（不埋 JSON）。
- **session 有作用域，记忆跨 session 共享**：`session_id` 只记**来源/出处**，检索不按它过滤。
- **反陌生人查询**（session 启动时 hydrate）：`SELECT content FROM memories WHERE user_id=? AND status='formal' ORDER BY importance DESC, last_accessed_at DESC LIMIT k` → 把用户的正式记忆注入开场上下文，不管是哪个历史 session 产生的。可顺手 bump `access_count/last_accessed_at`，让常用记忆逐渐靠前。
- **连续性只建在正式记忆上**：因为检索过滤 `status='formal'`，陌生人问候只由人工确认过的事实构成——**门和连续性互相加固**，候选在确认前永不跨 session 泄漏。

### 迁移路径：sidecar JSON → SQLite（低风险，基本 1:1）

0. **盘点 JSON**：枚举 sidecar 记录形态，每个 key 映到目标列，未知键收进 `metadata_json` 兜底。**现在就定 embedding 方案**（见风险 #1）：内联 `embedding_json`（evolver，最简，暴力搜）vs 外部 ANN + `vector_ref`（cross）。本地起步用内联即可。
1. **建 schema + PRAGMA**：开新 .db，设 WAL/foreign_keys/synchronous，跑上面 DDL，`schema_migrations(version=1)`。FTS5 放 try/except——缺了就置 `fts_available=False` 退回 LIKE 扫描（evolver 的 `_create_fts` 套路）。
2. **字段映射**（每条 JSON → 一行 `memories`）：text/fact→`content`；算 `md5(content)`→`content_hash`；user/agent/session INSERT-OR-IGNORE 进各自表；tags/keywords/persons/entities→`json.dumps`→`*_json`；时间戳→`event_timestamp`；importance/confidence 缺省 0.5/0.7；embedding→`embedding_json` 或推外部索引设 `vector_ref`；其余→`metadata_json`。**状态决策（唯一需要判断的）**：现有 sidecar 记忆通常已"可信"→ 导入为 `status='formal'`、`formalized_at=created_at`、审计行 `changed_by='migration'`；**cutover 之后新建的才从 `candidate` 起**。
3. **单事务批量插入**：整个导入包在 `BEGIN/COMMIT`（快几个量级且原子），用 `INSERT … ON CONFLICT(content_hash) DO NOTHING` 让重跑幂等。
4. **校验**：`COUNT(*)` 对得上、`*_json` 能 `json.loads` 回去、CHECK 没拒行、跑一遍某用户的 hydration 查询肉眼核对。
5. **双写/切换**：保留 JSON sidecar 写一个 release（shadow），读走 SQLite，比对；再翻成 SQLite-only，JSON **归档不删**（当回滚件）。
6. **后续 schema 变更**：用 `schema_migrations` 编号迁移器（比 evolver 的临时 ALTER 干净，比 OpenMemory 的 Alembic 轻）；evolver 的损坏恢复（坏 .db 改名 .corrupt 重建）值得抄。

### 记忆轴风险（按严重度）

1. ⚠️ **向量搜索是头号风险**：纯 SQLite 无原生 ANN。选项：(a) 内联 + Python 暴力余弦——本地 ~1万–5万条以内 OK，再多就退化；(b) 加载 `sqlite-vec/sqlite-vss` 扩展——两个库都不带，**macOS 默认 python sqlite3 常禁用 `enable_load_extension`**；(c) 外部 FAISS/Chroma + `vector_ref`——又引入第二个存储。**早决定，它决定 embedding 列怎么设。**
2. ⚠️ **FTS5 可能缺失**：是编译期选项，部分 python sqlite3 没有 → FTS 创建包 try/except + LIKE 兜底。
3. ⚠️ **SQLite 单写者**：多 agent 并发写，即便 WAL 也会序列化、可能 `database is locked` → 单条共享连接（`check_same_thread=False`）+ 写锁/队列 + `busy_timeout`，事务短，**别每线程开新连接**。
4. ⚠️ **ephemeral DB 陷阱**（来自 mem0）：`SQLiteManager` 默认 `db_path=':memory:'`，只因 config 传了路径才持久化（相关 open issue #4290）→ 若移植 mem0 风格代码，**无显式磁盘路径就硬失败或大声报警**。
5. ⚠️ **别认错 decoy**：SimpleMem evolver 的 `candidate.py/promotion.py` 提升的是**检索策略**不是记忆记录；mem0 核心的 history 表是**审计日志**不是暂存区。两个都是确认门的诱人假匹配——唯一真先例是 OpenMemory 状态机。
6. ⚠️ **若抄 mem0 核心会有 scoping 不对称**：它把 user_id/agent_id/run_id 放向量 payload，而 SQLite history 只有 actor_id/role → 必须把身份提升为索引列（上面 schema 已这么做）。
7. ⚠️ **SimpleMem 安全 issue**（2026-04 报的）：#50 MCP server 硬编码 JWT/加密密钥、#53 `VectorStore.structured_search` 注入。**若借它的查询构造代码（不只是 schema），务必审注入、全程参数化**。

---

## Part 3 — 给工作台的最终落地清单

**编排层**：
1. 摄取 agency-agents 的 ~232 persona 当角色目录（pin SHA），只信 `{name,description,color}`，7 段正文当 best-effort，`strategy/` 不是 agent（是 NEXUS 文档）。
2. Vendor `lib.sh` + `convert_codex` + `install_claude_code` 当你的 role→本地 CLI 双目标编译器（Codex TOML + Claude .md）。
3. **自建调度器**（job-function→节点、order/branch/parallel/retry、起 codex/claude 子进程、传上下文、存运行态），路由层抄 agency-swarm 的 `>` 边-DSL + enum 约束 + 持久化回调的**设计**。
4. agency-swarm 别进运行时（绑 OpenAI、非确定性委派）。

**记忆层**：
5. 落 SQLite 用上面的 DDL（OpenMemory 骨架 + evolver 富字段/内联向量/FTS5）。
6. 候选→正式门按 OpenMemory 状态机建（候选默认写入、检索过滤 formal、晋升=事务 UPDATE+审计、候选延迟 embedding）。
7. 身份用 FK 列 + session 启动 hydration 查询解决"别当陌生人"。
8. 向量方案早定（内联暴力 vs sqlite-vec vs 外部 ANN）；移植 SimpleMem 查询代码前审注入。

---

## 核实说明与置信度

- **2 条被推翻**：① "agency-agents 是 248 agents/17 divisions 且 strategy 是 agent 分区"被 0-3 否（真相 232/16，strategy/ 是文档无 frontmatter）；② "SimpleMem 基本不支持 SQLite"被 1-2 否（evolver/cross 子包是真 sqlite3）。
- **10 条确认（多数 3-0）**：含 agency-swarm 无法驱动本地 CLI、绑 openai-agents==0.14.8、mem0 history=SQLite/向量=Qdrant、OpenMemory SQLite+状态机、两库均无核心候选门。
- 所有数字经 GitHub API 实查（2026-06-14）；license 经 LICENSE 文件 + API 双验。
- 时效：agency-swarm v1.10.1（2026-06-11）、mem0 v2.0.6、SimpleMem 无 pyproject——都很新、会漂。

## 主要来源（代码级）

[msitarzewski/agency-agents](https://github.com/msitarzewski/agency-agents) · [VRSEN/agency-swarm](https://github.com/VRSEN/agency-swarm) · [aiming-lab/SimpleMem](https://github.com/aiming-lab/SimpleMem) · [mem0ai/mem0](https://github.com/mem0ai/mem0)（核心 + `openmemory/api/`）
