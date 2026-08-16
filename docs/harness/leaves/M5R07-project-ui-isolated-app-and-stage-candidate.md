# M5R07 项目 UI、隔离 App 与阶段候选

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：用冻结 DTO 接现有项目壳，不重写 execution kernel 或页面布局；在隔离 app-data / scratch 上证明完整闭环；形成只含 M5 投影的 candidate series 后保持 `AWAITING_INDEPENDENT_ACCEPTANCE`。不自行关闭 stage-14，不宣布 M5 完成。

来源收据：用户明确把提示内剩余工作做完；M5R06 PASS（`867fd20`）。

产品：m5_dto.rs、m5_product_commands.rs、m5_isolated_acceptance.rs

证据：docs/harness/reports/M5R07-project-ui-isolated-app-and-stage-candidate.md、docs/harness/reports/M5R07-isolated-acceptance-receipt.json

载体：candidate commit series tip（本叶提交后的 exact SHA）

允许动：

- docs/contracts/（仅新增 M5 UI/DTO 补充合同）
- prototypes/productized-desktop-shell/src-tauri/src/m5_dto.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m5_product_commands.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m5_isolated_acceptance.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包最小声明/command 接线）
- tasks/2026-08-16-syn-m5r07-project-ui-isolated-app-and-stage-candidate-v1.md [新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md
- docs/harness/reports/M5R07-*
- docs/harness/leaves/M5R07-project-ui-isolated-app-and-stage-candidate.md
- docs/harness/done/2026-08/M5R07-* [仅独立验收后 closeout 才归档]

不许动：

- 自行关闭 stage-14 或宣布 M5 完成
- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 真实资料/provider/push/reset；伪造窗口截图或 Hook receipt
