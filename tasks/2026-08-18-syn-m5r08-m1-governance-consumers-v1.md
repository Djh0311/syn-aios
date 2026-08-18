# M5R08 窄包 1：memory / mature-pattern 生产入口消费 M1 canonical ProjectId

执行者：Grok `grok-4.6 --reasoning-effort high`。本包只改产品源码；不要改 Harness、合同、报告或本任务包，不要 commit。

目标：修正 M5R07 独立验收 verdict 欠账 1。六个已注册生产 Tauri command 在读取或业务写之前，经普通 `AppState` 已安装的 M1 read port 用 `project_root` 作为 exact alias 解析 canonical `project:<uuid>`；memory entity/relation 与 mature-pattern 后续输入、store 顶层身份及新建正式记忆都消费该 canonical id，不再写 path-derived stable id。

允许改且只能改：

- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`

开工锚点（working-tree bytes，2026-08-18）：

- `commands.rs` SHA-256 `d64257b1e9ef86180ea38d46660ab73c059e2631572991f083604c4753009fb6`；该文件开工前已有用户 WIP（相对 HEAD 59 insertions / 56 deletions，主要是 rustfmt 形状），必须逐字保全无关 hunk。禁止对整个 `commands.rs` 跑 rustfmt。
- `mature_pattern_governance.rs` SHA-256 `3690939f1a5dc28a93c558afc1c06751619920515cf483315988d97340cfcb91`，开工前相对 HEAD 干净。
- `memory_entity_relation_governance.rs` SHA-256 `fcae41e3c6f2813285f7b394a41421684d559efca347d1a3c6ce634a98a18529`，开工前相对 HEAD 干净。

必须实现：

1. 生产 commands：`preview_mature_patterns`、`record_mature_pattern_decision`、`preview_memory_entity_relation_candidates`、`record_memory_entity_alias_decision`、`record_memory_entity_merge_decision`、`record_memory_relation_candidate_decision` 均先从 `state.m1_project_index_read_port()` 取得 authority，再 `resolve_exact_alias(project_root)`；未安装、unknown alias 或错误必须在任何 store load/write 之前直接返回固定 M1 error code。renderer/request 的 `project_id` 不可信，生产路径必须覆盖或拒绝，不能让调用方选择 canonical id。
2. governance 层增加明确的 trusted canonical-project 入口/上下文，使生产 command 传入的 canonical id 流到 preview、candidate/source/scope、store 顶层 `project_id`，以及 mature-pattern 用户确认后新建 formal-memory 的 `project_id`。不得只改 command 表面。
3. 现有直接单测入口可保留为 `#[cfg(test)]` legacy helper，以免大范围改测试文件；非测试生产调用图中不得再由 `crate::project_id(project_root)` 为这六条路径签发/写入项目身份。
4. 兼容边界：若旧 store 顶层 `project_id` 等于当前 `crate::project_id(project_root)` 的历史 stable id，允许在持锁写事务中迁到 canonical id；若为其他非空 id，fail-closed。旧嵌套 source/scope 只可作为 legacy read carrier 保留，不重写历史对象；所有新对象/写入使用 canonical id。加直接测试证明：legacy 顶层 id 可受控迁移、任意错项目 id 被拒绝且零 revision/业务写。
5. 同一根的 canonical preview/write 可读已有数据而不丢记录；不要新建第二 store、第二 owner、自动导入或 path fallback。
6. 在这三个允许文件内补以 `m5r08_m1_` 开头的直接测试/静态守卫，至少覆盖：六 command 走 M1 port、调用方 project_id 不能覆盖、authority missing/unknown alias 零写、canonical store 写入、legacy 顶层受控迁移、foreign id 拒绝。

禁止：

- 改 types/schema/DTO、M1 authority 实现、M5 runtime、M6、worker_report、页面或合同。
- 改冻结 M1–M4 合同；自动登记 alias；从 path/slug/index locator 生成 canonical ProjectId。
- reset、stash、clean、commit、`git add -A`、格式化整个文件或吞入开工前 WIP。

执行并报告：

```bash
cd /home/synadmin/workspace/syn/prototypes/productized-desktop-shell/src-tauri
cargo test --lib m5r08_m1_ --offline -- --test-threads=1
cargo test --lib memory_entity_relation_ --offline -- --test-threads=1
cargo test --lib mature_pattern_ --offline -- --test-threads=1
cargo check --lib --offline
```

交活时列出：改动路径、关键设计、命令与退出码、未解决项；不要自评 stage/M5 完成。
