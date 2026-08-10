# M4C07 DailyReport、scheduler 与空事件零模型

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：实现本地时区 daily window、DailyBrief/Report、catch-up、版本纠正、失败恢复、预算与空事件零模型机械证明。
干完的标准：稳定 daily_window_id；同窗重跑幂等；错过窗口受限补跑；夏令时/时区变更显式版本化；report 可重建可回源；模型不可用仍生成；空事件 invocation count 精确为 0。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_service.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_scheduler.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs
- prototypes/productized-desktop-shell/src/lib/secretaryReadModel.ts
- prototypes/productized-desktop-shell/src/lib/tauri.ts
- prototypes/productized-desktop-shell/tests/ [新增]
- docs/harness/

## 步骤

1. 写 timezone/window/catch-up/DST/幂等/失败恢复和 invocation spy 测试。
2. 实现 scheduler checkpoint、TimerFired/DailyWindowClosed/Versioned 事件与 report projector。
3. 先确定性聚合，再接显式用户解释的可选模型增强与预算。
4. 跑属性/故障注入/重启/前端测试，独立审查后精确提交并归档。
