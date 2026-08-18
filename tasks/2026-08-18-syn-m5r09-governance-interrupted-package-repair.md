# M5R09 Grok 返修包：收敛 governance 中断现场

## 现场与目标

上一包在写入过程中被主管中止；它的变更只属于本包，不是既有未归属 WIP。保留正确部分，完成以下三个窄目标：

1. 修复当前唯一失败测试 `memory_entity_relation_governance::m5r09_tests::m5r09_entity_relation_legacy_nested_store_converges_and_stays_resolvable`：持久化后的 relation 与 relation-candidate `source_refs[*].source_id` 仍保留精确 legacy owner ProjectId。只对能精确证明等于当前 `legacy = crate::project_id(&trusted.project_root)` 的 owner-bearing project source id 做 validate/rewrite；foreign 或 canonical+foreign mixed 必须在业务写前 fail-closed，不能宽泛替换普通 evidence/source id。
2. 完成六条生产 canonical 路径的固定非 path-derived fixture 覆盖。现有两个 governance 文件里的 test-only wrapper 可以保留兼容函数名，但只能把固定 fixture 传给生产 `*_for_canonical_project` 函数，不能自行发 path-derived 身份。
3. 在 `lib_memory_lint_mature_pattern_tests.rs` 把与 mature governance 相关的既有测试改为显式构造受信 canonical fixture并调用生产 canonical 入口；只做为适配现有测试所需的最小修改。

不要新增大段重复 fixture/test；当前两个 governance 文件已有 `m5r09_tests`，优先最小修正使它们真实通过。不要更改本包范围外文件。

## 唯一允许修改的产品文件

- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs`

不要做 git 操作，不要运行全仓格式化，不要触碰或覆盖其它未归属 WIP。

## 已知原始失败

从 `prototypes/productized-desktop-shell/src-tauri`：

```text
cargo test --lib --offline m5r09_ -- --test-threads=1
test result: FAILED. 20 passed; 1 failed; 2060 filtered out
failure: legacy owner project:tmp-m5r09-entity-legacy-nested remained in relation/relation-candidate source_refs
```

返修后必须证明：legacy nested 收敛、foreign/mixed 零写、六条 fixture canonical id 与 `crate::project_id(project_root)` 不同且路径仍走生产 canonical 函数。

## 验证

在 `prototypes/productized-desktop-shell/src-tauri` 运行并报告精确退出码与测试计数：

```bash
cargo test --lib --offline m5r09_ -- --test-threads=1
cargo test --lib --offline memory_entity_relation -- --test-threads=1
cargo test --lib --offline mature_pattern -- --test-threads=1
cargo check --offline
cargo test --lib --offline m5_ -- --test-threads=1
```

在仓库根运行：

```bash
git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs prototypes/productized-desktop-shell/src-tauri/src/mature_pattern_governance.rs prototypes/productized-desktop-shell/src-tauri/src/lib_memory_lint_mature_pattern_tests.rs
git diff --name-only
```

若必须改范围外文件，停止并报告，不得扩域。不要宣称真实项目/真实资料/GUI/Tauri/发布已验证，也不要关闭 M5R09、stage-14 或 M5。
