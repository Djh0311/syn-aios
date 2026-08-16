# 阶段10 Syn 5600X/WSL 原方案 C0 只读配置门

总计划：product-line 唯一基线与 Harness Lite 切换
目标：回到迁移计划原定的“Windows 宿主 Tailscale + WSL NAT + 受限转发/SSH”路线；本阶段只读查清未来一次性链路证明所需的精确参数和回滚边界，不安装软件，不修改 Windows/WSL/Tailscale/SSH/防火墙/端口，也不迁移源码。Windows 更新、Cockpit、Tailnet ACL 和安全加固已由用户明确延期，不作为本阶段或下一次临时可达性证明的阻断项。

编号说明：这里的 Harness `stage-10` 只是第十个开发护栏阶段，承载迁移计划 C 的 C0 只读配置门；它不是产品路线中的 M10，不激活 M5–M10。

干完的标准：

- Remote Control 备选明确记录为用户未采用；不配对，不把它设为原方案前置条件。
- 只读核对 WSL 当前地址、Windows Tailscale 地址与网卡类别、有效防火墙策略、现有 `portproxy`、IP Helper、未占用高位端口和 Tailnet 访问策略（若当前身份安全可读）。
- 只读查清 `0.0.0.0:19528 cockpit-tools` 的 PID、程序路径、签名、服务和防火墙规则归属；不停止、不修改、不复用该监听。
- 形成下一阶段唯一允许的一次性命令单：一个最长 300 秒、只返回随机 nonce 的临时无敏感服务；一个只绑定 Windows Tailscale 地址、指向实时 WSL IP 的 `47123/TCP` 精确转发；Mac 端 nonce 验证；以及无论成功失败都执行的完整回滚和残留检查。本轮依赖现有 `Tailscale-In`，不新增或修改防火墙规则，不能宣称只允许 Mac。
- 任一关键边界不明就停止；不把只读预检冒充 Mac 已能访问 WSL 服务。
- 不 Git add/commit/push，不修改产品代码、Headless Core、Primary 数据或 authority epoch。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-10.md
- docs/harness/leaves/D0C01-syn-codex-remote-control-wsl-entry-validation.md
- docs/harness/leaves/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/unfinished/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/unfinished/D0C01-syn-codex-remote-control-wsl-entry-validation.md [新增]
- docs/harness/done/2026-08/D0C01-syn-codex-remote-control-wsl-entry-validation.md [新增]
- docs/harness/done/2026-08/D0C02-syn-5600x-wsl-transport-readonly-gate.md [新增]
- docs/harness/done/2026-08/stage-10.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

允许的外部只读动作：

- 用户只需将我给出的单段只读检查提示粘贴到 5600X 既有的 WSL Codex 任务；若需要管理员权限才能读取，先停下回报，不自动提权。
- 5600X 任务只读查询 Windows、WSL、Tailscale、防火墙、端口转发、监听和服务归属，并回传去敏结果。
- Mac 端只读复核本机 Tailscale 状态、目标端口占用和回传证据的一致性。
- 只读查询可能留下 Codex、Windows、Tailscale 和 shell 普通日志；不把它冒充绝对零写入。

只读：

- docs/plans/2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md
- docs/harness/done/2026-08/stage-09.md
- docs/harness/done/2026-08/D0B01-syn-5600x-wsl-readonly-preflight.md
- 5600X Windows/WSL/Tailscale 的系统、网络、服务、签名、监听和规则状态
- Mac Tailscale 本机与 peer 状态

不许动：

- Windows、WSL2、Tailscale、SSH、防火墙、`portproxy`、Hyper-V、Docker、systemd、服务、计划任务、Codex Remote Control 或其他设备/应用配置
- 安装、升级、卸载、启停服务、重启、启动临时服务、开放端口或新建 SSH 入口
- 读取或输出密码、私钥、令牌、Tailscale 凭据或其他秘密
- 源码、测试、依赖、构建产物、运行数据、活动 SQLite、环境文件和凭据
- M5–M10 激活或实现、Headless Core/Edge 实现、Primary 数据迁移或 authority epoch 切换
- Git add、commit、push、merge、rebase、reset、clean、stash、删除或覆盖既有工作
- 现行产品/架构正本、当前状态、M1–M4、已归档 stage/leaf 和 M4R07 receipt/manifest

停止与回滚：

- 读取需要安装、提权、启停服务或修改设置时立即停止；不把“为了查看”扩大成配置授权。
- 本阶段原则上无主动配置变更，无设备配置需要回滚；普通查询日志如实作为环境副作用记录。

## 叶子

- [x] D0C01 Remote Control 备选退场决策
- [x] D0C02 原方案链路只读配置门

## 用户风险决定（2026-08-14）

- 用户明确表示当前环境按其判断可接受，要求优先完成迁移链路，Windows 安全更新、Cockpit、Tailnet ACL 和安全加固以后另做。
- 因此 C1 的结论上限只允许写成“临时链路可达并已回滚”，不签发 Windows 安全、Tailnet 排他访问、长期稳定、SSH、Headless Core 或 Primary 通过。
- `0.0.0.0:19528` 和既有防火墙规则全部排除：不测试、不停止、不修改、不复用。
