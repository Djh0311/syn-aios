# 阶段12 Syn 5600X/WSL C2 长期 SSH 开发通道与 D 源码迁移

总计划：product-line 唯一基线与 Harness Lite 切换
目标：保留 C2 长期 SSH 通道工作；D0D01 已按 Attempt6 完成并结算。D0C04、D0C05 保持 unfinished 且本轮不执行；stage-12 仍开着。D0D01 仅用现有 SSH 通道把 main 完整可达历史和获批 WIP 搬到 WSL，不把普通 SSH 或源码迁移冒充 Syn Edge/Core、Headless 或 Primary。

用户的新决定：用户在 C1 成功后明确要求建立长期通道；该指令只覆盖迁移计划 v1 中 NAT `portproxy` “仅临时”的限定，允许为本开发 SSH 入口建立唯一命名、精确范围、可回滚的持久 rule/portproxy。迁移计划其余 DRAFT 状态、分项授权、源码迁移和产品架构边界均不改变。

用户的精确 package 决定：Attempt2 因候选安装需要升级既有 `openssh-client` 而按旧 no-upgrade 合同停止。用户随后在收到完整四包计划与不可逆残留说明后明确回复“批准”，仅 supersede 该旧合同中的一个点：允许 `openssh-client:amd64` 从 `1:9.6p1-3ubuntu13.14` 升至 `1:9.6p1-3ubuntu13.18`；同时只允许新增 `openssh-server:amd64=1:9.6p1-3ubuntu13.18`、`openssh-sftp-server:amd64=1:9.6p1-3ubuntu13.18` 和 `libwrap0:amd64=7.6.q-33`。其余 package 与边界不变。

用户的 Attempt4 决定：Attempt3 已完成上述一升三装并通过精确 dpkg 差分，但在 SSH 服务启动前因 `/run/sshd` 尚不存在而停止。用户在收到失败回执、保留项、旧脚本不可重放和最小恢复范围后明确回复“批准”。Attempt4 把现有四包与 host key 作为不可变基线，不再运行 apt 或任何 package mutation；只允许先只读复核残留，再验证 package `ssh.service` 的 `RuntimeDirectory=sshd`、`RuntimeDirectoryMode=0755`，由 systemd 管理 `/run/sshd` 并继续原 D0C04 精确通道配置。旧 recovery anchor 全程只读保留，D0C05 仍未授权。

用户的 D0D01 决定：用户在收到当前 Harness 不匹配、main-only refs 边界及已提交合成 .env fixture 说明后明确回复“批准”。本轮允许把 Git bundle + 独立 WIP manifest/双 hash 合同落入 Harness 并执行；只迁 refs/heads/main 的 784 个可达提交和冻结后的 tracked/untracked WIP。允许完整历史携带 27 字节合成测试 fixture；真实 .env、密钥、凭据、活动 SQLite、构建产物、其他 heads、remote-tracking 私有实验、refs/stash 与 dangling objects 均不作为迁移输入。目标碰撞、源漂移、断传或任一校验不一致立即停止，不覆盖、不重试写入、不自动删除失败候选。

用户的 D0D01 Attempt4 决定：Attempt3 的 19 个传输文件 SHA-256 全部一致，但 WSL GNU tar 原始路径视图发现 untracked 归档为 26 项而冻结清单为 15 项；额外 11 项均是 macOS copyfile/bsdtar 为带 `com.apple.provenance` xattr 的源文件生成的 `._*` AppleDouble 元数据。目标仍为空且未 clone/应用 WIP。用户收到停止回执和全新 run 合同后再次明确回复“批准”；该新鲜批准只解除“不得自动重试”到一个全新 Attempt4，不允许重放/改写/清理 Attempt3，也不自动授权 Attempt5。Attempt4 固定新 run-id `20260816T012913`，旧本地/远端候选只读保留；新归档必须同时禁用 AppleDouble、xattr、ACL 和 file flags，并以 Python tarfile 原始成员视图与 WSL GNU tar 独立验证成员严格等于冻结清单，不能再把 macOS `tar -tf` 作为唯一真值。

用户的 D0D01 Attempt5 决定：Attempt4 在首次远端写入前因生成的本地 build 脚本把补丁续行落成字面 `+` 参数而停止；首个失败点是 tar argv，未生成 tar、bundle 或 payload，远端新 staging 未创建，目标未写。用户随后明确回复“继续”，并澄清硬停止只适用于目标/远端 staging 碰撞、传输开始后的中断、以及冻结候选后的 HEAD/ref/hash/tar/manifest/fsck/清单不一致。Attempt5 固定 run-id `20260816T015545`；先在独立 lint-only 根内修正并验证脚本，不得在那里生成任何迁移物或 SSH；lint PASS 后只把冻结 SHA 的相同脚本字节带入唯一正式本地根并全量重建。首次远端写入后任何流程异常按传输中断硬停止。Attempt1 至 Attempt4 保持只读，D0C04/D0C05 仍不执行。

用户的 D0D01 Attempt6 决定：Attempt5 的 bundle、WIP 归档与本地恢复内容均正确，但正式本地 root 的全局 `umask 077` 把新 checkout/extract 文件系统权限系统性变成 `600/700`，导致 3581 项 fs_mode 与源冻结不一致；路径、类型、Git mode、大小和 SHA-256 均零差异，且未创建 Attempt5 远端 staging。用户收到回执后明确要求“别停直接做到迁移完成”。本决定授权 Attempt6 从 run-id `20260816T023204` 开始持续推进到 SOURCE_BYTES_MATCH：保留所有旧候选、不覆盖已有路径；本地工具错误或候选校验失败时改用同一 Attempt6 根下全新单调编号子候选重新全量构建；传输中断时保留失败远端子候选并换全新单调编号子候选，不在同一路径重传。目标若不再为空则仍禁止覆盖，只能只读判明后报告阻断。D0C04/D0C05 和产品运行边界不变。

编号说明：这里的 Harness `stage-12` 是开发护栏编号，不是产品 M12，也不激活 M5–M10。

干完的标准：

- D0C04 完成持久 SSH 配置并在当前启动周期内反复连接成功：Mac 专用 key 固定为 `/Users/yoyi/.ssh/syn_5600x_wsl_ed25519`，Host 固定为 `syn-5600x-wsl`；WSL drop-in 固定为 `/etc/ssh/sshd_config.d/90-syn-development.conf`，授权文件固定为 `/home/synadmin/.ssh/authorized_keys`；固定 Windows Tailscale 入口、唯一持久防火墙规则和唯一持久 portproxy 均有精确回滚。
- Attempt4 开始前必须现场证明现有四包均为 `install ok installed` 且版本精确为 client/server/sftp `.18` 与 `libwrap0 7.6.q-33`，`dpkg --audit` 为空，SSH client 配置、conffile、host key 与只读旧 anchor 均未漂移；不得再运行 apt、安装、升级、重装、降级、删除、配置或触发任何 package 动作。
- 优先验证 Windows `127.0.0.1:22` 能通过 WSL localhost forwarding 到达 WSL sshd；只有该门通过，才允许建立 `100.98.94.76:47123 -> 127.0.0.1:22`。若不通立即停止，不自动硬编码会漂移的 WSL IPv4。
- D0C05 在另行授权的维护窗口中验证 WSL/Windows 重启或重新登录后的自动恢复；未完成前只能签 `PERSISTENT_SSH_READY_CURRENT_BOOT`，不能签 `PERSISTENT_SSH_RESTART_STABLE`。
- D0D01 以 main@9103c3b26b060e854be119a8cedaa856a2a900ce 为提交基线，使用具名 main-only bundle 与独立 WIP patch/untracked archive；两端通过 bundle SHA-256、HEAD/tree、允许 refs、git fsck --full --strict、WIP 状态/路径/模式/大小/SHA-256 和排除项校验后，只签 SOURCE_BYTES_MATCH。
- sshd 有效值必须经 `sshd -t` 与 `sshd -T -C user=synadmin,host=localhost,addr=127.0.0.1` 证明：`ListenAddress 127.0.0.1`、`AllowUsers synadmin`、`PubkeyAuthentication yes`、`AuthenticationMethods publickey`、`PasswordAuthentication no`、`KbdInteractiveAuthentication no`、`PermitRootLogin no`。
- `ssh.service` 必须来自已安装 package、没有未知 drop-in，并精确声明 `RuntimeDirectory=sshd`、`RuntimeDirectoryMode=0755`。只有该门通过才允许由 systemd 启动服务并管理 `/run/sshd`；禁止手工创建、持久化、改属主/权限或删除该目录。unit 不匹配或 runtime dir 来源不明时立即停止。
- host key 指纹先由已信任的 Windows/WSL 本地执行回执给出；Mac `ssh-keyscan` 结果只有指纹一致后才写专用 `/Users/yoyi/.ssh/known_hosts_syn_5600x_wsl`。正确 key 两次全新连接必须成功；无 key/密码路径与 root 登录必须失败。
- 不修改既有 `Tailscale-In`、Cockpit、`19528` 或其他现有防火墙对象；不开放公网或 Windows/Tailnet 的 `0.0.0.0` 对外入口。WSL sshd 也固定只监听 Linux `127.0.0.1:22`；若 Windows localhost forwarding 不能取得 OpenSSH banner，立即停止，不放宽监听。
- D0C04/D0C05 仍只处理开发运维入口；D0D01 只迁源码字节，不安装开发依赖、不运行产品、不实现 Headless Core、不切 Primary/epoch。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-12.md
- docs/harness/leaves/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md
- docs/harness/unfinished/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md [新增]
- docs/harness/unfinished/D0C05-syn-5600x-wsl-persistent-ssh-restart-validation.md [新增]
- docs/harness/leaves/D0D01-syn-full-source-migration-to-5600x-wsl.md [新增]
- docs/harness/unfinished/D0D01-syn-full-source-migration-to-5600x-wsl.md [新增]
- docs/harness/done/2026-08/D0D01-syn-full-source-migration-to-5600x-wsl.md [新增]
- docs/harness/done/2026-08/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md [新增]
- docs/harness/done/2026-08/D0C05-syn-5600x-wsl-persistent-ssh-restart-validation.md [新增]
- docs/harness/done/2026-08/stage-12.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

Attempt4 设备写入候选（必须取得精确 external 授权后执行）：

- WSL 只读验证 Attempt3 保留的四包、host key、package unit、SSH client 配置与旧 anchor `/var/backups/syn-d0c04-06bee21f1b764dc780b0e1409a2c2651`；旧 anchor 不复用、不修改、不清理。Attempt4 使用新的 run-id recovery journal，只记录这轮自定义对象的前态与所有权。
- 在 `RuntimeDirectory` 门通过后，建立 `/etc/ssh/sshd_config.d/90-syn-development.conf`，仅向 `/home/synadmin/.ssh/authorized_keys` 追加本轮专用公钥，先冻结并保持 `ssh.socket` disabled/inactive，再由 systemd enable/start `ssh.service`。服务自己的 RuntimeDirectory 与 ExecStartPre 负责运行目录和语法前检；启动后再独立执行 `sshd -t`、`sshd -T -C`、service/socket 和唯一 `127.0.0.1:22` 后验。不得编辑 unit 或其他 systemd unit。
- Attempt4 全程禁止 `apt`、`apt-get`、package 安装/升级/重装/降级/删除/purge/autoremove/configure/trigger、host key 生成/替换/删除、source/index/hold/pin 变更。任何 package、dpkg audit、conffile、host key 或旧 anchor 漂移都在创建新 journal 前停止。
- Mac 复用已存在的 `/Users/yoyi/.ssh/syn_5600x_wsl_ed25519` 专用 Ed25519 keypair，只允许标准 SSH 认证 act；不得创建、重建、修改或删除 keypair。仅在 READY 后创建或修改 `Host syn-5600x-wsl` managed block 和对应专用 known_hosts。模型不得查看、输出、复制或传出私钥，权限必须保持 `0600`。
- Windows 新增唯一持久 portproxy `100.98.94.76:47123 -> 127.0.0.1:22`，并新增唯一持久规则 `Syn-WSL-SSH-47123`：Direction=Inbound、Action=Allow、Enabled=True、Profile=Any、LocalAddress=`100.98.94.76`、RemoteAddress=`100.120.223.16`、Protocol=TCP、LocalPort=`47123`。
- Mac 用 `syn-5600x-wsl` 做公钥登录、身份和重复连接验收。
- D0C05 的 WSL/Windows 重启、登录触发器或计划任务必须另行授权，不从 D0C04 自动推导。

不许动：

- 既有 Tailscale、ACL、`Tailscale-In`、Cockpit、`19528` 或其他防火墙规则
- Windows 更新、安装 Windows OpenSSH、WSL 独立 Tailscale、mirrored networking、动态 WSL-IP portproxy、计划任务（D0C04）
- 密码、私钥内容、令牌、凭据或非本轮 public key
- D0D01 之外的源码写入；Git add/commit/push/merge/rebase/reset/clean/stash；非 main refs、refs/stash 或 dangling objects 迁移
- 产品代码、运行数据、活动 SQLite、M5–M10、Headless Core、Primary/epoch、部署或发布
- 删除发行版、用户目录、已有 SSH 配置或既有密钥

停止与回滚：

- Windows loopback 不能稳定到达 WSL sshd、端口冲突、固定名对象已存在、目标设备/用户/IP 不匹配或需要扩大范围时立即停止。
- 失败时只删除 Attempt4 新建的防火墙规则和 portproxy、撤回本轮 authorized_keys 行、drop-in、new recovery journal/tmp 与未接受的 Mac SSH 配置，并确保 `ssh.service`/`ssh.socket` 不留监听；systemd 管理的 `/run/sshd` 只做后态核验，不手工删除。四包、package unit、host key、既有密钥、发行版和 Attempt3 旧 anchor 均保留且不得改写，不能冒充完整 WSL 字节回滚。
- 失败后若四包 delta 精确完成，签 `PACKAGE_DELTA=AUTHORIZED_COMPLETE_RETAINED`；若部分安装、版本不符、出现第五包或 `dpkg --audit` 非空，签 `PACKAGE_DELTA=PARTIAL_OR_UNEXPECTED` 并保留 transaction journal，转人工只读复核。只能分别签 custom config/Windows rollback，禁止输出全局 WSL rollback PASS。
- D0C04 成功后不自动执行 D0C05；等待用户选择维护窗口和重启授权。

## 叶子

- [ ] D0C04 长期 SSH 开发通道配置与当前启动周期验收
- [ ] D0C05 重启后自动恢复与长期稳定验收
- [x] D0D01 完整 main 源码与当前 WIP 迁移到 5600X WSL（Attempt6 SOURCE_BYTES_MATCH，已结算）
