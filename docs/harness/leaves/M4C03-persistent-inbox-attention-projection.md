# M4C03 持久 Inbox 与 Attention source projection

阶段：stage-06 阶段6 M4 秘书、Attention 与日常节奏
目标：建立 M4 自有版本化 schema/repository，把内部结构化低风险事件投影成 source-first Inbox/OpenLoop/Decision read state。
干完的标准：source owner/ref、水位、去重键、排序理由和 sensitivity 均持久；同源重放幂等、不同 owner 不合并；投影重建和跨重启通过；不创建 Todo、不反写 source owner。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/lib.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_domain.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_schema.rs
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs [新增]
- prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs [新增]
- prototypes/productized-desktop-shell/tests/ [新增]
- docs/harness/

## 步骤

1. 写 schema、dedupe、owner 分离和重建失败测试。
2. 实现 M4-owned repository/UoW 和首个内部 source adapter。
3. 实现 deterministic projection、watermark、quarantine 与 read DTO。
4. 跑聚焦/迁移/重启测试，独立审查后精确提交并归档。
