# M6P00 canonical ProjectId 消费扩面与 relation owner 类型化前置

阶段：未来 stage-15 / M6 前置；当前不属于 stage-14 closeout 施工。

状态：`UNFINISHED` / `NOT_CURRENT` / `M6_NOT_ACTIVE` / `PRODUCT_WORK_REQUIRED_LATER`。

来源：`M5R09-20260818-1836.verdict.md` 欠账 2、4，以及用户 2026-08-18 18:40 closeout 纪律。两项均不满足“现在不修则普通产品对真实用户不可用”的提升门，因此不进入 M5C01 产品返修、不阻塞 stage-14 closeout；在 M6 域层施工前必须重新核验并建立具名 current leaf。

未来做完的标准：

1. 把 M1 canonical `ProjectId` 从现有六条 memory/mature governance 消费面扩展到 workflow、项目编排与执行链的正式读写入口；不得让 M6 跨项目查询同时消费 canonical id 与 path-derived id 两套命名空间。
2. 为 memory relation / relation candidate 的 `source_id` 建立可判别的 source kind / owner 类型边界；仅对明确属于 project owner 的 source 执行 canonical/legacy 校验，foreign project owner 在业务写前 fail-closed，合法 doc/tool/session source 不误拒。
3. 建立迁移、重启、跨项目拒绝、mixed owner 零部分写与 M6 ProjectSummary 查询反例；不得修改 M1–M5 冻结合同正文，解释变化需另建增补合同。
4. 施工前重新读取当时产品正本、stage-15、唯一 current leaf 和 authorization；本文件本身不授权 M6、产品源码、真实资料、外部业务写、提交或发布。

禁止从本文件推导：M6/stage-15 已激活、M5 closeout 未成立、当前可以改产品源码，或现有未跟踪 `m6_*.rs` 已被采纳。
