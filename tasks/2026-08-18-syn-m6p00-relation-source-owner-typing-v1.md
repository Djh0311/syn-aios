# M6P00-A relation source owner 类型化窄任务

日期：2026-08-18

执行记录与口径（用户 2026-08-19 改定）：本包由 Grok 完成首轮实现，Codex 独立复核并可在同一写域内返修；后续仍以 Grok 为首选、Codex 为保底，任一时刻只允许一个源码写者。

## 目标

只修 `memory_entity_relation_governance.rs` 中 relation / relation candidate 的 source owner 判别：停止用 `source_id` 是否碰巧等于 canonical/legacy project id 来猜 owner。建立文件内可判别的 source owner 类型边界；只有明确标记为 project owner 的 source 才接受 canonical/legacy 校验与迁移。明确 foreign project owner 必须在任何业务写前 fail-closed；合法 doc/tool/session source 即使有任意 `source_id` 也不得被误拒。

## 唯一允许改动

- `prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs`

不得改任何其他文件，不得提交，不得暂存，不得 reset/stash/clean。不要读取、引用或触碰任何未跟踪 `m6_*.rs`、`.bak` 或 `src-tauri/gen/schemas/linux-schema.json`。

## 实现约束

1. 在本文件内增加显式、可枚举的 owner 分类（例如 project owner 与 non-project source 的内部 enum/typed classifier）；不能再由“值恰好等于当前 canonical/legacy id”来决定它是不是 project owner。
2. 新生成的 project entity source 必须带明确且稳定的 project-owner 判别信息；workflow/session/doc/tool 等来源保持非 project owner。
3. 历史 project-owner source 的兼容识别必须基于显式的 legacy source 形状/上下文并写清注释、保留理由和失效条件，不能把任意 `project:*` 字符串当 owner。
4. validate 必须先于任何 rewrite/store mutation；显式 project owner 的 foreign id 返回固定 mismatch 语义并保持 sidecar 字节、revision、audit、entity/relation/candidate 数量全部不变。
5. 只有 project owner 的 legacy id 可迁成 canonical；non-project doc/tool/session 的 `source_id` 不重写。
6. 不改变 M1-M5 已接受的执行合同、Grant/receipt/audit/quarantine 语义，不改冻结合同。

## 必须补的定向反例

- relation 与 relation_candidate 中显式 foreign project owner：业务写前拒绝，零部分写；至少覆盖二者。
- mixed owner：一个合法 non-project doc/tool/session source + 一个 foreign project owner，同样零部分写。
- 合法 knowledge doc、tool、session（可用现有 source kind/metadata 表达）带不会等于当前项目的 `source_id`：允许读取/写入路径，不误拒，且 id 原样保留。
- legacy project owner 迁移后重启/再次读取仍解析为同一 canonical owner。

测试名统一以 `m6p00_` 开头，使用离线临时目录，不接真实资料/provider/账号/网络。

## 自检

仅运行：

```bash
cd prototypes/productized-desktop-shell/src-tauri
cargo test --lib --offline m6p00_ -- --test-threads=1
cargo test --lib --offline memory_entity_relation_ -- --test-threads=1
git diff --check -- prototypes/productized-desktop-shell/src-tauri/src/memory_entity_relation_governance.rs
```

交回：说明实际分类规则、legacy 失效条件、测试结果，以及精确改动文件；不要声称 Codex 已验收。
