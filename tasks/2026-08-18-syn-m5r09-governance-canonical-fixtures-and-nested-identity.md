# M5R09 Grok 产品任务包：governance canonical fixture 与 nested legacy identity

## 目标

只关闭 M5R09 做完标准第 2、3 条：

1. `memory_entity_relation_governance` 与 `mature_pattern_governance` 在 canonical authority 已解析后，不只迁移 store 顶层 `project_id`；由旧 `crate::project_id(project_root)` 写入的 nested owner 身份必须在写入事务内收敛为同一 canonical ProjectId，或在只读 preview 中按明确兼容边界解释为同一 owner。
2. nested foreign / canonical+foreign mixed owner 必须在业务写之前 fail-closed，且 sidecar 字节、revision、audit、entity/relation/pattern/formal-memory 等业务效果零变化。
3. 六条 governance 路径的测试必须经过现有生产 canonical 函数和受信 authority fixture；不得再由 `#[cfg(test)]` wrapper 自行调用 `crate::project_id(project_root)` 发行身份。

本包不改 M1 authority、普通产品 command graph、M5 执行链、类型结构、store 模块或冻结合同。

## 唯一允许修改的产品文件

- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs`

不得修改其他文件。不要做任何 git 操作。不要覆盖、暂存或格式化仓库内既有未归属 WIP。

## 必须保留的生产边界

- 六个生产 canonical 入口保持为唯一业务实现：
  - `preview_candidates_for_canonical_project`
  - `record_alias_decision_for_canonical_project`
  - `record_merge_decision_for_canonical_project`
  - `record_relation_decision_for_canonical_project`
  - `preview_mature_patterns_for_canonical_project`
  - `record_mature_pattern_decision_for_canonical_project`
- caller 的 `project_id` 仍不可信；只能使用已传入的 `TrustedCanonicalProject.project_id`。
- 允许为了读/迁移历史数据精确计算 `legacy = crate::project_id(&trusted.project_root)`；禁止把该值当新身份发行、写进新行或作为 canonical fixture。
- legacy 兼容必须是精确等值边界；不得把任意 `project:*`、路径、alias 或 caller id 当 canonical。
- preview 保持只读，不得为迁移而写 sidecar。
- 写路径必须先完整验证 owner 集合，再做 nested rewrite 和本次业务决定；不得边扫描边留下部分业务变更。
- 不新增依赖，不改 schema/type，不自动导入，不接真实项目资料。

## nested identity 的最低覆盖

按这两个 store 当前类型与生产写法审计所有确实承载 owner ProjectId、或由旧 path-derived ProjectId 直接/派生写成的 nested 字段。最低必须覆盖：

- mature pattern store：candidate `scope.project_id`、candidate/report `member_refs[*].project_id`、report `project_ids`，以及这些持久化结构中直接保存 owner ProjectId 的 source-ref id（若当前生产结构确有该语义）。
- entity/relation store：Project entity/candidate 的 `source_id` 与 source refs/aliases 中直接保存的 owner ProjectId，以及由该 owner ProjectId 形成且被 relation/candidate 引用的 project entity key/id；迁移时必须保持引用闭合，不能只改 source id 留下悬空 subject/object/entity id。

不要对普通 evidence/source id 做宽泛字符串替换。只迁移精确 legacy owner 值及其可机械重算的派生引用；遇到无法证明属于当前 canonical/legacy owner 的 owner-bearing值要 fail-closed。

## 测试 authority fixture

- 将六条 test-only path-derived wrapper 改造成或替换为单一受信 canonical fixture 适配层：fixture ProjectId 必须是固定、明显非 path-derived 的 canonical 值，并调用上述生产 canonical 函数。
- 由于 `lib_memory_entity_relation_tests.rs` 不在本包写域，允许保留它所调用的既有函数名，但这些函数本身不得再 path-derive；它们只能转交给受信 canonical fixture。也就是说，本包结束后不得存在“test wrapper 自行 `crate::project_id(input.project_root)` 再调用生产函数”的行为。
- `lib_memory_lint_mature_pattern_tests.rs` 中 mature pattern 测试改为显式使用受信 fixture/canonical 生产入口；不要靠字符串扫描替代行为测试。
- 直接行为反例必须证明 fixture canonical id 与 `crate::project_id(project_root)` 不同，而六条路径产出的/持久化的 owner 身份仍是 fixture canonical id。若有人把任一路径改回 path-derived，测试应因值不等而红。

## 必须新增或强化的直接反例

至少包含以下机械断言，测试名带 `m5r09_`：

1. entity/relation legacy nested store：写前含 legacy project entity/source/ref/引用，canonical 写后所有 owner 与引用闭合为 canonical，不残留 legacy；同一决定可继续解析。
2. entity/relation mixed/foreign nested owner：顶层即使是 legacy/canonical，nested foreign 时写返回精确稳定错误；sidecar bytes 完全不变，revision/audit/entity/relation 数量不变。
3. mature pattern legacy nested store：写前 candidate scope/member/report 为 legacy，canonical 写后全部 owner 为 canonical，不残留 legacy。
4. mature pattern mixed/foreign nested owner：写返回精确稳定错误；pattern sidecar bytes 完全不变，revision/audit/candidate/report 不变，并证明没有 formal-memory 新写。
5. 六条 canonical fixture 路径：行为上证明每条使用 fixture canonical id，而不是 path-derived id；覆盖 preview + 对应 record 路径，不能只做源码字符串扫描。

可以更新 M5R08 旧测试中“nested 仍保留 legacy”的过时断言，使其符合 M5R09 收敛标准；不得改写 M5R08 scoped PASS 的历史含义。

## 交付验证

在 `prototypes/productized-desktop-shell/src-tauri` 执行并报告精确命令、退出码与实际测试计数：

```bash
cargo test --lib --offline m5r09_ -- --test-threads=1
cargo test --lib --offline memory_entity_relation -- --test-threads=1
cargo test --lib --offline mature_pattern -- --test-threads=1
cargo check --offline
cargo test --lib --offline m5_ -- --test-threads=1
```

在仓库根执行：

```bash
git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs
git diff --name-only
```

若验证暴露必须修改写域外文件，停止并如实报告，不要自行扩域。

## 禁止宣称

- 不得宣称真实项目、真实个人资料、真实 provider/账号/凭据、GUI/Tauri 窗口、跨项目 M6 或发布已验证。
- 不得关闭 M5R09、stage-14 或 M5，不得激活 M6、stage-15、syn-shell/F2/F3/F5。
