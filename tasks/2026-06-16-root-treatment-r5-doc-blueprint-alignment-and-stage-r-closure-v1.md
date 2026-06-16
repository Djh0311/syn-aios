# Root Treatment / R5 Doc Blueprint Alignment And Stage R Closure

日期：2026-06-16

状态：独立复核 CLEAR，验证门通过，待用户拍板和提交。

## 拍板摘要

本包把 product-line 的文档口径对齐到外部蓝图正本和 R3 / M12 已确立实现，然后给 Stage R 收口提供判定输入。

代价：纯文档，不改代码、不跑 runner、不碰真实数据、不改外部蓝图源。

不做的后果：蓝图、记忆层、吸收建议和 Stage L deferred 口径继续漂移，Stage R 无法干净收口。

一句话判据：R5 收没收口，看四块是否都“对齐到蓝图 + 实现、查重无残漂、Stage L 纯文档项已并、复核 CLEAR”；是，则 Stage R 可进入用户拍板收口。

## 范围

做：

- 对齐蓝图 §17 / §22 / §26.4 的经验沉淀、成熟模式、成功运行模式口径到 M12 已实现路径。
- 识别两份吸收建议文档，并与 M1-M13 / C1-C6 查重。
- 确认产品蓝图正本路径已由 `AUTHORITY.md` 锚定，不迁移外部源。
- 将 Stage L 纯文档 / 口径项并入 R5 收口记录和入口文档，产品代码项继续 deferred。
- 写 Stage R 收口判定输入。

不做：

- 不启动 Stage L 产品代码、K3-B1 retry、K3-B2 或真实操作控制。
- 不改外部蓝图源。
- 不改 backlog，不解冻 backlog 功能。
- 不碰 R3 deferred 产品切换窗口。
- 不改代码、runner、真实数据目录或 `/Users/yoyi/.codex`。
- 不声称 Stage R 之外阶段完成。

## 读取范围

- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/local-ai-workbench-ui-blueprint-v1.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/blueprint-absorption-notes.md`
- `/Users/yoyi/Documents/Codex/2026-05-26/gan-xing-codexbridge-https-github-com/docs/architecture/xuanji-blueprint-absorption-notes.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/memory-layer-implementation-slice-v1.md`
- `docs/plans/middleware-version-stage-plan-v1.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- `AUTHORITY.md`
- `CURRENT.md`

## 写入范围

- `tasks/2026-06-16-root-treatment-r5-doc-blueprint-alignment-and-stage-r-closure-v1.md`
- `evidence/2026-06-16-root-treatment-r5-doc-blueprint-alignment-and-stage-r-closure-v1.md`
- `evidence/2026-06-16-root-treatment-r5-doc-blueprint-alignment-and-stage-r-closure-review-*.md`
- `docs/memory-layer-design-v1.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- `AUTHORITY.md`
- `CURRENT.md`

## 验收判据

- 蓝图 §17 / §22 / §26.4 的经验沉淀口径在 product-line 正本中统一为 M12 路径。
- 两份吸收建议文档身份明确，查重矩阵覆盖主要吸收项，并区分 accepted / partial / deferred / not_absorbed。
- `AUTHORITY.md` 保留两份蓝图唯一正本路径，并记录“已满足，不迁移外部源”。
- Stage L 纯文档 / 口径项已并入 R5 收口记录；L1-L6 产品代码 / 真实执行项仍为 `deferred_during_root_treatment`。
- 独立复核线逐条核对忠实性，结论 `STATUS: CLEAR` 且无 P0 / P1 overclaim。
- `node scripts/harness/workbench-shape-gate.js --mode check` 和 `git diff --check` 通过。

## 停止线

- 如发现外部蓝图与 product-line 实现存在不能由 M12 路径解释的实质冲突，停止并交用户裁决。
- 如发现吸收建议中有“看似已实现、实际没有 evidence”的项，必须标为 partial/deferred，不得粉饰。
- 如任何文档改动会要求启动 Stage L 产品代码、R3 产品切换、真实数据写入或 `.codex` 接触，立即停止。
