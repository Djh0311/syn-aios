# D0C04 长期 SSH 开发通道配置与当前启动周期验收

阶段：stage-12 阶段12 Syn 5600X/WSL C2 长期 SSH 开发通道
目标：配置 Mac 到 `Ubuntu-24.04 / synadmin` 的持久 SSH/Remote-SSH 入口，并在不重启 Windows/WSL 的当前启动周期内完成重复连接验收。
干完的标准：WSL sshd、公钥、Windows 精确持久 rule/portproxy 和 Mac Host 配置均落盘；Mac 至少两次全新 SSH 连接取得正确 WSL 身份；所有变更均有精确回滚；签发 `PERSISTENT_SSH_READY_CURRENT_BOOT`，但不冒充重启稳定。

用户的新决定：C1 成功后，用户明确要求把已验证路线落实为长期开发通道。该决定只 supersede 迁移计划 v1 对 NAT portproxy 的“仅临时”限定；其余 DRAFT、分项授权和产品边界不变。

用户的精确 package 决定：在同版本零升级方案被只读证伪、候选四包模拟清单和失败残留风险全部明示后，用户回复“批准”。该批准只允许一个既有 package 升级和三个 package 新装，不放开任何其他系统升级。

用户的 Attempt4 决定：Attempt3 的四包事务已完成，但在服务启动前因 `/run/sshd` 尚不存在而失败。用户在收到精确失败回执、保留项、旧脚本不可重放和最小恢复范围后明确回复“批准”。Attempt4 只做 retained-state recovery：不再运行 apt 或改变 package/host key/旧 anchor，只验证 package unit 的 RuntimeDirectory 合同，并由 systemd 管理 `/run/sshd` 后继续原定 SSH/Windows/Mac 验收。

冻结合同：

- Mac key：`/Users/yoyi/.ssh/syn_5600x_wsl_ed25519` 与同名 `.pub`；Host：`syn-5600x-wsl`；专用 host-key 文件：`/Users/yoyi/.ssh/known_hosts_syn_5600x_wsl`。模型不查看/输出/复制/传出私钥，只有本机标准 SSH 工具可为认证读取，私钥权限 `0600`。
- WSL drop-in：`/etc/ssh/sshd_config.d/90-syn-development.conf`；authorized keys：`/home/synadmin/.ssh/authorized_keys`，只追加本轮公钥行。
- sshd 有效值：`ListenAddress 127.0.0.1`、`AllowUsers synadmin`、`PubkeyAuthentication yes`、`AuthenticationMethods publickey`、`PasswordAuthentication no`、`KbdInteractiveAuthentication no`、`PermitRootLogin no`；用 `sshd -t` 与 `sshd -T -C` 验证。
- Ubuntu 24.04 unit：Attempt4 预检必须证明 package `ssh.service` 已加载、没有未知 drop-in，精确声明 `RuntimeDirectory=sshd`、`RuntimeDirectoryMode=0755`、`RuntimeDirectoryPreserve=no`，并由 root 运行 package 自带的 `sshd -t` ExecStartPre 与 `sshd -D` ExecStart。`ssh.service`/`ssh.socket` 前态必须 disabled/inactive 且 TCP 22 无监听。任何一项不符立即停止，不手工创建或覆盖 `/run/sshd`。
- Retained package baseline：`openssh-client/server/sftp-server=1:9.6p1-3ubuntu13.18`、`libwrap0=7.6.q-33` 必须均为 `install ok installed`，`dpkg --audit` 为空，SSH client conffile/config tree 和 host key 未漂移。Attempt4 禁止所有 apt/package mutation、source/index/hold/pin 与 host-key 生成/替换/删除。
- Recovery transaction：旧 anchor `/var/backups/syn-d0c04-06bee21f1b764dc780b0e1409a2c2651` 只读核验并永久保留，不复用、不修改、不清理。Attempt4 使用新的 `/var/backups/syn-d0c04-attempt4-<runid>` journal，只记录本轮 drop-in、authorized_keys、tmp、`.ssh`、service/socket、Windows 和 Mac 自定义对象的前态与所有权。
- Windows portproxy：`100.98.94.76:47123 -> 127.0.0.1:22`；firewall rule：`Syn-WSL-SSH-47123`，Inbound/Allow/Enabled/Profile Any，LocalAddress `100.98.94.76`、RemoteAddress `100.120.223.16`、TCP/LocalPort `47123`。
- host key：先由 Windows/WSL 本地回执给出 Ed25519 指纹，再与 Mac `ssh-keyscan` 的指纹比较，一致后才写 known_hosts。
- 正负验收：正确 key 两次全新 SSH 连接成功并返回 `synadmin / Ubuntu-24.04 / WSL2 / workspace`；无 key/密码路径失败；root 使用正确 key 也失败。

允许动：

- docs/harness/authorization.json
- docs/harness/stages/stage-12.md
- docs/harness/leaves/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md
- docs/harness/unfinished/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md [新增]
- docs/harness/done/2026-08/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 只读冻结 Attempt3 retained state：四包精确版本与完整 dpkg delta、dpkg audit、SSH client 配置、host key、旧 anchor、Syn 自定义配置/service/socket/监听、Windows rule/portproxy/47123 与 Mac Host/known_hosts；任何漂移或碰撞立即停止。
2. 复用 Mac 上已存在的固定路径专用 Ed25519 keypair，只允许标准 SSH 认证 act；不得创建、重建、修改或删除 keypair。只把既有公钥传入 WSL，私钥保持本机、权限 `0600` 且不进入模型输出。
3. 新建独立 Attempt4 recovery journal，以 no-clobber/精确 hash 方式建立 Syn drop-in并只追加本轮公钥；再次复核 immutable baseline 与 RuntimeDirectory 合同，保持 `ssh.socket` disabled/inactive，然后先 `systemctl start ssh.service`，让 systemd 创建 `/run/sshd` 并执行 package 的 ExecStartPre。启动后再独立核 `/run/sshd`、`sshd -t`/`sshd -T -C`、service/socket 与唯一 `127.0.0.1:22`；全部通过后才 enable service。
4. WSL sshd 只监听 Linux `127.0.0.1:22`；Windows 必须从 `127.0.0.1:22` 读取到 `SSH-2.0-OpenSSH` banner，才允许新增唯一持久 rule 和 `100.98.94.76:47123 -> 127.0.0.1:22` portproxy；失败不放宽监听。
5. Mac 从已信任的 Windows/WSL 回执取得 host-key 指纹，核 `ssh-keyscan` 指纹一致后写专用 `Host syn-5600x-wsl` 与 known_hosts；做两次正确 key 全新连接以及无 key/密码/root 负向测试。
6. 保存实际配置、验收和逐项回滚回执；不执行重启，不自动进入 D0C05。

## 当前授权状态

- 用户明确要求：临时连通验证后应按迁移计划建立长期通道。
- 用户随后在收到 D0C04 的精确设备写入范围后明确回复“批准”。
- 当前授权对既有 Mac keypair 只允许标准 SSH act；只允许创建/修改本轮 Host managed block 与专用 known_hosts。Ubuntu `openssh-server`、Syn 专用 sshd drop-in/公钥/服务、Windows loopback 前检、唯一持久 rule/portproxy、两次 Mac SSH 验收及失败时精确回滚仍按本 leaf 精确范围执行。
- 用户随后在被明确告知 Ubuntu 24.04 默认 `ssh.socket` 激活及原授权缺口后回复“允许”；当前授权因此新增且仅新增：可禁用/停止本轮安装产生的 `ssh.socket`，改由已批准的 `ssh.service` 运行，并在失败时确保二者无监听。禁止 package 卸载/purge、其他 systemd unit 或重启。
- 用户随后逐字批准 apt 必需依赖与最终事务辅助路径：禁止 recommends/升级/替换/删除，允许无碰撞时创建并清理本轮 policy、run-id journal、两个具名 tmp 和本轮新建空 `.ssh`；失败时 package/host key 保留并如实报告，只撤销本轮配置、规则和转发。
- Attempt2 证明旧 no-upgrade 合同无法安装：现有 client 为 `.14`，源中没有 server/sftp `.14`；候选 `.18` 模拟恰好是一升三装。用户随后明确批准：只把 client `.14 -> .18`，只新增 server/sftp `.18` 和 `libwrap0 7.6.q-33`；失败时不降级、卸载或 purge，允许保留四包、package unit、host key 和 apt index，只回滚本轮 Syn/Windows/Mac 自定义对象。
- Attempt3 已实际完成上述一升三装并按合同保留四包、package unit、host key 与旧 anchor；在 `sshd -t` 因 `/run/sshd` 尚不存在而停止。用户随后批准 Attempt4 仅恢复服务启动顺序，不再运行 apt 或 package mutation；当前授权 id 为 `USER-SYN-D0C04-ATTEMPT4-RUNTIME-DIRECTORY-RECOVERY-STAGE-12-20260814`，scope 精确为 `leaf:D0C04`，D0C05 仍无执行授权。
- 当前仍禁止 Windows/WSL 重启、计划任务、动态 WSL-IP 转发、既有规则/Tailscale/Cockpit、源码迁移和 Git 写入；D0C05 未获设备执行授权。

## Mac 专用密钥回执（2026-08-14）

- 执行前确认 `/Users/yoyi/.ssh` 为当前用户所有且权限 `0700`；专用 key、`.pub` 与 `Host syn-5600x-wsl` 均无碰撞。
- 已创建 `/Users/yoyi/.ssh/syn_5600x_wsl_ed25519` 与 `.pub`；私钥权限 `0600`，公钥权限 `0644`。
- 公钥指纹：`SHA256:GzRyrzalVJ7eboufrfrCT1vZFMra/Rl4/ae7WRECRoc`；公钥注释：`syn-5600x-wsl-20260814`。
- 本轮按用户“怎么快怎么来、后续再加安全防护”的既有决定创建为无 passphrase 专用 key；私钥内容未读取、未输出、未复制或传输。
- `~/.ssh/config` 与专用 `known_hosts_syn_5600x_wsl` 尚未创建；必须等 WSL 本地 host-key 公钥/指纹回执并完成交叉核对后再写。

## D0C04 设备脚本 Attempt1（2026-08-14）

- 用户转回：`D0C04_COLLISION_FREEZE=PASS`，说明 Windows/WSL 身份和固定入口碰撞前检已通过。
- 随后 Python `-c` 在 `<string>` 第 1 行发生语法错误；根因属于 Windows PowerShell 5.1 → `wsl.exe` 对 launcher 内层引号的参数重解析，Linux payload 未进入执行。
- PowerShell 退场回执：`ROLLBACK=PASS`、`FIREWALL_RULE_Syn-WSL-SSH-47123=ABSENT`、`PORTPROXY_47123=ABSENT`、`WSL_D0C04_CHANGES=REVERTED`。
- 结构化结论：`ATTEMPT1_RESULT=LAUNCHER_QUOTING_FAILED_BEFORE_LINUX_PAYLOAD`；`OPENSSH_INSTALL=NOT_EXECUTED`；`FIREWALL_ADD=NOT_EXECUTED`；`PORTPROXY_ADD=NOT_EXECUTED`；`DEVICE_RESIDUAL_CHECK=PASS_BY_USER_RETURNED_TEXT`；`MAC_KEY=RETAINED_AS_INTENDED`；`PERSISTENT_SSH_READY_CURRENT_BOOT=NOT_ISSUED`。
- 当前授权并非单次尝试授权，允许只修正同一 exact target 内的 launcher quoting 后重试；不得借机扩大到 D0C05、重启、计划任务或其他网络方案。

## D0C04 launcher 只读自检（2026-08-14）

- 用户在运行双层 Base64、无安装/无配置写入的 PowerShell → `wsl.exe` → Python → Bash 自检后报告“符合预期”。
- 证据口径：`D0C04_LAUNCHER_SELFTEST=PASS_BY_USER_REPORT`；用户未粘贴独立原始输出或附件，因此不升级为本任务直接终端证据。
- 该结果只证明 Attempt1 的 native argv/内层引号故障已被绕开；不证明 `openssh-server` 已安装、sshd 已配置、Windows loopback/portproxy 已通过或长期 SSH 已建立。
- 后续只允许从完整 collision freeze 与事务起点重新执行同一 D0C04 合同，不从 Attempt1 中间续跑。

## D0C04 正式重试候选（2026-08-14）

- 候选脚本：`/private/tmp/syn-d0c04-retry-v2.ps1`；SHA-256：`37ec99b0f72446f9556cbd63e2fd6989fdcc39655adbe956b6550c633316d72b`；`1678` 行、`66281` bytes。
- 五段内嵌 Bash 已分别通过 `bash -n`：Preflight `3400` bytes、Apply `22347` bytes、Rollback `9503` bytes、CommitCheck `4400` bytes、Finalize `1467` bytes；对应 Base64 长度分别为 `4536 / 29796 / 12672 / 5868 / 1956`。
- 当前 Mac 没有 Windows PowerShell 5.1 运行时；PowerShell 部分只做了静态兼容性审查，未冒充 5600X 实际解析或执行通过。先前用户回报的双层 Base64 launcher 自检符合预期，只覆盖参数传递链。
- 当前 authorization id `USER-SYN-D0C04-PERSISTENT-SSH-TRANSACTION-EXTENSION-STAGE-12-20260814` 下，已登记本次实际范围的 `11` 条 external allow 与 `4` 条 destructive rollback allow；旧授权回执未被复用。
- 事务模型：管理员 PowerShell 必须保持为单写者并一直开启到精确 `COMMIT <RunId>` 或 `ROLLBACK <RunId>`。portproxy 没有原生 run-id 所有权元数据；若窗口、PowerShell 进程或主机异常终止，状态一律为 `TRANSACTION_STATE_UNKNOWN`，禁止重跑或手工删除，需先只读恢复审计。
- READY 只表示设备侧当前启动周期候选已就绪；必须先取得独立 Mac host-key 对齐、两次全新正向 SSH 与无 key/password/root 负向回执，才能输入 COMMIT。COMMIT 文本本身不是 Mac 验收证据。
- `EXECUTION=NOT_YET_EXECUTED`；`PERSISTENT_SSH_READY_CURRENT_BOOT=NOT_ISSUED`；D0C04 保持 current，D0C05 保持未授权/未启动。

## D0C04 设备脚本 Attempt2（2026-08-14）

- 用户转回附件 SHA-256：`3b0cd405cac2ad4ddced76769526277af48eba0015602917b0a130b1d08f93ec`；证据来源为用户转回文本，不是本 Mac 任务直接取得的 5600X 终端遥测。
- `LINUX_PREFLIGHT=PASS`、`D0C04_COLLISION_FREEZE=PASS`；随后 `apt-get update` 下载约 `36.5 MB` 并刷新软件索引，脚本明确标记 `LINUX_APT_INDEX_REFRESH=PASS_NOT_ROLLED_BACK`。该索引、缓存、时间戳和日志变化保留，不能宣称 WSL 字节级零残留。
- apt 模拟计划发现安装候选会升级既有 `openssh-client`，触发 `LINUX_ERROR=apt_plan_would_upgrade_existing:openssh-client`，依照用户最新的 no-upgrade 合同在正式安装前以 exit `48` 停止。
- 回执支持：`LINUX_RETAINED_PACKAGES=` 空、`LINUX_DPKG_STATE_DELTA=0`、`LINUX_HOST_KEY_RESIDUAL=0`、`LINUX_SSH_PACKAGE_ARTIFACT_RESIDUAL=0`；即没有安装或升级 dpkg 包，没有留下 openssh-server、host key 或本轮 SSH package artifact。
- `ROLLBACK_PORTPROXY=NOT_ATTEMPTED_PRESERVED`、`ROLLBACK_FIREWALL=NOT_ATTEMPTED_PRESERVED`；Windows firewall/portproxy 本轮未尝试写入。本轮自定义 Linux 配置与事务对象回滚为 PASS/NOT_NEEDED。
- 结构化结论：`STOPPED_BEFORE_PACKAGE_INSTALL_FOR_FORBIDDEN_OPENSSH_CLIENT_UPGRADE`；`EXACT_CONFIG_AND_WINDOWS_ROLLBACK_PASS_WITH_APT_INDEX_REFRESH_RETAINED`；`READY=NOT_REACHED`；`MAC_SSH_ACCEPTANCE=NOT_EXECUTED`；`PERSISTENT_SSH_READY_CURRENT_BOOT=NOT_ISSUED`。
- 当前 `37ec99b...` 脚本是 Attempt2 历史字节，原样重跑会确定性再次在旧 no-upgrade 门停止；不得再执行。新候选必须修正升级行目标版本解析、精确四包模拟/安装、完整 dpkg delta 与残留标签后重新冻结 SHA。

未完成原因：APT 模拟要求升级既有 openssh-client，但用户最新指令禁止任何升级；等待零升级精确版本模拟证据，或等待新的精确升级授权

## D0C04 四包模拟与 Attempt3 授权（2026-08-14）

- 只读版本表：已装 `openssh-client=1:9.6p1-3ubuntu13.14`；源中 server/sftp 没有 `.14`，因此同版本零升级路线不可用。
- 候选模拟（用户转回文本）列出：唯一升级 `openssh-client .14 -> .18`；唯一新装 `openssh-server .18`、`openssh-sftp-server .18`、`libwrap0 7.6.q-33`；`0 to remove`、其余 `156 not upgraded`。最后的 `SIM_EXIT` 命令尚未回显，因此正式写入前必须由脚本重跑同一模拟并自行核 exit 0。
- 用户在收到上述完整四包清单、禁止其他 package 动作、失败不降级/卸载和允许保留 package/unit/host-key/apt-index 的说明后明确回复“批准”。该直接授权 supersede 历史 park 原因中“任何升级都禁止”的部分，但只限 client 这一个精确升级。
- D0C04 已通过 resume-only stage bridge 恢复为唯一 current leaf；D0C05 仍 parked。此时只完成 Harness 恢复与合同更新，`ATTEMPT3_EXECUTION=NOT_YET_EXECUTED`，新脚本 SHA=`NOT_YET_FROZEN`，`PERSISTENT_SSH_READY_CURRENT_BOOT=NOT_ISSUED`。
- Attempt3 为本次精确四包执行；任何重模拟漂移、版本不可用、第二个既有 package 变化、第五包、dpkg 异常或执行失败，先按获批口径收口并停止，不自动进行下一轮。

## D0C04 设备脚本 Attempt3 候选冻结（2026-08-14）

- 历史 `未完成原因` 已被本次精确四包批准解除，但只解除 `openssh-client .14 -> .18` 这一处升级门；它作为 Attempt2 的停放史实保留，不再代表当前授权状态。
- 当前候选仍为 `/private/tmp/syn-d0c04-retry-v2.ps1`；SHA-256：`bff8d7e51677f5f1be1ac6e9bf2264415188c1eed09d464cb49fe8a109c45abc`；`2312` 行、`95615` bytes。旧 `37ec99b...` 与 `7fc5b6a...` 候选均属于历史审查字节，禁止重放。
- 五段内嵌 Bash 重新独立通过 `bash -n`：Preflight `6612` bytes、Apply `33581` bytes、Rollback `17358` bytes、CommitCheck `9687` bytes、Finalize `1651` bytes。
- Windows PowerShell 到 WSL 的 payload 已改成内存 GZip + Base64；替换真实长度占位后，最大 Apply payload 的 Base64 为约 `9852` 字符，估算整条 native command 约 `10582 / 32767`，不再接近 Windows 命令行上限。正式写入前必须先取得同链路 `D0C04_COMPRESSED_LAUNCHER_SELFTEST=PASS`。
- 模拟必须自行取得 exit `0`，且 `Inst`/`Conf` 均恰好四行并逐字匹配唯一的一升三装；实际安装后和 COMMIT 前均以完整 dpkg 快照证明只有这四个 package 的精确状态/版本变化，且 `dpkg --audit` 为空。本轮不再执行 `apt-get update`。
- `openssh-client` 的全部 dpkg Conffiles 必须在预检、安装前、安装后和 COMMIT 保持 MD5 clean，且不存在 `.dpkg-*` 冲突文件；`/etc/ssh/ssh_config*` 与 `synadmin` SSH client 配置树按类型、owner/mode、文件 SHA-256、symlink target 冻结。任一漂移都在启用服务前停止、保留 recovery journal，并明确签 `LINUX_SSH_CLIENT_CONFIG_DRIFT=1`，不得冒充成功或自动恢复这些 package-owned bytes。
- 独立代码审查对上述候选给出静态 `GO`，但当前 Mac 没有 WinPS 5.1，且候选尚未在 5600X 执行；静态结果不能冒充设备运行通过。
- 当前 execution authorization id 为 `USER-SYN-D0C04-EXACT-FOUR-PACKAGE-ATTEMPT3-STAGE-12-20260814`，scope 精确为 `leaf:D0C04`；只包含 `14` 个 Attempt3 external targets（新增纯只读压缩 launcher 自检）和 `5` 个本轮对象 destructive rollback targets，不延伸到 D0C05。

### Attempt3 的 Mac 强制伴随协议

- 只有 Windows 输出 `D0C04_DEVICE_READY_FOR_MAC_ACCEPTANCE` 后，当前 Mac 主任务才可创建本轮 managed Host block 和专用 known_hosts；现有专用 key 必须标记 `MAC_KEY=RETAINED_AUTHORIZED`，不得重建、读取输出或删除。
- 任何 Mac 验收失败，必须先取得 `MAC_ROLLBACK_HOST_BLOCK=PASS_ABSENT_VERIFIED`、`MAC_ROLLBACK_KNOWN_HOSTS=PASS_ABSENT_VERIFIED`、`MAC_ROLLBACK_KEY=RETAINED_AUTHORIZED`，之后才允许用户在仍打开的管理员 PowerShell 中输入精确 `ROLLBACK <RunId>`。
- 即使用户已输入 `COMMIT <RunId>`，只要设备脚本仍未跨过 commit boundary，后续 Linux/Windows 提交前复核失败时，Mac 主任务仍必须执行上述 Mac rollback；设备脚本自身不负责修改 Mac。
- 若设备输出 `ROLLBACK=NOT_ATTEMPTED_ACCEPTED_CONFIG_PRESERVED` 和 `DEVICE_RESULT=COMMIT_CONFIG_ACCEPTED_JOURNAL_CLEANUP_FAILED`，说明 commit boundary 已跨越：Mac 与设备配置均保留，只把 journal cleanup 转人工复核，不得反向删除 Mac 配置。
- 成功必须同时绑定独立 Mac 两次全新公钥连接、host-key pin、无 key/password/root 负向回执，以及设备 `D0C04_DEVICE_COMMIT=PASS_CURRENT_BOOT`。脚本的 `MAC_ACCEPTANCE_EVIDENCE=REQUIRES_SEPARATE_MAC_RECEIPT` 不是验收结果。
- `ATTEMPT3_EXECUTION=NOT_YET_EXECUTED`；`PERSISTENT_SSH_READY_CURRENT_BOOT=NOT_ISSUED`；`PERSISTENT_SSH_RESTART_STABLE=NOT_TESTED`；D0C04 保持 current，D0C05 保持 parked。

## D0C04 设备脚本 Attempt3 执行回执（2026-08-14）

- 用户转回附件 SHA-256：`e698b33e8a7b5a25072ec9ad4819cfd1d29e599992612bfcb33a095a269b9c5f`；附件共 `126` 行、`6649` bytes。证据来源是用户转回的 5600X 脚本原始文本，不是本 Mac 任务直接取得的设备遥测，也不代表附件生成后的当前实时状态。
- 启动与事务前检通过：`D0C04_COMPRESSED_LAUNCHER_SELFTEST=PASS`、`LINUX_PREFLIGHT=PASS`、`D0C04_COLLISION_FREEZE=PASS`、`LINUX_APT_EXACT_FOUR_PACKAGE_PLAN=PASS`。本次没有再次执行 `apt-get update`。
- 获批的一升三装已经完成并通过全量 dpkg 差分：`openssh-client 1:9.6p1-3ubuntu13.14 -> 1:9.6p1-3ubuntu13.18`；新增 `openssh-server=1:9.6p1-3ubuntu13.18`、`openssh-sftp-server=1:9.6p1-3ubuntu13.18`、`libwrap0=7.6.q-33`；`LINUX_SSH_CLIENT_CONFIG_FREEZE=PASS`、`LINUX_SSH_CLIENT_CONFIG_DRIFT=0`、`LINUX_DPKG_AUDIT=PASS`、`LINUX_PACKAGE_DELTA=AUTHORIZED_COMPLETE_RETAINED`。
- 唯一明确失败点：四包事务完成后、SSH 服务启动前，`/usr/sbin/sshd -t` 返回 `Missing privilege separation directory: /run/sshd`。因此 `READY` 未到达，Windows loopback/rule/portproxy 和 Mac Host/known_hosts/SSH 验收均未执行。
- 依照用户已批准的失败残留合同，四包、package unit、host key 与默认 SSH package artifact 均保留，不执行降级、卸载或 purge；恢复锚点保留在 `/var/backups/syn-d0c04-06bee21f1b764dc780b0e1409a2c2651`。这些保留项是预期残留，不能写成整台 WSL 字节级回滚。
- 本轮 Syn 自定义配置退场回执为 `LINUX_ROLLBACK_CUSTOM_CONFIG=PASS_WITH_PACKAGE_OR_HOSTKEY_RESIDUAL`、`ROLLBACK_LINUX_CUSTOM_CONFIG=PASS_VERIFIED`；Windows 输出 `ROLLBACK_PORTPROXY=NOT_ATTEMPTED_PRESERVED`、`ROLLBACK_FIREWALL=NOT_ATTEMPTED_PRESERVED`，表示两项写入没有开始，不是创建后删除。最终标签为 `ROLLBACK_EXACT_CONFIG_AND_WINDOWS=PASS` 与 `DEVICE_RESULT=FAILED_WITH_PACKAGE_OR_HOSTKEY_RESIDUAL_REQUIRES_READONLY_RECHECK`。
- 结构化结论：`ATTEMPT3_RESULT=FAILED_AFTER_AUTHORIZED_PACKAGE_DELTA_BEFORE_SSH_SERVICE_START`；`PERSISTENT_SSH_READY_CURRENT_BOOT=NOT_ISSUED`；`MAC_SSH_ACCEPTANCE=NOT_EXECUTED`；`D0C05=NOT_AUTHORIZED_NOT_STARTED`。
- 冻结的 Attempt3 脚本 `bff8d7e5...` 仍要求安装前旧 package 基线，当前四包已保留，禁止原样重跑。下一轮必须先用新的 Attempt4 授权与专用 recovery 脚本只读复核四包、dpkg audit、SSH client 配置、host key、恢复锚点、自定义配置/service/socket/监听以及 Windows 端点；再单独冻结 `/run/sshd` 的 systemd RuntimeDirectory 或精确创建/回滚方案。不得自动开始 Attempt4。

未完成原因：Attempt3 已完成获批四包变更，但 sshd -t 因 /run/sshd 不存在而失败；四包、host key 与 recovery anchor 按合同保留，Syn 自定义配置已回滚，Windows rule/portproxy 未尝试；等待新的 Attempt4 残留复核与 runtime-dir 精确授权。

## D0C04 Attempt4 批准与恢复候选（2026-08-14）

- 上述历史停放原因已经由用户对完整 Attempt4 范围的直接“批准”解除，但只解除 retained-state recovery 与 systemd RuntimeDirectory 启动顺序；不恢复任何 package、重启、D0C05、源码、Git 或产品权限。
- Attempt4 新脚本固定为 `/private/tmp/syn-d0c04-attempt4-runtime-dir-recovery.ps1`；冻结 SHA-256 为 `d5ebfe83b15e9db1f5fa718edc4814d2fd91b6e3b9a2bfa224a35935a373790b`，共 `1717` 行、`67613` bytes。任何不同 SHA 的副本都不是本轮候选，禁止执行。
- 五段合成 Bash 已分别通过 `bash -n`，原始长度为 Preflight `36154`、Apply `40681`、Rollback `35361`、CommitCheck `35675`、Finalize `35550` bytes；对应 GZip+Base64 长度为 `10144 / 11112 / 9788 / 9900 / 9848`，低于 Windows 命令行上限。
- PowerShell here-string/自定义分隔符结构检查通过；静态扫描未发现 apt/package 写入、`ssh-keygen -A`、旧脚本重放、Attempt5、`reset-failed` 或手工修改 `/run/sshd`。当前 Mac 没有 WinPS 5.1，因此这些仍是静态证据，不能冒充 5600X 实际解析或执行通过。
- 两名独立审查者都对上述同一 SHA 给出 `GO`，共同结论为 `P0=0 / P1=0`；该 GO 只允许进入既有精确授权下的设备脚本，成功仍必须绑定独立 Mac 验收。`ATTEMPT4_EXECUTION=READONLY_PREFLIGHT_FAILED_NO_MUTATION`。
- 旧 Attempt3 脚本 `/private/tmp/syn-d0c04-retry-v2.ps1` 与旧 anchor 只读封存，禁止重放、修改或清理。新 recovery journal 必须使用新的 run id，并与旧 anchor 完全分离。
- 预检必须签发 `D0C04_RECOVERY_PREFLIGHT=PASS`、四包/`dpkg --audit`/client config/host key/旧 anchor精确通过、Syn/Windows/Mac 未接受对象无碰撞、unit RuntimeDirectory 合同通过后，才允许任何 Attempt4 写入。
- 启动必须由 systemd 创建和管理 `/run/sshd`；unit 不精确或目录前态不明时签 `RECOVERY_BLOCKED_READONLY_STATE_DRIFT_NO_MUTATION` 并停止，不得手工 mkdir/chown/chmod/rm。
- 成功仍需设备 READY、独立 Mac host-key 与两次全新公钥连接/负向验收、设备 COMMIT 全部齐全，才可签 `PERSISTENT_SSH_READY_CURRENT_BOOT`；`PERSISTENT_SSH_RESTART_STABLE=NOT_TESTED`，D0C05 保持 parked。

## D0C04 Attempt4 只读预检执行回执（2026-08-14）

- 证据来源：用户在聊天中转回冻结脚本 `d5ebfe83...` 的原始输出；没有独立附件 hash，也不是本 Mac 直接控制 5600X 的终端遥测。
- Windows 写入前态均通过：`WINDOWS_RULE_PRESTATE=ABSENT`、`WINDOWS_PORTPROXY_PRESTATE=ABSENT`、`WINDOWS_47123_LISTENER_PRESTATE=ABSENT`；压缩 launcher 自检通过。
- WSL 在第一道只读总门签发 `D0C04_RECOVERY_PREFLIGHT=FAIL reason=retained_state_unit_or_collision_drift` 并以 exit `20` 停止；`D0C04_FAILURE_PHASE=READONLY_PREFLIGHT`，没有进入 Linux apply。
- 退场输出为 `ROLLBACK_PORTPROXY=NOT_ATTEMPTED_PRESERVED`、`ROLLBACK_FIREWALL=NOT_ATTEMPTED_PRESERVED`、`ROLLBACK_LINUX_CUSTOM_CONFIG=NOT_NEEDED_PRE_APPLY`、`ROLLBACK_EXACT_CONFIG_AND_WINDOWS=NOT_NEEDED_READONLY_STOP`，最终 `DEVICE_RESULT=RECOVERY_BLOCKED_READONLY_NO_MUTATION`。Mac 未接受配置也未进入写入阶段。
- 该汇总门包含 retained package/dpkg/client config/host key/旧 anchor、unit、service/socket、监听、`/run/sshd` 与 Syn 临时对象等多项断言；当前回执不足以锁定唯一失败子项。禁止原样重跑；下一步只做逐项零写入诊断，任何修正或新写入脚本都需要新的直接批准。

未完成原因：用户已明确批准先执行 D0D01 源码迁移；D0C04 保持未完成，本轮不执行，迁移完成后也不自动恢复
