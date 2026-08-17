# M1I01R02 project_index 已建立标记纠正报告

日期：2026-08-17

任务包：`M1I01R02`

阻塞 candidate：`253a03ec4eeae13e2a153cdb4503e325eb70c12c`

本报告记录独立复核对 `253a03e` 的窄 P1，以及本纠正做了什么。它不是独立验收，也不把 M3O01 标成已解阻。

## 1. 拒绝原因

P1：已建立后删除整个 `m1/` 被误判为从未建立，允许静默重建。

## 2. 纠正

- 在 app-data 根、`m1/` 之外写入 `.m1-project-index.established`。
- 读打开与登记在 marker 仍在而 registry 丢失时返回 `m1_project_index_registry_missing`。
- 增加整目录丢失定向测试。

## 3. 证据范围

只证明离线 `cargo check --lib --offline` 与定向 `m1_project_index` 单测。不证明真实 App、provider、网络、发布或独立验收。
