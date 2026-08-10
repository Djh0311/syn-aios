# M4C10 全量回归、独立验收与阶段收口

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：在干净主线候选上执行 M4 退出矩阵、全量离线回归、文档同步、独立验收和 stage-06 收口。
干完的标准：M1/M3 冻结输入 exact；M4 聚焦和完整 Rust/前端/build 通过或如实记录；隔离证据可复核；旧读面回切与未进入边界清楚；current-state/master/M4/README/task queue/handoff/report 同步；工作树洁净；stage-06 关闭。

允许动：

- docs/current-state.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/plans/2026-08-01-syn-stage-4-secretary-attention-and-daily-rhythm-plan-v1.md
- docs/plans/README.md
- docs/task-queue.md
- docs/harness/reports/M4C10-mainline-integration-and-acceptance.md [新增]
- docs/harness/
- handoffs/2026-08-10-syn-m4-to-m5-m6-m7-handoff-v1.md [新增]
- prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs
- prototypes/productized-desktop-shell/scripts/run-m4-isolated-app-acceptance.mjs
- prototypes/productized-desktop-shell/scripts/run-r4-isolated-app-preflight.mjs
- refs/heads/main

范围校正：C10 完整 `cargo test --lib` 首跑暴露 C09 在共享 R4 launcher 中新增的
五处 source-string 静态契约碰撞。该文件只允许做保持运行时语义不变的 C09 局部
消歧，并复跑旧 R4/M3 与 M4 验收；不借此扩展隔离能力或修改 M1-M3 产品语义。

## 步骤

1. 核对 HEAD、Git 洁净、M1/M3 frozen inputs、M4C01-C09 提交与证据清单。
2. 运行合同、Rust 聚焦/完整、前端 typecheck/offline/build、迁移回切和隔离证据复核。
3. 由独立 Terra-ultra 任务按退出矩阵审查 P0/P1/P2、证据等级和未进入边界。
4. 回写当前状态、master/M4/README/task queue、验收报告和下游交接。
5. 精确提交收口内容，声明本叶完成，关闭 stage-06，并确认工作树洁净。
