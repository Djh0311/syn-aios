# 阶段11 Syn 5600X/WSL C1 临时链路证明

总计划：product-line 唯一基线与 Harness Lite 切换
目标：按用户选择的最快原方案，仅证明 Mac 能经 Windows Tailscale 地址和一条临时 `portproxy` 到达 WSL 内的 nonce 服务，并在同回合完整回滚。本阶段不做 Windows 安全加固，不碰 Cockpit，不修改 Tailscale/ACL，不安装 SSH，也不迁移源码；Attempt3 仅例外允许一条本轮唯一命名、精确到 Mac/Windows Tailscale 地址和 `47123/TCP` 的临时入站规则，并必须同回合删除。

编号说明：这里的 Harness `stage-11` 是开发护栏编号，不是产品 M11，也不激活 M5–M10。

干完的标准：

- 执行前实时核对设备、发行版、用户、Windows Tailscale IPv4、WSL IPv4、`47123/TCP` 空闲、同地址同端口 `portproxy` 不存在且 WSL 已有 `python3`。
- WSL 只启动最长 300 秒、只返回本轮随机 nonce 和 WSL 身份标识的临时监听；不读取源码、凭据或业务数据。
- Windows 只新增 `<实时 Windows Tailscale IPv4>:47123 -> <实时 WSL IPv4>:47123` 的临时 `portproxy`，禁止 `0.0.0.0`。Attempt3 另只新增一条唯一命名的临时 allow：`100.120.223.16 -> 100.98.94.76:47123/TCP`；不修改任何既有规则。
- Attempt3 规则固定名为 `Syn-C1-Attempt3-47123`，执行前必须不存在；Direction=Inbound、Action=Allow、Enabled=True、Profile=Any、LocalAddress=`100.98.94.76`、RemoteAddress=`100.120.223.16`、Protocol=TCP、LocalPort=`47123`。
- Mac 使用 `curl --noproxy '*'` 取得 HTTP 200、正确远端 IP、同一 nonce 和 WSL 标识。
- 无论成功、失败或超时，精确删除本轮 firewall rule 和 `portproxy`、停止本轮 listener，并由 Windows/WSL/Mac 三侧确认规则、入口和监听均已消失。
- 只有“链路成功 + 回滚成功”才能记为 `C1_PASS_ROLLED_BACK`；不签发安全、Mac-only、长期稳定、SSH、开发环境、Headless Core 或 Primary 验收。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-11.md
- docs/harness/leaves/D0C03-syn-5600x-wsl-fast-temporary-transport-proof.md
- docs/harness/unfinished/D0C03-syn-5600x-wsl-fast-temporary-transport-proof.md [新增]
- docs/harness/done/2026-08/D0C03-syn-5600x-wsl-fast-temporary-transport-proof.md [新增]
- docs/harness/done/2026-08/stage-11.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

实际设备动作（必须等精确 external 授权）：

- 5600X Windows/WSL 对 `47123/TCP` 做本轮只读冻结。
- WSL 启动最长 300 秒的本轮 nonce listener。
- 管理员 PowerShell 新增并删除一条只绑定 Windows Tailscale IPv4 的本轮 `portproxy`。
- Attempt3 新增并删除一条本轮唯一命名的精确 Windows 入站 allow rule；不碰任何既有防火墙规则。
- Mac 对本轮 URL 做一次成功探测和回滚后失败探测。

只读：

- docs/harness/done/2026-08/stage-10.md
- docs/harness/done/2026-08/D0C02-syn-5600x-wsl-transport-readonly-gate.md
- docs/plans/2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md
- 本轮命令输出中的设备名、非秘密 IP、端口、nonce、listener 与 portproxy 状态

不许动：

- Windows 更新、ESU、Cockpit 进程/文件/规则、`19528`、除 Attempt3 本轮精确 allow rule 之外的防火墙、`Tailscale-In`、Tailnet ACL、Tailscale 配置
- 安装、升级、卸载、重启、SSH、Docker、Hyper-V、systemd、持久服务、计划任务或永久监听
- `47123/TCP` 之外的端口，或绑定 `0.0.0.0`
- 密码、私钥、令牌、凭据、源码、业务数据、活动 SQLite、环境文件
- 源码迁移、产品代码/测试修改、Git add/commit/push/merge/rebase/reset/clean/stash
- M5–M10、Headless Core/Edge、Primary 数据、authority epoch、部署或发布
- 既有 Harness 归档、M1–M4 与 M4R07 receipt/manifest

停止与回滚：

- 地址/设备不匹配、端口占用、已有同目标 portproxy、固定名 firewall rule 已存在、缺少 `python3`、不是管理员、nonce 不一致或需要碰既有配置时立即停止。
- 任何已创建的本轮对象都必须进入 `finally`：先删本轮 portproxy 和精确 firewall rule，再停本轮 listener，最后核零残留；回滚不完整就保留 stage/leaf 为未完成并报告精确残留。

## 叶子

- [x] D0C03 快速临时链路证明与完整回滚
