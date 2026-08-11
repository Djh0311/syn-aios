# M4R07 普通产品隔离验收、全量回归与收口

阶段：stage-07 阶段7 M4 独立修正与再验收
目标：只做普通产品 composition 集成、故障恢复、可携带证据、新鲜全量回归、独立审查和 M4 修正文档收口。
干完的标准：全新隔离 root 无 repository 预灌；普通 AppState/registry/dispatcher/scheduler/resolver/transport/readers 覆盖全部对象与五项 P1；SIGKILL/restart/duplicate/failure/rollback 无重复 effect；可携带 UI 证据带 hash；全量回归通过；stage-07 关闭后等待总线独立复核。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/m4_acceptance.rs
- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src/App.tsx
- prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx
- prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/src/lib/types/
- prototypes/productized-desktop-shell/tests/
- prototypes/productized-desktop-shell/scripts/
- prototypes/productized-desktop-shell/package.json
- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md
- docs/plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/
- handoffs/2026-08-10-syn-m4-to-m5-m6-m7-handoff-v1.md
- prototypes/productized-desktop-shell/dist/
- prototypes/productized-desktop-shell/src-tauri/target/

## 步骤

1. 用全新隔离 root 和普通产品 composition 覆盖全部 M4 对象、五项 P1 和旁路禁令。
2. 覆盖 transaction interruption、SIGKILL/restart、duplicate event/tick/message、fake failure 与 guarded rollback。
3. 保存脱敏可携带 UI/receipt/hash 证据，完成 Git/Harness、代码测试、文档下游三条独立复核。
4. 在新专用 TMPDIR 跑完整 Rust，并跑 typecheck、离线入口、production build、launcher syntax、rustfmt 与冻结 hash exact。
5. 只按实际证据同步 current-state/计划/交接，精确提交、归档叶与 stage，停止等待总线复核。
