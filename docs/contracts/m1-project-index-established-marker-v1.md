# M1 project_index 已建立标记纠正合同 v1

状态：`ADDITIVE CORRECTION / DOES_NOT_REWRITE_FROZEN_TEXT`

日期：2026-08-17

适用任务包：`M1I01R02`

被拒绝/阻塞 candidate：`253a03ec4eeae13e2a153cdb4503e325eb70c12c` 的 P1

## 0. 为何纠正 253a03e

`253a03e` 把“已建立”判定绑在 `m1/` 目录是否存在。整目录删除后被当成从未建立，登记会静默重建空白 registry。已建立状态丢失必须 fail closed。

本文件不改冻结合同正文、hash 或 schema，不改 `253a03e` 的所有权边界。

## 1. 标记

`project_index` 在 app-data 根下、`m1/` 目录之外，持有持久 established marker：`.m1-project-index.established`。

第一次成功 persist 之前写入该标记。标记不是 Actor / Role / Scope / Identity，也不进入读端口对外形状。

## 2. Fail closed

registry 文件缺失且 established marker 存在时，读打开与登记都必须返回 `m1_project_index_registry_missing`，不得重建 `m1/` 或空白 registry。

从未建立过（无 marker、无 registry 目录）的普通产品仍保持读端口未安装。

## 3. 非目标

与 `m1-project-index-base-correction-v1.md` 第 5 节相同。不声称 M3O01 已解阻。
