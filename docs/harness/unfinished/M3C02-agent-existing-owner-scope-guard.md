# M3C02 Agent existing-thread owner / scope 后端守卫

阶段：stage-05 阶段5 M3 角色会话与显式交接
目标：在 provider spawn 前让 Agent existing-thread 与 supervisor 路同样校验 thread 的 project owner，并由服务器解析 role、scope、channel 与 profile；错误归属、跨项目和越权写默认拒绝。
干完的标准：伪造/跨项目 thread、错误 project root、客户端 role/profile 注入和未授权写在 spawn 前有确定性拒绝测试；合法同项目 existing-thread 保持兼容；不依赖前端隐藏。

允许动：

- prototypes/productized-desktop-shell/src-tauri/src/commands.rs
- prototypes/productized-desktop-shell/src-tauri/src/codex_db.rs
- prototypes/productized-desktop-shell/src-tauri/src/manual_relay/conversation_transport.rs

## 步骤

1. 冻结 Agent 与 supervisor existing 路的输入、owner lookup 和 spawn 调用顺序。
2. 先写跨项目、未知 thread、root 漂移和客户端伪造的失败测试。
3. 增加服务器 resolver 与 spawn 前 owner/scope 守卫，保持明确授权外部项目的独立范围入口。
4. 跑聚焦 Rust 测试、非测试构建和回归；核对没有 provider 调用。
5. 独立审查并精确提交；不在本叶接 schema、repository 或前端。
