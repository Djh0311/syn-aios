# 后端 util 去重治理方案 v1（⚠️ 已作废，并入合并正本）

> **本文已并入 `docs/plans/2026-06-13-stage-r-remaining-execution-plan-v1.md` 的 §3 R-U，作废。后续以合并正本为准，本文仅留作历史。**


日期：2026-06-13
出自：咨询线（Claude），基于 2026-06-13 子 agent 代码重复扫描。
性质：**治理子项方案草案，待全局/项目主管线接入 + 用户确认后执行。**无行为变化重构，严格符合 `decisions/2026-06-01-architecture-module-split-guardrail-v1.md`。属 Root Treatment / Stage R 范畴（后端形状治理，与 R4 前端瘦身同类），非新功能、不违反冻结。

> **拍板摘要（你在批什么）**：agent 扫出后端有一堆重复的小工具函数——算 hash 的写了 23 遍、拼 sidecar 路径 12 遍、字符串规范化 12 遍等。本方案把它们各抽成一份公共 util，调用处改成调公共的。**纯搬函数、不改任何行为/数据/业务规则**，每包以 `cargo test --lib` 全绿为"行为没变"的铁证,风险极低。代价：会碰十几个 store 文件，和 R3 SQLite 迁移碰的是同一批文件，**时序要排好别撞**。不批就维持现状——重复留着，且 Codex 以后还会接着各写各的。

---

## 1. 背景

子 agent 2026-06-13 扫描 `prototypes/productized-desktop-shell/` 确认的后端重复（纯 util 复制粘贴，非业务）：

- `sha256_hex()` **23 处**、`short_hash()` **14 处**（实现完全相同）。
- `sidecar_path()` **12 处**（实现相同，仅错误消息里的名词不同）。
- `normalize()`（`trim().to_lowercase()`）**12 处** + 若干特化变体。
- `remove_file_if_exists()` / `fixture_dir()` **4–6 处**。

根因：Codex 逐文件开发，每个 store 需要 hash/path 就地写一个，无查重门拦截。前端相对干净（`format.ts` 已集中复用），仅 `DetailLine`(4)/`SummaryTile`(3) 两个小组件重复。

## 2. 目标

1. 后端重复的纯 util 抽到公共模块，调用处改调公共。
2. 前端 2 个重复组件抽到 `src/components/`。
3. 给 harness 加一道"查重门"，防再造（治本，否则抽完又长回来）。

完成后：同一 util 全项目一份。

## 3. 范围

**做（无行为变化、纯函数抽取）：**
- U1：`sha256_hex` / `short_hash` → `src-tauri/src/utils/hash.rs`。
- U2：`sidecar_path`（加 `store_name` 参数容纳各店差异）→ `utils/store_paths.rs`。
- U3：`remove_file_if_exists` / `fixture_dir` → `utils/fs_ops.rs`。
- U4：`normalize` 基础版 + 必要特化（`normalize_symbol` 等）→ `utils/normalization.rs`。
- U5（前端）：`DetailLine` / `SummaryTile` → `src/components/`，各处改 import。

**不做（守 guardrail + 尊重 agent 判断）：**
- store 模式（`load_store` / `empty_store` / `write_store_atomic` / `validate_store`）**不强行合并**——每个 store 数据结构、JSON、validate 业务规则不同，合并会碰状态机/JSON 结构，违反 guardrail。各 store 保持独立。
- `validate_*` 业务校验逻辑**不合并**（逻辑确实不同）。

## 4. 严格符合 module-split-guardrail（`decisions/2026-06-01`）

- **无行为变化**：抽出的 util 实现与原逐字相同；调用处行为、serde 字段、命令名、输出不变。
- **必须保持**：函数行为、测试语义、JSON workflow state 结构、状态机语义——全部不变。
- util 是无状态纯函数，比 guardrail 第一批（拆类型/命令包装）**更安全**。
- **停止条件**（同 guardrail）：若抽某 util 牵连改 store 状态机 / JSON / 业务规则 → 立即停，该 util 留原地不抽，记 deferred + 理由。

## 5. 分批（每包独立任务包 / 实现 / 验证 / 复核 / commit）

| 包 | 内容 | 消除重复 | 关键禁止 |
|---|---|---|---|
| U1 | hash util 抽取 + 调用点替换 | `sha256_hex` 23 + `short_hash` 14 | 不改 hash 算法 / 输出格式 |
| U2 | sidecar path util | `sidecar_path` 12 | 错误消息可统一，但路径计算行为不变 |
| U3 | 文件操作 util | `remove_file_if_exists` / `fixture_dir` | 不改文件操作语义、不碰原子写时机 |
| U4 | normalize util | `normalize` 12 + 变体 | 特化版各自语义不变，不合并语义不同的 |
| U5 | 前端 2 组件 | `DetailLine` / `SummaryTile` | 不改组件渲染、className、prop 名 |

每包：任务包写明"预计消除 N 处重复 + 涉及哪些文件" → 实现 → 验证 → 独立复核 CLEAR → commit → checkpoint → 停在复核点。

## 6. 配套：harness 查重门（防再造，治本）

抽完后给 shape gate / harness 加一条：**新增 Rust 函数若与 `utils/` 公共 util 同名或同实现 → 警告**（或文档规则：写 hash/path/normalize 前先查 `utils/`）。这是 backlog「成熟模式沉淀为可执行脚本」的一个实例——查重交给脚本，不靠 Codex 自觉。**没有这道门，抽完迟早又长回 23 个。**

## 7. 时序（关键，与 R3 协调）

- util 去重碰十几个 store 文件（`formal_memory_store` 等），R3 SQLite 迁移碰的是同一批。
- util 去重本身**不迁 SQLite**（只抽纯函数），合规；但**避免和 R3 同时改同一 store 文件**（merge 冲突 + 复核混乱）。
- 建议：① R3 Level B 未排期 → util 去重可先做（无行为变化、低风险，先清理地基）；② R3 Level B 临近 → util 去重让位或与之协调插空。
- **具体插在 R 的哪个位置、和 R3/R4 的先后，由主管线根据当前排期定。**

## 8. 验证（每包）

- Rust 包：`cargo test --lib`（全绿 = 行为没变的铁证）+ 相关聚焦测试 + shape gate + `git diff --check`。
- 前端包（U5）：`npm run typecheck` + `npm run test:offline-interaction` + `npm run build`。

## 9. 复核（按复核严格度分级）

util 去重是"机器能验"的纯机械重构——`cargo test --lib` 焊死行为不变。按复核分级判据：**同源复核足够**（脚本门是真关卡，复核脑独不独立次要）。不需跨模型。

## 10. 不接受为

- 改了任何业务逻辑 / 状态机 / serde 字段 / JSON 结构。
- 合并了 store 模式或 validate 业务规则。
- 迁移 SQLite / 碰 R3 真实执行路径 / 读 `~/.codex`。
- 前端大改（仅抽 2 组件）。
- harness 查重门未加就声明"去重完成"（没门=会复发）。

## 11. 待确认（主管线 + 用户）

1. 定位：作为 Stage R 的哪个子项（新分支如 R-U，还是并入 R4 之后）——主管线定编号。
2. 时序：与 R3 Level B 的先后（见 §7）——主管线按排期定。
3. harness 查重门的具体实现形态（gate 脚本 / 文档规则）——动工时定。
