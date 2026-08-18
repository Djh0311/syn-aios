# ENG-01 M5 后非阻塞测试加固与工作树卫生

阶段：后续具名工程维护；当前不属于 stage-14 closeout 产品施工。

状态：`UNFINISHED` / `NOT_CURRENT` / `NON_BLOCKING_FOR_M5_CLOSEOUT`。

来源：`M5R09-20260818-1836.verdict.md` 欠账 5–8、用户 2026-08-18 18:40 closeout 纪律，以及 M6P00 PASS verdict `stage-15-m6p00-20260819-0342.verdict.md` 欠账 1、2、6、7、9。以下均属断言收紧、死代码、warning 分类、legacy command cutover 或载体卫生，不满足“现在不修则普通产品对真实用户不可用”门；只记录，不进入 M6D01 合同叶施工。

未来做完的标准：

1. duplicate-effect 测试的 `persisted_event_count` 与 `execution_readback_count` 查询失败必须显式报错；不得用 `unwrap_or(0)` 让表名漂移退化为 `0 == 0`。
2. 裁决 memory/mature governance 中 `#[allow(dead_code)] validate_preview_input`：删除，或把仍有效的字段校验并入 canonical 入口；不得重新接回 path-derived project id。
3. 对当前 `cargo check` warning 做来源分类：后续前置留桩、候选未接线、真实死代码、既有兼容面分别记账；不能用 blanket allow 掩盖空转能力。
4. 对历史 Git worktree 逐个确认目录、注册项、owner 和占用后再决定保留/移除/prune；不得在无人确认时批量 `git worktree prune`，不得删除其他会话仍使用的目录或引用。
5. 施工时只动新 current leaf 的精确写域，保全当前未归属 WIP；本文件本身不授权 reset、stash、clean、删除、产品源码或 Git ref 变更。
6. 复现并单独处理 `conversation_transport_` 既有 6 个失败；M6P00 候选与 clean HEAD 均为 22 passed / 6 failed / exit 101，失败集合相同，不得在 M6D01 或其他 M6 叶里顺手修。
7. 在测试迁移与旧 command cutover 后裁决 M6P00 留下的 16 个完全无调用者旧 wrapper；clean HEAD 883 warnings、M6P00 候选 897，净增 14 条 dead-code warning。删除前先确认没有 guarded legacy/test consumer，禁止 blanket allow。
8. 将受保护 WIP 保全改成可机检三条件：不在 index、不在任何产品分支、内容 hash 不变；识别 `refs/poracode/checkpoints/**` 会快照整棵工作树并可能在还原时覆盖未跟踪 WIP，补 manifest/预检模板但不擅自删 IDE refs。
9. 请总指导确认 `c60caa9` 在 stage-15 建立前吸收 21 个 tracked Rust WIP 是有意的 rustfmt 基线；确认后更新 M5R08 的 30 项静态 WIP manifest，未确认前不得把旧表继续当全部当前 status 事实。
10. 随后续 command cutover 迁移 C4 旧 preview/prepare 的 production path-derived `project_id` 比较；这不是 M6D01 合同叶，也不得借 ENG-01 放宽 M5 项目主管语义。
