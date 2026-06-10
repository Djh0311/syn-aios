# Root Treatment R-Preflight Authority Sync And Git Baseline v1

日期：2026-06-10

## 结论

R-Preflight 已完成本轮核心前置：

- 已复核当前权威入口与治本方案 / 交接文档的口径冲突。
- 已新增 Stage L 与治理阶段 R 的冻结关系 decision。
- 已同步当前权威入口，把当前执行方向切到 Root Treatment / Stage R / R-Preflight。
- 已初始化 `product-line` git 仓库并建立治理前 baseline commit。

本轮不接受为 R0 / R1 已完成，不接受为 Stage L 已完成或取消，不接受为 K3-B1 / K3-B2 取消或恢复成功。

## 关键产物

- `decisions/2026-06-10-stage-l-root-treatment-freeze-relationship-v1.md`
- `docs/plans/2026-06-10-root-treatment-official-development-plan-v1.md`
- `CURRENT.md`
- `tasks/README.md`
- `AUTHORITY.md`
- `STAGE_PLAN.md`
- `README.md`
- `docs/plans/2026-06-10-stage-l-post-k-deferred-closure-and-daily-use-hardening-plan-v1.md`
- `.gitignore`

## 权威口径

当前执行方向：

```text
Root Treatment / Stage R / R-Preflight
```

Stage L 状态：

```text
Stage L / L1-L6 在治理冻结期内暂挂为 deferred_during_root_treatment。
这不等于 Stage L 完成或取消，也不等于取消 K3-B1 / K3-B2。
治理收口后再回到 Stage L / Stage K。
```

治理期边界：

```text
不授权真实 codex exec / codex exec resume。
不发送 prompt。
不读写 /Users/yoyi/.codex。
不启动 K3-B1 retry 或 K3-B2。
不解冻 backlog 功能。
```

## 版本控制基线

初始化位置：

```text
/Users/yoyi/workspace/product-line
```

Baseline commit：

```text
ed01c6f281e3fd7a38548da948046e8366cc368d
```

Baseline message：

```text
chore: establish root treatment governance baseline
```

忽略规则已排除：

- `node_modules/`
- `dist/`
- Rust / Tauri `target/`
- `prototypes/tauri-capability-probe/.cargo-home/`
- `prototypes/tauri-capability-probe/.cargo-target/`
- `prototypes/tauri-capability-probe/.tauri-cli/`
- 顶层 `.cargo-home/`
- `.harness/`
- `.env` / `.env.*`
- 系统文件和日志

## 验证记录

权威入口旧口径扫描：

```text
rg -n "L1 任务包已创建并待执行|状态为待执行|先用 Stage L|Stage L 收口后再回到 Stage K|Stage L 收口后再回 Stage K|待用户确认本文后|用户确认前|不改当前入口事实" ...
```

结果：

```text
无命中
```

新口径扫描确认 `Root Treatment / Stage R / R-Preflight / deferred_during_root_treatment` 已出现在当前入口、正式计划、Stage L 计划和 decision 中。

Git 检查：

```text
git rev-parse --show-toplevel
```

R-Preflight 前结果：

```text
fatal: not a git repository (or any of the parent directories): .git
```

初始化后：

```text
git rev-parse HEAD
ed01c6f281e3fd7a38548da948046e8366cc368d
```

```text
git status --short
```

结果：

```text
无输出，工作树干净
```

提交前运行过：

```text
git diff --cached --check
```

结果为失败，原因是历史文件存在大量既有 trailing whitespace / EOF blank line 债务。该检查发生在首次 baseline 提交前，属于治理前事实，不在本轮批量修复，后续由 shape gate / 治理任务逐步处理。

## 边界确认

- 未改产品运行时代码。
- 未执行真实 `codex exec` / `codex exec resume`。
- 未发送 prompt。
- 未读写 `/Users/yoyi/.codex`。
- 未读取 auth/token/`.env`/keychain/OAuth/provider credential/full transcript。
- 未启动 Tauri / Browser / Chrome / Vite / 截图工具。
- 未执行 R0 / R1。

## 下一步

- 写 R0 任务包：shape gate、任务包模板、治理任务包类型、解冻后 `1:3` 治理配额。
- 写 R1 任务包：workflow state StoreLock 和 backup retention。
- R0 / R1 可并行推进，但每次完成后必须形成独立 git commit 并记录 hash。
