# ENG-01 M5 后非阻塞测试加固与工作树卫生

阶段：后续具名工程维护；当前不属于 stage-14 closeout 产品施工。

状态：`UNFINISHED` / `NOT_CURRENT` / `NON_BLOCKING_FOR_M5_CLOSEOUT`。

来源：`M5R09-20260818-1836.verdict.md` 欠账 5–8，以及用户 2026-08-18 18:40 closeout 纪律。以下均属断言收紧、死代码、warning 分类或载体卫生，不满足“现在不修则普通产品对真实用户不可用”门；只记录，不进入 M5C01 返修。

未来做完的标准：

1. duplicate-effect 测试的 `persisted_event_count` 与 `execution_readback_count` 查询失败必须显式报错；不得用 `unwrap_or(0)` 让表名漂移退化为 `0 == 0`。
2. 裁决 memory/mature governance 中 `#[allow(dead_code)] validate_preview_input`：删除，或把仍有效的字段校验并入 canonical 入口；不得重新接回 path-derived project id。
3. 对当前 `cargo check` warning 做来源分类：后续前置留桩、候选未接线、真实死代码、既有兼容面分别记账；不能用 blanket allow 掩盖空转能力。
4. 对历史 Git worktree 逐个确认目录、注册项、owner 和占用后再决定保留/移除/prune；不得在无人确认时批量 `git worktree prune`，不得删除其他会话仍使用的目录或引用。
5. 施工时只动新 current leaf 的精确写域，保全当前未归属 WIP；本文件本身不授权 reset、stash、clean、删除、产品源码或 Git ref 变更。
