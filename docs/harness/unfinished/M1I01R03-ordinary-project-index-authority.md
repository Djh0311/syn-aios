# M1I01R03 普通 project_index 权威

阶段：stage-14 仍开。本文件只是独立 M1-owner 纠正投影，**不是** current leaf。唯一 current leaf 保持 `M5R07-project-ui-isolated-app-and-stage-candidate`。`authorization.json` 保持精确 closed 两字段。

目标：让普通 `AppState` 安装服务器-only M1 登记 / 读权威。显式精确别名签发 `project:<uuid>`，原子持久化，重建后同一解析。空 / 未安装走 `m1_project_index_unavailable`。不自动从 path / locator / scratch / M5 / 启动登记。不给 M1 角色身份或 M3 所有权。M3 还不消费本端口。不声称 M1 / M3 已解阻。

来源：独立验收发现普通登记仍是测试专用；用户要求在当前工作树做窄纠正。

产品：`docs/contracts/m1-project-index-ordinary-authority-v1.md`，`M1ProjectIndexAuthorityPort`，普通 `AppState` Result 边界。
