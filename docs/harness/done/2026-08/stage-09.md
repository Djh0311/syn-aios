# 阶段9 Syn 5600X/WSL/Tailscale B 只读预检

总计划：product-line 唯一基线与 Harness Lite 切换
目标：只读取 Mac、5600X Windows、WSL2 与 Tailscale 的现有状态，确认设备、容量、网络和远程入口事实；不安装、不升级、不配置、不迁源码，遇到没有现成远程通道或任何必须修改设置的情况立即停止。

编号说明：这里的 Harness `stage-09` 只是第九个开发护栏阶段，承载 D0 的 B 只读预检；它不是产品路线中的 M9 阶段，不激活 M5–M10。

干完的标准：

- 唯一 5600X 设备、Windows/WSL2 版本、CPU/内存/GPU、虚拟化、磁盘余量和 WSL 目标目录事实已核对。
- 两端 Tailscale 登录状态、设备身份、地址和 Windows 宿主可达性已核对；不把宿主可达冒充 WSL 服务可达。
- 仅盘点现有 WSL 网络模式、端口转发、防火墙、OpenSSH、监听和绑定；不新建服务、端口或远程入口。
- 若没有现成远程通道，D0B01 立即停止并停放，只向用户提供一段最小复制粘贴的只读命令，不冒充 `DEVICE_PRECHECK_PASS`。
- 若现有通道足够，形成聊天内 `DEVICE_PRECHECK_PASS` 回执和 C 阶段精确配置清单；不在仓内新增设备证据文件。
- 不 Git add/commit/push，不修改产品代码、设备配置、Headless Core、Primary 数据或 authority epoch。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-09.md
- docs/harness/leaves/D0B01-syn-5600x-wsl-readonly-preflight.md
- docs/harness/unfinished/D0B01-syn-5600x-wsl-readonly-preflight.md
- docs/harness/done/2026-08/D0B01-syn-5600x-wsl-readonly-preflight.md
- docs/harness/done/2026-08/stage-09.md
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

只读：

- docs/plans/2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md
- docs/product/syn-primary-edge-core-distributed-runtime-architecture-candidate-v2.md
- Mac 上现有 Tailscale 状态和既有远程连接定义，不读取或输出密钥、令牌、密码
- 5600X Windows 与 WSL2 的系统、资源、磁盘、网络、进程和监听状态
- 仓库的 Git LFS、submodule、忽略规则和运行前置事实

不许动：

- Windows、WSL2、Tailscale、SSH、防火墙、端口转发、Hyper-V、Docker、systemd、服务、计划任务或其他设备配置
- 安装、升级、卸载、启停服务、重启设备、启动临时服务、开放端口或创建新的远程通道
- 源码、测试、依赖、构建产物、运行数据、活动 SQLite、环境文件、凭据和密钥
- M5–M10 激活或实现、Headless Core/Edge 实现、Primary 数据迁移或 authority epoch 切换
- Git add、commit、push、merge、rebase、reset、clean、stash、删除或覆盖既有工作
- 现行产品/架构正本、当前状态、M1–M4、已归档 stage/leaf 和 M4R07 receipt/manifest

环境副作用说明：只读远程连接和系统查询可能留下 Tailscale、SSH、Windows 或 shell 的普通访问日志；这不等于业务配置写入，但必须在回执中如实说明。

## 叶子

- [x] D0B01 5600X、WSL2 与 Tailscale 只读预检
