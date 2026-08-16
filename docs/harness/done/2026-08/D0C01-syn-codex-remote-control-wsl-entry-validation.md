# D0C01 Remote Control 备选退场决策

阶段：stage-10 阶段10 Syn 5600X/WSL 原方案 C0 只读配置门
目标：如实记录用户未采用后来提出的 Codex Remote Control 备选，并把后续入口恢复为迁移计划原定的 Windows Tailscale + WSL NAT + 受限转发/SSH 路线。
干完的标准：确认未执行 Codex Remote Control 配对、未修改 5600X，备选不再是后续前置条件，并切换到 D0C02 原方案只读配置门。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-10.md
- docs/harness/leaves/D0C01-syn-codex-remote-control-wsl-entry-validation.md
- docs/harness/leaves/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/unfinished/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/unfinished/D0C01-syn-codex-remote-control-wsl-entry-validation.md [新增]
- docs/harness/done/2026-08/D0C01-syn-codex-remote-control-wsl-entry-validation.md [新增]
- docs/harness/done/2026-08/stage-10.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 固定当前 HEAD、既有 dirty 路径、A/B 归档状态和 C0 精确授权边界。
2. 记录用户明确选择按原迁移计划执行，不采用 Codex Remote Control 作为前置通道。
3. 核对本轮尚未执行设备配对、安装、网络配置、临时服务或源码迁移。
4. 归档本决策 leaf，并自动进入 D0C02 原方案链路只读配置门。

## 最终结果（2026-08-14）

- `REMOTE_CONTROL_OPTION_NOT_SELECTED`：用户明确要求按原方案执行。
- Mac 当前仍只发现 `local` host；未执行 Codex Remote Control 配对。
- 5600X、WSL、Tailscale、防火墙、SSH、端口和源码均未因本备选发生配置变更。

<!-- 先落 plan/stage/leaf 再执行。整阶段已经授权时，不逐 leaf 重复询问。
     未完成用 hl park；完成才用 hl done，后者代表完成声明并归档。 -->
