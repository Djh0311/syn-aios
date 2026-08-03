# FND-006 验收记录 · 2026-08-03

验收日期: 2026-08-03
验收环境: macOS (darwin arm64), tauri-cli 2.11.4(npm 全局), cargo 1.95.0, node 23.11.0
验收人: 总指导线（Claude Code）
被测代码: worktree `product-line-syn-fnd-002`, branch `syn-fnd-002-dev` @ `2d4a772`

## 隔离机制（对 runbook 的订正）

runbook 原文写的 `SYN_ISOLATED_PROFILE=1` **在代码库中不存在**（全仓 grep 零命中，属
上一轮文档虚高）。本次实际使用的真实隔离机制：

```bash
HOME=/tmp/fnd006-isolated-home \
RUSTUP_HOME=/Users/yoyi/.rustup \
CARGO_HOME=/Users/yoyi/.cargo \
tauri dev
```

- 应用全部数据目录（`~/Library/Application Support/CodexGovernanceWorkbench/**`）由
  `$HOME` 派生，改 `HOME` 即真隔离；
- `RUSTUP_HOME`/`CARGO_HOME` 必须显式指回真实路径——rustup 按 `$HOME/.rustup` 找
  toolchain，不指回会报 "no default toolchain"（本轮首次启动即因此失败）。

## 运行时证据（真机）

| 证据 | 内容 |
|---|---|
| 构建 | `tauri dev` dev profile 40.28s 完成，599 warnings（与当轮 check 基线一致） |
| 启动 | 进程 `target/debug/codex-governance-workbench` 存活；日志 `storage mode=json_only reason=storage_mode_config_missing:/tmp/fnd006-isolated-home/Library/Application Support/...` —— 存储层实际绑定到隔离 HOME |
| 隔离 | 运行期间隔离 HOME 内仅产生 `Library/Logs/local.codex.governance.workbench/CodexGovernanceWorkbench.log`（及首次失败启动留下的 .rustup/.npm 痕迹）；真实 HOME 应用目录 1098 项指纹（路径+mtime+size）运行前后 **逐字节 diff = IDENTICAL** |
| 重启 | 同一隔离 HOME 二次启动成功（0.49s 无重建，同一带守卫二进制），profile 持久、无修复无报错 |
| 相互影响 | 机器上另有 product-line worktree 的既有实例（PID 62929，端口 5174）全程未受影响；本验收实例 PID 树已清理 |

## 八场景逐项结论

| # | 场景 | 运行时证据 | 单测/集成证据 | 结论 |
|---|------|-----------|--------------|------|
| 1 | 跨项目访问拒绝 | **未跑**——隔离 profile 无项目夹具，且 `withGlobalTauri:false` 无控制台 invoke 入口 | identity_kernel 16 tests + FND-004A 3 负例 | 单测级 |
| 2 | 路径逃逸拒绝 | **未跑**——UI 无裸输 canvas id 的入口 | path_guard 32 tests + fnd006 scenario_2 | 单测级 |
| 3 | 伪造 report（无 grant） | **未跑**——consume 只随 director 链真实回程触发，需 fake runner 全链夹具 | worker_report 2 专测 + fnd006 scenario_3（真调生产入口，断言零文件副作用） | 集成级 |
| 4 | 伪造 grant（格式非法） | 同上 | 同上 + 中间态 attempt 拒绝专测 | 集成级 |
| 5 | Station 3b 写入拒绝 | **未跑**——需 supervisor 会话夹具 | identity_kernel channel ReadOnly 断言 + fnd006 scenario_5 | 单测级 |
| 6 | 审计 UI 脱敏 | **部分**——隔离 profile 尚无 report 事件可看；脱敏发生在落库边界（集成已证） | scrub_content 单测 + c4_c6 落库路径集成 | 集成级 |
| 7 | 重启后守卫仍生效 | **已跑（部分）**——同一隔离 profile 干净重启，同一带守卫二进制，存储层仍绑定隔离 HOME | path_guard 纯函数本就无状态 | 运行时（部分） |
| 8 | 身份解析验证 | **未跑**——无控制台入口 | identity_kernel 16 tests + fnd006 scenario_8 | 单测级 |

## 总结

**FND-006 运行时验收 = 部分完成**：真机拿到了「隔离 profile 可启动、存储层绑定隔离
HOME、真实数据零接触、干净重启」四项证据；但 8 场景中 6 个命令级场景的运行时验证
**未完成**，原因是双层缺口：① `withGlobalTauri:false`，webview 控制台没有 invoke 桥；
② 全新隔离 profile 里没有双项目/workflow/会话夹具，UI 也无可触发的入口。

补齐路径（下次开工直接做）：
1. dev 配置临时开 `withGlobalTauri`（仅验收用，验收完回退），用 devtools 控制台直接
   `invoke('canvas_load', {id: "../../etc/passwd"})` 等拿命令级运行时拒绝证据；
2. 造隔离夹具：两个 scratch 项目 + 一条 workflow + 一份含敏感内容的 report 事件；
3. 按本表重跑场景 1/2/5/6/8，场景 3/4 走 fake runner 全链。

## 遗留问题

- runbook（README.md）的 `SYN_ISOLATED_PROFILE=1` 为虚构机制，已在本记录订正，
  README 已同步改写为真实机制。
- 场景 3/4 的全链运行时验证依赖 fake runner 夹具，工作量单独评估。
