# 阶段8 Syn Primary/Edge D0 文档与迁移权威收口

总计划：product-line 唯一基线与 Harness Lite 切换
目标：把外部 Primary/Edge 草案修订为仓内可审计的架构候选，并把 5600X + Windows/WSL2 + Tailscale 开发迁移拆成独立计划；只收口文档权威和后续停止门，不连接设备、不迁源码、不实现或切换 Primary。

编号说明：这里的 Harness `stage-08` 只是第八个开发护栏阶段，承载 D0 文档收口；它不是产品路线中的 M8 connector 阶段，不激活 M8 或其他 M5–M10 工作。

干完的标准：

- 架构候选明确标为 `CANDIDATE / DRAFT`，保存外部来源 SHA-256，并修正近期 Primary 时间线、Mac UI 过渡期、失效权威引用和 M1–M10 状态。
- 唯一候选登记新增 Primary/Edge 候选入口，明确开发环境迁移不等于候选转正、Headless 实现或 Primary 切换。
- 独立迁移计划按 A–G 分阶段写清操作者、动作、成功标准、失败停止点、回滚、写磁盘/联网/授权边界。
- M1–M4、现行架构与产品正本、M4R07 证据保持不改写；M5–M10 保持 `PLANNED / NOT_ACTIVE`。
- 本阶段不 Git add/commit/push，不联网、不连接 5600X、不安装或配置设备、不迁源码、不修改产品代码、不实现 Headless Core、不迁 Primary 数据或切换 epoch。

允许动：

- docs/product/syn-primary-edge-core-distributed-runtime-architecture-candidate-v2.md
- docs/product/candidate-register-v1.md
- docs/plans/2026-08-13-syn-5600x-wsl-development-environment-migration-plan-v1.md
- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-08.md
- docs/harness/leaves/D0A01-syn-5600x-wsl-d0-authority-closeout.md
- docs/harness/done/2026-08/D0A01-syn-5600x-wsl-d0-authority-closeout.md
- docs/harness/done/2026-08/stage-08.md
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

只读：

- /Users/yoyi/Library/Containers/com.tencent.xinWeChat/Data/Documents/xwechat_files/wxid_o97veuui2g6m22_75c7/temp/RWTemp/2026-08/d7632f22f514452b259d2a63ea8504ee/2026-08-11-syn-primary-edge-core-distributed-runtime-architecture-draft-v1(1).md
- docs/product/authority-register-v1.md
- docs/product/syn-product-canon-v1.md
- docs/product/knowledge-infrastructure-canon-v1.md
- docs/workbench-system-architecture-v1.md
- docs/current-state.md
- docs/plans/README.md
- docs/plans/2026-08-01-syn-personal-ai-workbench-master-development-plan-v1.md
- docs/harness/done/2026-08/stage-07.md
- docs/harness/done/2026-08/M4R07-isolated-product-reacceptance-closeout.md
- docs/harness/reports/M4R07-isolated-product-reacceptance-closeout-behavior-receipt.json
- docs/harness/reports/M4R07-isolated-product-reacceptance-evidence/manifest.json

不许动：

- 现行产品正本、现行架构正本、当前状态、M1–M4 冻结合同、已归档 stage/leaf 和 M4R07 receipt/manifest
- 产品源码、测试、构建产物、运行数据、活动 SQLite、环境文件、凭据和密钥
- M5–M10 激活或实现、Headless Core/Edge 实现、Primary 数据迁移或 authority epoch 切换
- Windows、WSL2、Tailscale、SSH、防火墙、端口转发、Docker、systemd 或其他设备配置
- 网络、远端、服务器、真实数据、真实 provider/connector、部署和发布
- Git add、commit、push、merge、rebase、reset、clean、stash、删除或覆盖既有工作

## 叶子

- [x] D0A01 Primary/Edge 架构候选与 5600X/WSL 开发迁移权威收口
