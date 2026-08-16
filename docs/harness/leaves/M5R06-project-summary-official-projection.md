# M5R06 ProjectSummary 正式投影

阶段：stage-14 M5 项目主管与执行闭环（事实重整与产品闭环）

目标：Secretary/M6 只能经受控端口读取最小、可重建、只读的 ProjectSummary；看不到原文，也不能反写项目。每次按 consumer RoleSession / scope / policy 判权。

来源收据：用户继续完成提示内剩余工作；M5R05 PASS（`a5d93e8`）。

产品：m5_project_summary.rs 正式 projector / query port

证据：docs/harness/reports/M5R06-project-summary-official-projection.md [新增]

载体：working-copy + 独立内容 commit（opening HEAD=a5d93e8）

允许动：

- docs/contracts/（仅新增 M5 ProjectSummary 补充合同）
- prototypes/productized-desktop-shell/src-tauri/src/m5_project_summary.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs（仅本包最小声明）
- tasks/2026-08-16-syn-m5r06-project-summary-official-projection-v1.md [新增]
- docs/harness/plan.md、docs/current-state.md、docs/harness/audit/2026-08.jsonl、docs/harness/stages/stage-14.md
- docs/harness/reports/M5R06-project-summary-official-projection.md [新增]
- docs/harness/leaves/M5R06-project-summary-official-projection.md
- docs/harness/done/2026-08/M5R06-project-summary-official-projection.md [退场时新增]

不许动：

- M1–M4 冻结合同；m6_*.rs；stage-12 / D0C04 / D0C05
- 让 consumer 直接读项目 store / 文件 root / 完整 snapshot
- 真实资料/provider/push/reset
