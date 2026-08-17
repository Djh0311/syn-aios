# M1I01R03 普通 project_index 权威报告

日期：2026-08-17

任务包：`M1I01R03`

本报告记录独立验收发现的窄缺口，以及本纠正包做了什么。它不是独立验收，也不把 M1 / M3 标成已解阻。

## 1. 发现

M1 已有原子 UUID registry。普通登记只挂在测试用 `M1ProjectIndexRegistrar`。普通 `AppState` 只打开读句柄；从未建立时槽位为 `None`，无法显式签发 canonical `M1ProjectId`。

## 2. 纠正

- 新增服务器-only `M1ProjectIndexAuthorityPort` / `M1ProjectIndexAuthorityHandle`。
- 普通 `AppState` 安装该权威；验收 / 遗留保持未安装。
- `AppState::m1_project_index_authority` 是 `Result` 槽位边界；未安装返回 `m1_project_index_unavailable`。
- 启动不写 registry。只有显式精确别名登记才 mint `project:<uuid>` 并原子持久化。
- 普通 `AppState` 重建后按同一别名 / id 解析到同一值。
- 不持有 Actor / RoleSession / permission / scope / identity。M3 不消费本端口。

## 3. 未声称

- 不声称 M1 / M3 已解阻。
- 不创建活动 RoleSession，不改 M5R07 current。
- 不证明真实 App、renderer、Tauri command、provider、网络、发布或独立验收。

## 4. 证据范围

只证明离线 scoped checks。不证明真实 App、provider、网络、发布或独立验收。

- `git diff --check`：clean
- `cargo test --lib --offline -- m1_project_index -- --test-threads=1`：17 passed；0 failed
- `cargo check --lib --offline`：exit 0（既有 warning，无本包新增 error）

实现 SHA 另作 evidence binding commit。
