# D0B01 5600X、WSL2 与 Tailscale 只读预检

阶段：stage-09 阶段9 Syn 5600X/WSL/Tailscale B 只读预检
目标：通过现有 Tailscale 和既有远程通道核对 Mac、5600X Windows 与 WSL2 的真实状态，为 C 阶段形成精确、可回滚的配置输入；没有现成通道时停止并交付最小只读命令块。
干完的标准：完成设备、WSL、容量、Tailscale、现有网络暴露和仓库搬运前置核对，形成聊天内 `DEVICE_PRECHECK_PASS`；或明确停在 `REMOTE_CHANNEL_MISSING`，不把用户陈述、Tailscale 宿主可达或未执行服务探测冒充通过。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-09.md
- docs/harness/leaves/D0B01-syn-5600x-wsl-readonly-preflight.md
- docs/harness/unfinished/D0B01-syn-5600x-wsl-readonly-preflight.md [新增]
- docs/harness/done/2026-08/D0B01-syn-5600x-wsl-readonly-preflight.md [新增]
- docs/harness/done/2026-08/stage-09.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 固定当前 HEAD、既有 dirty 路径、A 阶段归档状态和本 leaf 的精确授权边界。
2. 只读核对 Mac 的 Tailscale 安装、登录、设备地址和现有远程连接定义，不输出凭据。
3. 识别唯一 5600X Tailscale 节点并核 Windows 宿主可达；不把它冒充 WSL 或 Syn 服务可达。
4. 仅在已有远程通道存在时，读取 Windows、WSL2、容量、虚拟化、网络、SSH、防火墙、portproxy 和监听状态；不执行任何修改型命令。
5. 只读核仓库的 LFS、submodule、私有依赖和忽略文件边界，为后续源码搬运停止门提供输入。
6. 若无现成远程通道，立即停止并停放 D0B01，向用户提供一段最小复制粘贴命令；不得安装或配置远程入口。
7. 若证据完整，输出 `DEVICE_PRECHECK_PASS` 与 C 阶段精确清单，机械验证后归档 D0B01 和 stage-09，并停止等待新授权。

未完成原因：REMOTE_CHANNEL_MISSING：Tailscale Windows 宿主可达，但 Mac 无既有 SSH 配置且目标 22 端口超时；按用户授权停止，等待用户回传最小只读命令输出

未完成原因：REMOTE_CHANNEL_MISSING：当前 Mac 任务仅有 local host，无法调用 5600X 的 Windows/WSL Codex 任务；Mac 与仓库侧只读预检已完成，等待 Windows/WSL 任务回传最小只读检查原始输出

## B 阶段最终只读验收（2026-08-13）

结论：`DEVICE_PRECHECK_PASS`，仅表示 B 阶段的设备、容量、现有网络与暴露面盘点足以作为 C 阶段的配置输入；不表示 Mac 已能访问 WSL 服务、设备已安全加固、开发环境已重建或 5600X 已成为 Primary。

证据来源与上限：

- Mac 侧实时只读复核：Tailscale `BackendState=Running`；Mac 地址 `100.120.223.16`；唯一 Windows peer 为 `DESKTOP-FRK8K62 / 100.98.94.76`，在线；最后一次 `tailscale ping` 成功，约 8ms。这只证明 Windows 宿主可达。
- 5600X 侧原始摘要回执由用户从 Windows/WSL Codex 任务转回，附件 SHA-256 为 `6eaed0cb4ca0fc30c213451502d0cb7a0372e3e24475f4f393d6c37c2a79f74c`。它是转交的查询回执，不是 Mac 直接执行所得，也不是防篡改遥测。
- 仓库侧只读复核：Git LFS 文件 0，submodule 0，未发现 Git/SSH 私有依赖；`target` 约 51GB、`node_modules` 约 87MB、`dist` 约 1.6MB，均为被忽略生成物，不进入后续源码迁移。

已核心对的设备事实：

- Windows 10 Pro 22H2，build `19045.6466`；Ryzen 5 5600X，6 核 12 线程；宿主内存约 48GiB；RTX 3060 Ti；虚拟化和 Hypervisor 均已启用。
- Ubuntu 24.04 WSL2，默认用户 `synadmin` / UID 1000；systemd 为 `running`，0 个失败单元。
- 发行版位于 `A:\WSL\Ubuntu-24.04`，VHDX 为 `A:\WSL\Ubuntu-24.04\ext4.vhdx`；开发目录 `/home/synadmin/workspace/syn` 位于 ext4 内，可用约 955GiB，高于计划中 120GiB 的停止线。
- 当前现象符合 WSL2 NAT：WSL 地址 `172.18.102.245/20`，无 `.wslconfig`，无 `portproxy`，Windows 无 `sshd` 服务，WSL 只见回环 DNS 监听 `127.0.0.53:53` 与 `127.0.0.54:53`。
- Windows Tailscale 本机为 `DESKTOP-FRK8K62 / 100.98.94.76`，与 Mac 侧 peer 身份一致。

网络证据分层：

1. Mac → Windows Tailscale：`PASS`。
2. Mac → Windows 指定 TCP 服务：`NOT_TESTED / DEFERRED_TO_C`。
3. Mac → Windows Tailscale → WSL 临时服务：`NOT_TESTED / DEFERRED_TO_C`。

仍未知或必须带入 C 的停止门：

- Tailnet 完整 ACL 策略未读取；Windows 防火墙 `NotConfigured` 最终继承的有效默认策略不明。
- WSL 两个 53 端口的进程归属未由普通用户权限解析。
- Windows 存在 `Tailscale-In / Protocol Any` 规则，同时有 `0.0.0.0:19528 cockpit-tools` 等现有监听。它们只是现有暴露候选，不能当远程入口或安全证据；C 在新增任何入口前必须核实有效防火墙边界、`19528` 的程序身份/用途与当时端口占用。
- Windows 当前安全维护状态未核实；在建立持久 SSH 入口前必须先确认。

C 阶段候选输入（仅规划，未授权执行）：

- 保持“Windows 宿主 Tailscale + WSL NAT”作为起点；不在 B 阶段新增 WSL Tailscale 身份，不把 mirrored 当成已实现方案。
- 若需 Mac 直连 WSL，C 内只做一个可撤销测试：精确高位端口、只绑定 Windows Tailscale 地址、防火墙只放行 Mac `100.120.223.16/32`，临时服务回应带随机 nonce/WSL 身份，验证后删除临时服务、规则和转发。
- 安装、管理员查询/修改、防火墙、`portproxy`、SSH、服务启停和重启必须在新的 C 阶段精确授权中单列。

副作用边界：本次回执声明其只读查询未主动修改配置、未启停服务、未运行 Git；只读查询可能留下系统普通日志。B 阶段期间用户曾自行完成 WSL 搬迁和 ChatGPT 客户端重装，因此不声称整个 B 时间段“绝对零写入”。
