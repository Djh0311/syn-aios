# F2C01 已归档 leaf 覆盖表述更正

日期：2026-08-19
状态：更正报告。不改写 `docs/harness/done/2026-08/F2C01-shell-core-bridge-v1.md` 的历史结论正文。

来源：stage-16 返修 kickoff 第 5 步；Harness 既有惯例是另写更正，不悄悄改已归档 leaf。

## 被更正的历史句子

已归档 F2C01 leaf 第 70 行写：

> cfg(test) 可达 case：5 个精确方法的 ready/unavailable、Jiaoban fixed-host directory/detail 与 opaque selector、operation receipt/同键 authoritative-audit replay/分歧冲突/执行自报拒绝，以及 11 个稳定桥错误、Stop、显式路径、external refs receipt-only、no-model registry/source 约束均由 `f2c01` 过滤覆盖。

该句把两件事写得过满：

1. **external-refs receipt-only** 被记成已由精确 case 覆盖。首轮 fixture 的 `CF-F2-POS-008` 虽然把 `external_refs` 挂在 `role_session.secretary_status` 上，但定向测试把回显断言写在 `operation_control.record_decision` 的成功路径上，并没有挂在 POS-008 自己的 case 上。
2. **fixture 覆盖** 被记成 `f2c01` 过滤已覆盖全部可达 case。实际上 `CF-F2-POS-010` 只查了字段齐全；`CF-F2-NEG-015` / `016` 没有合同正文文本断言；`CF-F2-NEG-017` 没有可测形式。只查 required keys 的检查被算进了覆盖。

## 现口径

以上过满表述不作为首轮已通过事实，也不作为本返修的完成证明。精确覆盖以 `F2C01R01` 的 fixture `case_class` + `precise_assertion` 和
`docs/contracts/fixtures/f2-bridge-001/coverage-audit.cjs` 机械统计为准。
已归档 F2C01 第 70 行原文保留为历史记录，本文件是对它的更正。
