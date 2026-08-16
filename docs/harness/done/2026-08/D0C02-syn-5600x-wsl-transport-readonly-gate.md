# D0C02 原方案链路只读配置门

阶段：stage-10 阶段10 Syn 5600X/WSL 原方案 C0 只读配置门
目标：只读查清 Windows Tailscale + WSL NAT 模式下一次性链路证明所需的精确地址、端口、规则、现有监听归属和回滚检查，形成可审计的下一阶段命令单；不修改设备。
干完的标准：关键网络事实足以形成一个不复用 `19528`、只绑定 Windows Tailscale 地址、使用 `47123/TCP`、指向实时 WSL IP、最长 300 秒并可同回合撤销的 C1 命令单。按用户决定，Windows 更新、Cockpit、Tailnet ACL 和安全加固延期；C1 只证明临时链路，不证明安全或只允许 Mac。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-10.md
- docs/harness/leaves/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/unfinished/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/done/2026-08/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/done/2026-08/stage-10.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 重新固定 Mac/Windows Tailscale 地址、WSL 当前地址、网络模式和开发目录；不沿用可能漂移的旧 IP。
2. 只读核 Windows 有效防火墙 profile/default policy、Tailscale 网卡类别、Tailscale-In 规则、现有 `portproxy` 与 IP Helper 状态。
3. 只读核 `0.0.0.0:19528 cockpit-tools` 的 PID、路径、签名、服务和规则归属；不停止、不修改、不复用。
4. 只读核 Tailnet ACL（若当前账号可安全读取）、候选高位端口占用和 WSL 内目标端口占用。
5. 根据实时结果生成下一阶段精确命令单与逐项回滚命令；此 leaf 不运行它们。
6. 任一关键事实不明即停止；证据完整才归档 D0C02 和 stage-10，等待 C1 设备写入授权。

## 已完成的 Mac/官方侧只读核对（2026-08-14）

- Mac Tailscale `BackendState=Running`，地址仍为 `100.120.223.16`。
- 唯一目标 Windows peer 仍为 `DESKTOP-FRK8K62 / 100.98.94.76`，实时状态 online；单次 `tailscale ping` 成功，约 106ms。这只证明 Windows 宿主可达。
- Microsoft 官方资料显示 Windows 10 22H2 已于 2025-10-14 结束常规支持，继续接收 2026 安全更新需要正确加入 ESU；2026-08 官方最新 ESU build 为 `19045.7663`。B 阶段回执中的 `19045.6466` 必须在本 leaf 重新核实，不能直接视为当前安全维护通过。
- 5600X Windows/WSL 的实时防火墙、监听归属、ESU/补丁、当前 WSL IP 和候选端口仍待只读回传；尚未运行临时服务、端口转发、防火墙写入、SSH 安装或源码迁移。

官方依据：https://learn.microsoft.com/en-us/windows/release-health/release-information

## 5600X 转交回执与独立验收（2026-08-14）

回执来源与证据上限：

- 用户从 5600X 既有 WSL Codex 任务转回去敏只读摘要；附件 SHA-256 为 `fc6abe6e47d70890fb5e2ccae67f2beeef0e41e284c0cf844f49a91ab35ff26a`。
- 回执声明查询时 `IsAdministrator=False`，未运行 `sudo`、Git、安装、配置、服务启停、重启或临时监听；它是转交摘要，不是 Mac 直接执行所得，也不是完整原始遥测。
- 大部分现场事实可接受：WSL 为 NAT，地址 `172.18.102.245/20`；Windows Tailscale 为 `100.98.94.76/32` 且网卡类别 Private；三类防火墙默认入站 Block/出站 Allow；当前无 `portproxy`；IP Helper 正常；`47123`、`47124`、`47125` 在查询时 Windows/WSL TCP/UDP 均未占用；WSL 未安装或运行 `sshd`。

独立验收：`NO_GO_TO_C1 / SECURITY_BASELINE_BLOCKED`，不能接受回执中的“无 UNKNOWN”，也不能归档 D0C02。原因：

1. Windows 实时 build 为 `19045.6466`，对应 2025-11-11 的 `KB5071959`。Microsoft 说明该更新提供给尚未加入消费者 ESU 的设备，用于修复 ESU 注册向导；安装它本身不证明 ESU 已成功注册。2026-08 官方最新 ESU 安全 build 为 `19045.7663`，当前主机的安全维护状态未达当前基线。
2. 回执含 `KB5072653` 只能证明 ESU Licensing Preparation Package 存在；没有 ESU enrollment/licensing 的正式成功证据，也没有当前累计安全更新和无待重启证明。
3. Tailnet ACL/访问策略没有查询。当前浏览器访问 Tailscale ACL 管理页只到登录页，未读取、输入或要求任何登录凭据，因此 `TAILNET_ACL=UNKNOWN`。
4. 两条现有 `Tailscale-In` 规则在 Private/Domain profile 上绑定 Tailscale 本地地址，但 `RemoteAddress=Any`、`Protocol=Any` 且无端口/程序/服务限制。未来只新增“仅允许 Mac”的窄 allow 规则不会收窄这条既有宽 allow；Microsoft 的规则优先级也不支持靠 allow 规则顺序覆盖它。
5. `0.0.0.0:19528` 已查明为用户目录中的未签名 `cockpit-tools.exe`，无 Windows 服务关联，另有 Public profile 下 TCP/UDP、Any remote、Any port 的程序 allow 规则。未签名不等于恶意，但该监听不得复用、停止或修改；其安装来源、文件 hash、父进程/自启动和实际用途仍未核清。
6. 三个候选端口只是在查询瞬间空闲，不是保留；C1 必须只选一个，并在任何写入前同回合重新核对。WSL IP 也必须实时取值，不能硬编码本回执的地址。

解除停止门前需要的新授权（不在本 leaf 执行）：

- 单独完成 Windows 10 ESU/当前累计安全更新、必要重启和重启后验证；至少证明 build 不低于当日官方当前值、当前累计安全更新存在、ESU 已正式注册/激活且无 pending reboot。
- 只读核 Tailnet ACL；若仍无法读取，则下一阶段必须显式处理现有宽泛 `Tailscale-In` 对排他性的影响，不能声称“只允许 Mac”。
- 只读核 `cockpit-tools.exe` 的安装来源、SHA-256、父进程/自启动、当前连接和用途；不得因未签名直接处置。
- 安全基线通过后再建立独立 C1：只选 `47123/TCP`，实时冻结 Windows/WSL/Tailscale 地址，最大暴露 300 秒；一条只允许 Mac 的 allow 加一条阻断其他 Tailnet IPv4 的 block；精确绑定 `100.98.94.76` 的 `portproxy`；nonce 验证后无论成功失败均先删转发、停临时服务、删本轮规则，并验证零残留。该合同仍需新的管理员写入、临时服务和设备配置授权。

官方依据：

- https://support.microsoft.com/en-us/topic/november-11-2025-kb5071959-windows-10-version-22h2-os-build-19045-6466-out-of-band-565c78a7-5b5f-4cbd-8ca8-2a73a48f4e2b
- https://learn.microsoft.com/en-us/windows/whats-new/enable-extended-security-updates
- https://learn.microsoft.com/en-us/windows/security/operating-system-security/network-security/windows-firewall/rules

<!-- 只读查询可能留下普通系统日志；不得运行安装、配置、服务启停、监听或端口开放命令。 -->

历史停放原因（现已由下述用户决定解除）：SECURITY_BASELINE_BLOCKED：Windows 19045.6466 未证明当前 ESU/累计安全更新；Tailnet ACL 未查询；现有 Tailscale-In 宽泛 allow 使单条窄 allow 无法证明只允许 Mac；等待独立安全维护授权

## 用户决定与 C0 最终口径（2026-08-14）

- 用户明确决定：当前先不处理 Windows 更新、Cockpit、Tailnet ACL 和安全加固，以最快可逆方式证明原方案链路，安全防护以后单列。
- 上述旧停放原因作为历史审计保留，但不再阻断 C0/C1。`19528`、Cockpit、既有 `Tailscale-In` 和其他现有监听均不得修改。
- 现有 `Tailscale-In` 是宽泛 allow，因此快速 C1 不新增防火墙规则，也不能签发 `Mac-only` 或安全验收；在约 300 秒测试窗口内，其他被当前 tailnet policy 放行的节点理论上也可能尝试访问该测试端口。

## 已冻结的快速 C1 命令合同

1. 固定 `47123/TCP`。执行前实时确认 Windows 与 WSL 都没有该端口监听、没有同地址同端口的既有 `portproxy`，并确认 WSL 已有 `python3`；任一不满足即停止，不换端口、不安装软件。
2. 运行时重新取得 Windows Tailscale IPv4（预期 `100.98.94.76`）和 Ubuntu `Ubuntu-24.04` 的当前 WSL IPv4；地址不匹配或无法唯一取得即停止，不使用旧回执里的 WSL IP。
3. 生成一次性 run id 与随机 nonce；在 WSL 当前 IPv4 上启动最长 300 秒的内存型临时 HTTP 响应，只返回 proof、nonce、发行版、用户和 WSL kernel 标识，不读取源码、凭据或业务数据。
4. 只新增一条精确 `portproxy`：`<Windows Tailscale IPv4>:47123 -> <实时 WSL IPv4>:47123`；禁止绑定 `0.0.0.0`，不新增或修改任何防火墙、Tailscale、ACL、Cockpit、SSH、服务或计划任务。
5. Windows 先本地取得同一 nonce；输出 `C1_READY` 后，Mac 用 `curl --noproxy '*'` 访问 Windows Tailscale 地址并核对 HTTP 200、远端 IP、nonce 与 WSL 标识。此时最多记为 `C1_CHAIN_PASS_PENDING_ROLLBACK`。
6. 无论成功、失败或超时，都先精确删除本轮 `portproxy`，再停止本轮临时监听；核 Windows/WSL 无 `47123` 监听且 Mac 再访问失败，才可记为 `C1_PASS_ROLLED_BACK`。任何残留标记 `ROLLBACK_INCOMPLETE` 并停止。

## C0 最终结论

- `C0_COMMAND_CONTRACT_READY`：现场地址、NAT 拓扑、候选端口、现有 `portproxy` 空状态和回滚边界已足以生成上述快速合同。
- 本结论不是设备写入授权；C1 的临时监听、管理员 `portproxy` 和 Mac 实测必须进入独立 stage/leaf 并取得精确授权。
