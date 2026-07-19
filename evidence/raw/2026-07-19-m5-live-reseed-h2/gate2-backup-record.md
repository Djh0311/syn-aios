# Gate 2 仓外备份记录（2026-07-20 00:30–00:33 +0800）

## 唯一新备份目录（解析后绝对路径，创建前确认不存在）

`/Users/yoyi/workbench-backups/workflow-state-backup-20260719-pre-reseed-003058/`

（命名沿用包模板 `20260719` 任务日期；本地时钟已过零点。07-14/07-16 两代旧备份未触碰。）

## snapshot/ 内容（`cp -Rp`，源三根的完整副本）

- `snapshot/workflow-state/` — workflow-state 全根（含 backups/ 子目录）
- `snapshot/runtime-artifacts/` — 外层 runtime-artifacts 全量（内含 storage-mode.v1.json）
- `snapshot/production-db/` — 移动前 DB 三件复制件

## manifest 与校验

- `manifest.json`（SHA-256 `227330d99247f4515a8f701dff932659de0557a53479c809107ab8def0c8846d`）
- `manifest.txt`（SHA-256 `b042c41fd05ed50480f9839398c2dc4c503b57a949e21caa294b91e7808227cd`）
- 逐文件：相对路径 / 类型 / size / mtime / SHA-256。
- 源↔副本核对：**467 文件、405,017,547 字节，逐文件 SHA-256 全 PASS**，无缺失、无 size/hash 不等。
- `ROLLBACK-NOTE.md`：回滚源与「尚未执行 apply」已写明。

## stale-db 归档（移动原件，逐个解析绝对路径、目标预先确认不存在）

- 生产 DB 三件移入 `stale-db/` 后，hash 与 Gate 0 冻结逐一相等：
  - sqlite `5cbf8e2c…ec69a` ✓ / wal（空）`e3b0c442…b855` ✓ / shm `fd4c9fda…89eb` ✓
- `snapshot/production-db/` 复制件 hash 同样相等 ✓
- 移动后 `production-db/` 目录为空：**生产 DB 三件均不存在**（Gate 3/4 前置证明）。

**Gate 2 绿，进入 Gate 3。**
