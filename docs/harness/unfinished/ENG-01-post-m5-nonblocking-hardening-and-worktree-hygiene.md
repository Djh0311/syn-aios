# ENG-01 M5 后非阻塞测试加固与工作树卫生

阶段：后续具名工程维护；当前不属于 stage-14 closeout 产品施工。

状态：`UNFINISHED` / `NOT_CURRENT` / `NON_BLOCKING_FOR_M5_CLOSEOUT`。

来源：`M5R09-20260818-1836.verdict.md` 欠账 5–8、用户 2026-08-18 18:40 closeout 纪律、M6P00 PASS verdict `stage-15-m6p00-20260819-0342.verdict.md` 欠账 1、2、6、7、9、CP1 PASS verdict `stage-15-cp1-20260819-0521.verdict.md` 欠账 7–9，以及 CP2 PASS verdict `stage-15-cp2-20260819-0733.verdict.md` 欠账 1、3、4、7。以下均属断言收紧、死代码、warning 分类、legacy command cutover、合同索引或载体卫生，不满足“现在不修则普通产品对真实用户不可用”门；只记录，不进入当前 M6 产品叶施工。

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
10. 随后续 command cutover 迁移 C4 旧 preview/prepare 的 production path-derived `project_id` 比较，并裁决普通 `record_project_director_process_fact_decision` 当前仍走 `record_project_director_process_fact_decision_at` 的 ownerless `core(None)` 分支；其 `default_workflow_id` path 派生兜底只有在所有 caller 都显式提供 `workflow_id` 且 workflow-state migration 已为所有 nested owner 落章后才能移除。这不是 M6 产品叶，也不得借 ENG-01 放宽 M5 项目主管语义；M6D08 只验证该 ordinary 入口不能进入任何 M6 跨项目 query 输入面，不在 stage-15 实施旧 command cutover。
11. 为 M2–M6 增补合同设计独立机器索引/校验器，或明确扩展现有机制但不得改写 M1 冻结十合同 `manifest.v1.json` 的旧 hash/语义；能发现增补合同间字段、export 与依赖矛盾，不把“未登记”继续当作已自动校验。
12. 最终裁定 6 个未跟踪旧 `m6_*.rs` 原型的 owner/保留/归档/删除去处；在归属明确前继续满足“不在 index、不在产品分支、内容 hash 不变”，不得把它们升格为 M6 基线或作为 `m6_org_*` 实现输入。
13. 统一 warning 计数口径：CP1 独立重跑的 rustc 汇总为 897 warnings，而 `rg '^warning:'` 连同汇总行计 898；M6D02 新文件贡献 0。CP2 独立重跑在 M6D03 `60a8e19` 为 897 warnings、在 M6D04 `ec1ba99` 为 888 warnings，减少 9 条来自旧 wrapper 接上真实消费者。后续基线报告必须同时写工具汇总与文本计数并绑定精确 SHA，不能继续无条件沿用 883 或把相同基线冒充新回归。
