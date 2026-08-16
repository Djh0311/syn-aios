# Syn 剩余开发迁移到 5600X Windows/WSL2 + Tailscale 计划 v1

日期：2026-08-13

状态：**DRAFT / USER-APPROVED PLANNING INPUT / NOT_EXECUTION_AUTHORITY**

当前范围：只冻结迁移顺序、验收门、停止点和回滚；每个执行阶段仍需新的当前指令与匹配 Harness 授权。

架构关系：详细目标见 [`CAND-013 架构候选`](../product/syn-primary-edge-core-distributed-runtime-architecture-candidate-v2.md)。开发迁移不等于候选转正、Headless 实现或 Primary 切换。

## 0. 大白话结论

先把“以后能继续开发的源码和工具环境”安全搬到 WSL，再继续 Syn 剩余工作。整个过程分成七道门：

1. 文档边界说清楚；
2. 两台机器只读体检；
3. 可逆地配置 WSL 和网络；
4. 用 Git bundle 搬完整源码；
5. 在 WSL 从零重建开发环境；
6. 以后另做 Headless Core/Edge；
7. 最后再单独迁正式数据和切 Primary。

前一道门通过，只代表前一道门通过：

```text
设备可达
  ≠ 源码完整
  ≠ WSL 可开发
  ≠ Headless Core 已实现
  ≠ 5600X 已成为正式 Primary
```

建议使用五个独立验收标签：

| 标签 | 只证明什么 | 明确不证明什么 |
|---|---|---|
| `DEVICE_PRECHECK_PASS` | 设备、WSL、磁盘和网络方案已查清 | WSL 已配置、源码已迁移 |
| `SOURCE_BYTES_MATCH` | Mac 与 WSL 的 Git HEAD/tree/对象一致 | 依赖可安装、项目可运行 |
| `WSL_DEV_READY` | 当前提交可在 WSL 继续开发 | Headless Core 已实现 |
| `HEADLESS_CANDIDATE_READY` | 独立 Core 候选达到具名故障验收门 | 正式 Primary 已切换 |
| `PRIMARY_SWITCH_ACCEPTED` | 新节点已成为唯一合法写者并有切换 receipt | 发布、高可用或长期生产已自动成立 |

## 1. 当前基线与证据上限

### 1.1 当前本机已核实

- 仓库：`/Users/yoyi/workspace/product-line-syn-integration-main`
- 当前基线：`main@9103c3b26b060e854be119a8cedaa856a2a900ce`
- 当前 tree：`4080362684d53e6ccc845de87827ec722490efae`
- 相对本机现有 `origin/main` tracking ref：`0 behind / 117 ahead`；本轮没有联网刷新，不能写成远端服务器的当前事实。
- 当前 checkout 是 linked worktree；`.git` 指向 Mac 绝对 gitdir，不能整目录 rsync 到 WSL。
- `prototypes/productized-desktop-shell/src-tauri/target` 约 51.36 GiB；`node_modules` 约 87 MiB；`dist` 约 1.6 MiB，均应在 WSL 重建。
- 产品 Rust crate 仍直接依赖 Tauri；仓内没有独立 `syn-core-lib` 或 Headless Primary service。
- M1–M4 已完成；M4R01–M4R07 与 `stage-07` 已关闭；M5–M10 为 `PLANNED / NOT_ACTIVE`。

上述 HEAD、tree、体量和 tracking-ref 差异都是 2026-08-13 的本机快照。真正执行 D/E 前必须重新冻结，不得盲用旧值。

### 1.2 尚未验证

- Mac 与 5600X 两端 Tailscale 的安装、登录、tailnet、设备身份、ACL 和端口可达；
- 5600X 的 Windows、虚拟化、WSL 发行版、资源、磁盘、GPU、systemd 和网络模式；
- Windows OpenSSH、防火墙、NAT、mirrored networking、`portproxy` 或 WSL 独立 Tailscale 的实际状态；
- 5600X 是否适合常驻运行、Primary 数据盘与备份链是否可恢复；
- Linux/WSL 下全部依赖、Tauri 构建、macOS 专属代码和现有回归是否通过。

## 2. 全阶段权限表

| 阶段 | 目的 | 主要操作者 | 写磁盘 | 联网/连接设备 | 需要的独立授权 |
|---|---|---|---:|---:|---|
| A | D0 文档与权威收口 | 我起草、核对；用户批准 | 是 | 否 | Harness control/context、文档写入；Git 另算 |
| B | 5600X/WSL/Tailscale 只读预检 | 用户开机解锁；我做批准后的只读检查 | 原则上无主动配置写入 | 是 | 连接 5600X、Tailscale 探测、系统只读检查 |
| C | 可逆设备配置 | 我按清单执行；用户处理管理员/GUI确认 | 是 | 是 | 安装、管理员权限、SSH、防火墙、端口、重启分别授权 |
| D | Git 完整源码迁移 | 我冻结、打包、传输、克隆和验收 | 是 | 是 | 创建 bundle、传输、WSL 落盘；不含 push |
| E | WSL 开发环境重建 | 我安装依赖和运行验证 | 是 | 通常是 | 软件安装、依赖下载、构建和测试；代码修改另算 |
| F | Headless Core/Edge 实现 | 后续独立开发 stage | 是 | 按测试分层 | 合同、产品代码、测试和 Git 分别授权；M5–M10 不自动激活 |
| G | 正式 Primary/数据/epoch 切换 | 用户临场批准；我按手册执行 | 是 | 是 | 生产级数据、停写、fencing、切换、观察和回退授权 |

## 3. A — 文档与权威 D0 收口

**谁操作：** 用户批准范围；我负责起草、机械核对和 Harness 生命周期。

**磁盘/联网/授权：** 写仓库文档，不联网；需要 `context/control` 和精确文档写域；Git add/commit/push 不由本阶段自动获得。

### 具体动作

- 把外部微信临时稿接成仓内 `CANDIDATE / DRAFT`，保存来源路径和 SHA-256。
- 修复四项已知冲突：
  1. 时间线改为“当前 Mac 内嵌 → 只迁开发环境 → Headless 候选 → 5600X 显式切换 → 未来 EPYC”；
  2. 补上 Mac UI 连接 5600X Interim Primary 的过渡期；
  3. 删除不存在的 `docs/harness/CURRENT.md`、`AUTHORITY.md` 现行引用；
  4. 状态改为 M1–M4 完成、M5–M10 未激活。
- 在唯一候选登记建立 `CAND-013`，不直接改现行架构正本。
- 把设备配置和源码迁移拆到本计划，不让架构图冒充执行步骤。

### 成功标准

- 候选、候选登记和迁移计划互相引用且状态一致。
- 当前事实、用户陈述、目标候选和未验证项分开。
- 明确不重做 M4、不激活 M5–M10、不把迁移升级成 Headless/Primary 完成。
- 明确 Harness `stage-08` 只是 D0 文档收口编号，不是产品 M8 connector 阶段。
- Harness stage/leaf 在验证后归档，但不产生 Git 提交。

### 失败停止点

- 文档仍把 WSL 构建、Tailscale ping、用户陈述或目标拓扑写成已实现事实；
- 必须修改现行产品/架构正本才能完成最小候选登记；
- 写域扩大到产品代码、当前状态、M1–M4 或历史 receipt。

### 回滚

- 未提交时保留明确 diff，由用户决定是否撤销；不使用 reset/clean/stash。
- 若未来已有提交，只用新的反向提交，不改写历史。

## 4. B — 5600X、WSL 与 Tailscale 只读预检

**谁操作：** 用户只负责开机、解锁和必要登录；我在获得远端只读授权后完成检查。

**磁盘/联网/授权：** 不主动改配置，但 Windows、SSH、Tailscale 可能留下普通系统日志；需要联网、连接设备和读取系统状态的单独授权。

### 具体动作

1. 核对唯一目标设备：Windows 版本、主机名/设备标识、CPU、内存、GPU、虚拟化。
2. 核对 WSL：版本、发行版、WSL1/2、systemd 能力、默认用户、VHDX 位置和当前状态。
3. 核对容量：Windows 各盘、WSL Linux 文件系统和目标工作目录的可用空间。
4. 核对 Tailscale：两端登录状态、tailnet、设备身份、重复节点、地址和 ACL；不在聊天中输出密钥或登录令牌。
5. 核对网络：WSL NAT/mirrored、已有 `portproxy`、Hyper-V/Defender 防火墙、Windows OpenSSH、监听端口和绑定接口。
6. 核对仓库搬运前置：Git LFS、submodule、私有依赖、tracked `.env` 测试夹具和其他运行必需但被忽略的文件。
7. 冻结目标目录在 WSL Linux 文件系统，例如 `~/src/product-line-syn-integration-main`；不把 `/mnt/c` 作为长期 Rust 构建目录。

### 网络证据分层

必须区分三层，不能互相替代：

1. Mac 能识别或 ping 到 Windows 的 Tailscale 节点；
2. Windows 宿主的指定临时端口能通过 Tailscale 到达；
3. Mac 经 Windows Tailscale 地址访问到 WSL 内的临时服务端口。

第 1 层只证明 Windows 宿主可达；不证明 WSL 端口、Syn 服务或业务授权成立。B 只盘点现有状态：只有现场已经存在且明确获准的安全测试服务时，才做第 2/3 层只读探测；否则记录 `NOT_TESTED / DEFERRED_TO_C`。B 不为了验证而启动服务、开放端口、改转发或改防火墙。

### 成功标准

- 唯一设备、发行版、磁盘余量和目标目录已确定。
- 在 NAT、mirrored、NAT + 受限 `portproxy` 或 WSL 独立 Tailscale 中选出一个可解释方案。
- 已有监听和暴露面已查清，没有为 B 新增公网或非批准网络暴露。
- 形成聊天内 `DEVICE_PRECHECK_PASS` 只读回执和进入 C 的精确配置清单；如需仓内证据文件，另获本地写入授权。
- 第 2/3 层若无现成安全测试服务，可以保持 `NOT_TESTED / DEFERRED_TO_C`，不冒充已通过。

### 失败停止点

- Tailscale 账号、tailnet 或设备身份不确定；
- WSL/虚拟化异常或目标盘空间不足；
- 建议最低先保留约 120 GiB 可用构建空间，低于此值先重新测算，不直接安装；
- 防火墙或转发必须开放到所有网络；
- LFS、submodule、私有依赖或 tracked `.env` 性质不清。

### 回滚

本阶段原则上没有主动配置变更，结束只读会话即可。系统自动日志如实记录为环境副作用，不冒充绝对零写入。

## 5. C — 可逆设备配置

**谁操作：** 我准备精确清单并在授权后执行；用户只处理 Windows 管理员确认、GUI 登录或必须由人完成的重启。

**磁盘/联网/授权：** 会修改 Windows/WSL 配置并联网；WSL 安装、资源配置、SSH、防火墙、端口转发、Docker、计划任务和重启分别授权。

### 推荐起步原则

- 默认先让 Tailscale 运行在 Windows 宿主。
- 是否让 WSL 自己成为第二个 Tailscale 节点，由 B 的入站稳定性和 ACL 需求决定；不为“看起来完整”制造双设备身份。
- Windows 本机 VS Code 使用 Remote-WSL；Mac 访问 WSL 使用 SSH/Remote-SSH。这是两条不同链路，分别验收。
- 此阶段只验证 systemd 能力，不创建尚不存在的 Syn Primary service。

### 具体动作

1. 安装或启用批准的 WSL2 发行版，建立普通开发用户。
2. 按 B 的资源事实配置 `.wslconfig`、`wsl.conf` 和 systemd；先保存原值。
3. 只安装迁移前必需的基础管理能力，不提前搬 Docker、ERP、模型或正式数据。
4. 按选定网络模式处理：
   - mirrored：单独核 Windows/Hyper-V 防火墙和监听范围；
   - NAT：仅对批准端口建立精确、临时、可撤销的 `portproxy`，并记录 WSL IP 可能在重启后变化；
   - WSL 独立 Tailscale：作为单独设备登记，单独核 ACL、撤权和密钥边界。
5. 启动无敏感数据的临时 HTTP/SSH 服务，验证 Mac → Windows Tailscale 地址 → WSL 端口。
6. 若用户另行批准重启，再重复端到端验证，区分“一次可达”和“重启后稳定”。

### 成功标准

- Mac 访问到的确实是 WSL 临时服务，不只是 Windows 宿主。
- 端口只绑定批准接口和来源，公网扫描面没有扩大。
- 原配置、变更项、撤销命令和重启后结果都有记录。

### 失败停止点

- 需要开放 `0.0.0.0`/所有网络且无法用防火墙收窄；
- `portproxy` 目标不稳定或指向错误 WSL IP；
- Windows 与 WSL 双重防火墙边界无法解释；
- 管理员命令目标、发行版或用户不明确；
- 配置过程要求删除发行版、迁移正式数据或改变其他项目服务。

### 回滚

- 删除本阶段新增的精确防火墙规则、端口转发和临时服务。
- 恢复备份的 `.wslconfig`、`wsl.conf` 和 SSH 配置；必要时执行获批的 `wsl --shutdown`。
- 不自动注销发行版、删除用户目录或清理磁盘；这些是新的破坏性授权。

## 6. D — Git 完整源码迁移与双端校验

**谁操作：** 我重新冻结基线、创建 bundle、校验、传输、克隆和双端验收；用户原则上只批准传输。

**磁盘/联网/授权：** Mac 创建 bundle、WSL 创建新 clone；经批准的 Tailscale/SSH 通道联网；不包含 push、远端凭据或删除旧仓库。

### 为什么不用 rsync 整个目录

- 当前 `.git` 是 linked-worktree 指针，指向 Mac 上另一个绝对 gitdir；搬到 WSL 后会失效。
- 普通远端 clone 只能得到远端已有对象，当前本机相对 tracking ref 多出的 117 个提交不能假定已在远端。
- `target`、`node_modules` 和 `dist` 是平台相关生成物，搬过去既浪费空间又污染验证。

### 未来执行顺序

1. Mac 重新核对 HEAD、tree、分支、工作树/index、现有 dirty 字节、submodule、LFS 和 tracked `.env`。
2. **先过 clean-baseline 硬门：** D0 收口字节及所有需要迁移的源码/文档必须先在独立授权下形成具名本地提交，工作树/index 必须 clean，再冻结新的 HEAD/tree。A 阶段不提供这项 Git 授权。
3. 若用户决定保留 dirty/untracked 状态而不提交，则 D 停下，另建“Git bundle + WIP manifest/双 hash/恢复校验”合同；在该合同获批前不得迁移或签发 `SOURCE_BYTES_MATCH`。
4. 若 HEAD 不再是本计划快照，重新冻结新基线；不强行把旧 hash 当目标。
5. 仅为新冻结 `main` 的完整可达历史创建 Git bundle；不无差别携带可能含私密实验的所有 refs。
6. 在 Mac 运行 bundle 自检并计算 SHA-256，限制 bundle 文件权限。
7. 通过批准的加密通道自动传输，减少用户手工拖拽。
8. WSL 先核 bundle SHA-256，并用 `git bundle list-heads` 观察预期 refs。
9. 从 bundle 克隆到 WSL Linux 文件系统中的全新空目录。
10. 在新 clone 的仓库上下文中运行 `git bundle verify <bundle>` 与 `git fsck --full`。
11. 双端比较 HEAD、`HEAD^{tree}`、分支、clean 状态和固定格式树清单。
12. 远端配置和凭据以后单独处理；不 push，也不把“已复制”写成“远端已同步”。

计划中的命令形态示例仅用于未来命令单审查，本阶段不执行：

```text
Mac: git bundle create <受保护临时路径>/syn-main-<HEAD>.bundle main
Mac: git bundle verify <bundle>
Mac: shasum -a 256 <bundle>
WSL: sha256sum <bundle>
WSL: git bundle list-heads <bundle>
WSL: git clone <bundle> <全新目标目录>
WSL clone 内: git bundle verify <bundle>
WSL: git fsck --full
```

### 明确排除

- `target`、`node_modules`、`dist` 和其他构建缓存；
- 本机真实 `.env`、凭据、私钥、登录态和未登记工具缓存；
- 活动 SQLite、运行时数据目录和正在使用的数据库文件；
- Mac linked-worktree 的 `.git` 指针；
- 未经分类的 untracked/ignored 文件。

Git bundle 会携带全部已提交字节。当前已发现一个 tracked `.env` 测试夹具路径；执行前必须确认其为合成测试素材。性质不清时停止，不能通过偷偷改历史或过滤 tree 来“解决”。

### 成功标准

- 两端 bundle SHA-256、HEAD 和 tree ID 完全一致。
- `git bundle verify` 与 `git fsck --full` 通过。
- WSL clone clean，没有依赖 Mac 绝对 `.git` 路径。
- 所有计划迁移的 D0 权威字节都已进入新冻结提交，或另有明确获批的 WIP 搬运合同；不能默默漏掉 dirty/untracked 文件。
- 形成 `SOURCE_BYTES_MATCH` 收据；Mac 原仓库保持不动。

### 失败停止点

- 任一 hash、HEAD、tree 或对象校验不一致；
- Mac 在冻结前仍有需要迁移的 dirty/untracked 字节，且没有具名本地提交或获批 WIP 搬运合同；
- Mac 在冻结后出现新改动；
- bundle 缺少必要对象、LFS/submodule 或私有依赖；
- 目标目录非空、存在不明文件或位于不适合长期构建的挂载盘；
- 传输要求暴露凭据、开放公网或绕过已批准通道。

### 回滚

- Mac 原仓库和分支头不动，继续作为回退基线。
- WSL 候选标记为失败并停止；不 reset、不覆盖目标目录。
- 是否删除失败 clone 或 bundle 另获删除授权；默认保留到 E 验收完成。

## 7. E — WSL 开发环境重建与可运行验证

**谁操作：** 我根据锁文件安装依赖并分层验证；用户只处理必要管理员提示。

**磁盘/联网/授权：** 会下载和安装软件、生成 `target/node_modules/dist` 并运行测试；不默认授权修改产品代码、真实数据或凭据。

### 具体动作

1. 记录 Windows、WSL、发行版、内核、文件系统、CPU/GPU 和工具链版本。
2. 按仓库锁文件重建 Rust、Node/npm 和 Linux/Tauri 系统依赖；优先 `npm ci` 与 Cargo locked 模式。
3. `.env` 只从模板或合成测试配置建立；正式凭据走以后单独的最小权限流程。
4. 使用临时目录、合成数据和临时 SQLite；不复制活动数据库。
5. 分层验证：
   - 依赖可重建；
   - TypeScript typecheck 与前端 build；
   - Rust 编译和平台中立测试；
   - 既有离线交互与 M1–M4 对应回归；
   - Linux/WSL 与 macOS 专属差异单列。
6. 构建后核 Git tracked 状态，确保生成物没有污染已提交字节。

### 成功标准

- 当前冻结提交在 WSL 能按锁文件重建依赖。
- 具名构建和回归有原始结果；跳过项、平台不适用项和既有 warning 单列。
- WSL 工作目录的 Git tracked 状态仍 clean。
- 当前开发工作流可以继续，形成 `WSL_DEV_READY`。

### 失败停止点

- 必须使用生产凭据、真实 provider 或活动数据才能启动；
- 只能复制 Mac `target`、`node_modules` 或活动 SQLite 才能运行；
- 安装脚本会连接真实服务、执行不可逆迁移或修改其他项目；
- 只有修改产品代码才能通过，而当前未获得 portability 修复授权；
- 把 macOS 专属测试跳过后冒充全量绿灯。

### 回滚

- Mac 保持原开发基线并可继续原工作流。
- 停止 WSL 构建，保留脱敏错误与版本清单供新任务诊断。
- 若需要代码兼容修复，另开具名 portability stage；不在 E 内顺手改。
- 删除缓存、clone 或发行版需要单独授权。

即使此阶段 `cargo build` 成功，也只能证明当前 Tauri-linked crate 在 WSL 可构建，不能使用 `HEADLESS_CANDIDATE_READY`。

## 8. F — 后续 Headless Core / Edge 实现路线

**谁操作：** 后续独立 Harness stage；开发者实现，独立审查与验收。

**磁盘/联网/授权：** 会修改合同、产品代码和测试；按本地/网络验证分层授权。M5–M10 仍需逐阶段显式激活。

### 建议实现顺序

1. 冻结平台无关 Core、transport、单写者、epoch、拒绝语义和数据 owner 合同。
2. 从 Tauri command、窗口和 AppState 生命周期中抽出 `syn-core-lib` / application service。
3. 建立无需 Tauri 或窗口即可启动的 Headless service。
4. 用同一 application contract 支持 in-process transport 与受认证远程 transport。
5. 实现超时、幂等、Stop、重启恢复、持久化、日志和可观测性。
6. 再加入 NodeIdentity、心跳、capability、ExecutionGrant、AuthorityLease、本地 spool 和 reconciliation。
7. 用合成数据覆盖断网、租约过期、策略漂移、重复、乱序、旧 epoch、撤销和旧节点恢复。
8. 通过独立验收后只标记 `HEADLESS_CANDIDATE_READY`，仍不接管正式 Primary。

### 成功标准

- Core service 无需 Tauri、窗口系统或 Mac UI 独立启动。
- transport、持久化、故障、重启和 Edge 重连具名测试通过。
- 旧实例和旧 epoch 无法继续正式写入。
- 当前 Mac 链路仍是回退基线，未迁正式数据、未改正式权威。

### 失败停止点

- 核心路径仍隐式依赖 Tauri/GUI；
- 写入 owner 或数据真源不明确；
- epoch/fencing 无法阻止旧节点；
- 测试只有 happy path；
- 必须接真实个人资料、凭据或 provider 才能证明候选。

### 回滚

停止候选服务，恢复使用 Mac 当前链路；不迁数据、不切路由、不改变 Primary。代码回滚按独立 stage 与 Git 授权处理。

## 9. G — 正式 Primary 数据与 authority epoch 切换

**谁操作：** 用户在切换窗口最终批准；我按冻结并演练过的操作手册执行。

**磁盘/联网/授权：** 生产级写入、停写、备份、恢复、路由、fencing、观察和回退均需单独授权，不能继承 A–F。

### 前置条件

- F 已达到 `HEADLESS_CANDIDATE_READY`；
- 独立备份和恢复演练通过；
- 切换脚本、旧 Primary fencing、epoch 单调性和回切窗口已验证；
- 候选完成约定稳定观察；
- 数据范围、停写窗口、责任人、GO/NO-GO 和回退触发器已冻结；
- 用户在当前窗口再次明确批准。

### 具体动作

1. 观察并冻结切换前事实、数据边界、旧 epoch 和写入来源。
2. 进入短时停写，确定性 fencing 旧 Primary。
3. 导出、hash、恢复并校验候选 Primary 数据。
4. 启动新 epoch，验证旧 Lease/Grant/写入被拒绝。
5. 让 Mac Edge/UI 连接新 Primary，核关键对象、事件、审计和 outbox。
6. 在约定观察窗口内持续检查双写、延迟、错误和恢复。
7. 只有全部通过后发布绑定节点、字节、epoch 和窗口的切换 receipt。

### 成功标准

- 新 Primary 是唯一合法写者，旧 Primary 已被 fencing。
- authority epoch 单调递增，无双写、倒退或无法解释的数据差异。
- 数据量、关键记录、事件序列、审计和 outbox 一致。
- Mac Edge/UI 已切换并通过观察窗口。
- 形成 `PRIMARY_SWITCH_ACCEPTED` 真实 receipt。

### 失败停止点

- 任一双写、split-brain、epoch 倒退、数据不一致或无法解释的超时；
- 旧 Primary 无法确认已被阻断；
- 备份不可恢复或回切边界不再安全；
- 观察证据缺失，却只凭配置文件、HTTP 成功或口头声明要求 GO。

### 回滚

不能简单重启旧 Primary。必须先停止新写入、核对新旧数据边界，再按已演练的 epoch/fencing 手册恢复唯一写者。边界不清时冻结写入并进入人工恢复，不能冒险形成双主。

## 10. 用户手工最小化

### 现在

- A 阶段只处理仓内文档；用户无需操作 Mac 或 5600X。
- 不要手工复制仓库、开放端口、安装依赖、迁数据库或发送密码、IP、私钥和 Tailscale 凭据。

### 以后进入 B 时

用户只需要：

1. 把 Mac 和 5600X 开机并解锁；
2. 在确有必要时处理 Windows 登录、Tailscale 登录或管理员弹窗；
3. 对每个会写配置或重启的动作单独确认。

其余系统检查、命令生成、hash、bundle、传输和验收记录应尽量由执行方完成。

## 11. 全局停止规则

任一阶段出现以下情况，立即停在事实，不自动扩大范围：

- 目标设备、账号、目录、数据 owner 或写入 owner 不唯一；
- 需要真实凭据、个人资料、provider、connector、部署或发布，但没有当前授权；
- 需要 reset、clean、stash、覆盖或删除既有工作；
- 当前 HEAD、tree、Harness stage/leaf/auth 与冻结基线漂移；
- 局部构建、ping、HTTP 成功或候选进程被要求冒充下一层验收；
- M5–M10、Headless 实现或 Primary 切换被要求从开发迁移授权中自动继承。
