# FND-006 验收记录 · 2026-08-03

验收日期: 2026-08-03（两轮：上午静态轮 + 傍晚真机轮）
验收环境: macOS (darwin arm64), tauri-cli 2.11.4(npm 全局), cargo 1.95.0, node 23.11.0
验收人: 总指导线（Claude Code）+ 用户（窗口内 console 操作）
被测代码: worktree `product-line-syn-fnd-002`, branch `syn-fnd-002-dev` @ `2d4a772`

## 隔离机制（对 runbook 的订正）

runbook 原文写的 `SYN_ISOLATED_PROFILE=1` **在代码库中不存在**（全仓 grep 零命中，属
上一轮文档虚高）。实际使用的真实隔离机制：

```bash
HOME=/tmp/fnd006-isolated-home \
RUSTUP_HOME=/Users/yoyi/.rustup \
CARGO_HOME=/Users/yoyi/.cargo \
tauri dev --config /tmp/fnd006-tauri-override.json
```

- 应用全部数据目录由 `$HOME` 派生，改 `HOME` 即真隔离；
- `RUSTUP_HOME`/`CARGO_HOME` 必须显式指回（rustup 按 `$HOME/.rustup` 找 toolchain）；
- override 配置内容 `{"app":{"withGlobalTauri":true}}`——仅验收用，repo 内
  `tauri.conf.json` 保持 `false` 未动；它只打开 console invoke 桥，不改任何守卫逻辑。

## 真机夹具

- scratch 项目目录：`/tmp/fnd006-project-a`、`/tmp/fnd006-project-b`（空目录）；
- 预置 store（隔离 HOME 内 `workflow-state.v0.json`）：两条 workflow（A/B 各自
  `project_id` 精确归属）、B 下一个 worker 节点、A 下一个 work item；
- 攻击载荷：`../../etc/passwd` 画布 ID、A→B 跨项目 workflow 查询、含假密钥
  （token/api_key/oauth_token）的 worker report。

## 运行时证据（真机 · 全部经 console invoke 打到生产命令）

| 场景 | 操作 | 结果 | 判定 |
|---|---|---|---|
| 正控·画布 | `canvas_load('fnd006-scratch')` | 返回 bootstrap 画布对象 | ✅ 合法放行 |
| 场景 2·路径逃逸 | `canvas_load('../../etc/passwd')` | `path_guard_rejected: object ID 含路径分隔符`；磁盘仅存在合法 `canvas/fnd006-scratch.json`，零越界写入 | ✅ **运行时拒绝** |
| 场景 1·跨项目 | A root 查 B workflow | `fnd004a_rejected: workflow '...' 不属于项目 'project:tmp-fnd006-project-a'，拒绝返回节点` | ✅ **运行时拒绝** |
| 正控·workflow | B root 查 B workflow | 返回 1 个预置节点（`role:subagent`） | ✅ 合法放行 |
| 场景 6·脱敏 | 录入含假密钥 report 后读磁盘 store | 持久化 audit event 中 `executed_what=[REDACTED: token content]`、`changed_what=[REDACTED: api_key content]`、`reason=[REDACTED: token content]` | ✅ **运行时脱敏** |
| 场景 8·身份 | 同上 audit event | `actor_ref="worker:fnd006-console-probe"`（服务端解析格式 `role:actor`）；004B 绑定字段（`authenticated_actor_id`/`authenticated_project_scope`/`report_hash`/`report_kind`/`attempt_id`）全部持久化 | ✅ **运行时解析+持久化** |
| 场景 7·重启持久 | 杀进程→同隔离 HOME 重启→重跑两条攻击 | 重启后 `path_guard_rejected` 与 `fnd004a_rejected` 原样拒绝；audit event 重启后仍在（count=1） | ✅ **运行时持久** |
| 隔离 | 全程真实 HOME 应用目录指纹（1098 项，路径+mtime+size） | 验收前后两次 diff 均 IDENTICAL；App 自报路径全在 `/tmp/fnd006-isolated-home/**` | ✅ 真实数据零接触 |

## 未达运行时、维持单测/集成级的场景（诚实边界）

| 场景 | 现状 | 原因 |
|---|---|---|
| 场景 3/4·伪造 report/grant | 集成级（worker_report 3 个 fail-closed 专测 + fnd006 scenario_3 真调生产入口） | `consume_worker_report_after_completion` 无 Tauri 命令入口，只随 director 链真实回程触发；全链运行时验证需 fake runner 夹具，工作量单独评估 |
| 场景 5·Station 3b 写入 | 单测级（identity_kernel channel ReadOnly 断言 + fnd006 scenario_5） | 需 supervisor 会话夹具；当前 Station 3b 防护在 identity kernel 的通道 side-effect 层 |

## 总结

**FND-006 = 运行时验收通过（8 场景中 5 个命令级场景 + 全部正控真机坐实，2 个场景维持集成级、1 个维持单测级并已明写原因）。**

真机证据链：隔离启动 → 存储绑定隔离 HOME → 两类攻击运行时拒绝（重启前后各一次）→
脱敏+身份+004B 字段持久化经磁盘文件坐实 → 真实 HOME 两次指纹终检零接触。

## 遗留问题

- 场景 3/4 的全链运行时验证依赖 fake runner 夹具，转入 M2 前评估；
- 场景 5 的 Station 3b 运行时验证需 supervisor 会话夹具；
- code-map advisory（`MAP_UPDATE_REQUIRED`）自首批起持续告警，非阻断，待处理。
