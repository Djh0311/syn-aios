# PLH006 分区验收、回滚证据与阶段收口

日期：2026-08-08

## 结论

Stage 1 的六区验收通过并已归档。迁移分支已经同时具备一条可复现代码基线和唯一的 Harness Lite 开发入口；旧 Adaptive Harness 不再参与活动流程。两个既有脏 worktree 的路径、内容和暂存状态都没有变化。本阶段没有 push、部署、发布或接触其他外部边界。

本报告只验收 Harness 迁移和已形成的代码基线，不把机器检查冒充真实 App 或产品发布验收。

## 1. 代码区

- 当前分支：`codex/product-line-lite-migration`。
- 验收前 HEAD：`c9d53a9063a076d409239a35cdd12e25332c1869`。
- 候选 tree：`04dbe9e7ae546e55697b381d8ca9f2f83e94a5c9`，与 PLH002 隔离演练冻结值一致。
- 两个父提交：`main@e5269557b65998de56d09d83fa901c4bd92145bd`、`syn-fnd-002-dev@2a7229bde7f0b5bb6701f4a7aa21944973f1881f`。
- `prototypes/**` 工作树 diff 为 0；最终迁移不改产品实现。
- 验收时迁移树 staged 为 0；最后只按已核清的迁移路径清单暂存并创建本地提交。

两个受保护现场三重指纹都与 PLH001 完全相同：

| worktree | status | tracked diff | untracked 内容清单 | staged |
| --- | --- | --- | --- | --- |
| `/Users/yoyi/workspace/product-line` | `9de7a6ac5e561ec61b6404204f960d78ddf9cfad5c51bf74716126b539bb598b` | `a9b688c3aaacb7fc51d0f9dd7fdbb7c66b75aec77ee0a7f26be72a060cf16fad` | `0214a578acc5852537d8b52d64cd5bbb0736a821a7ee44a3be1f9a05b2ee5941` | 0 |
| `/Users/yoyi/workspace/product-line-syn-fnd-002` | `60f1395f2df3b0e7355952b65f23bae4c121fbe63090802d8c4793178385f399` | `996452ac558c938d8558bd77c6d233be0d26f00d4989aa67a0dc773475086b2e` | `00ea3446b3bc465de304018f2ca803b048ace695619e6e066e516c1fce89629a` | 0 |

## 2. 控制面区

- 切换前 archive 中旧 manifest SHA-256 为 `9460364c1676911c14f7a461dbb74bab1a9e91fabf733c3b450973558e2267b2`。
- manifest 唯一生成 50 个 `created + replace-managed` 目标；再加漏账 test 和最后删除的 manifest，共 52 个退出目标。
- 52/52 当前不存在；`scripts/harness-v2/**` 剩余文件 0；`harness.config.json` 不存在。
- 活动 Hook、宿主 carrier 对旧 runtime 的引用 0。
- AGENTS、CLAUDE、README、TASK_TEMPLATE、project-context、Code Map README、tasks/plan 入口对旧活动路由的引用 0。
- 旧 AUTHORITY/CURRENT 作为带来源说明的历史快照保存在 `docs/harness/history/adaptive-v0.5/`，不再授权或定义 next。

## 3. Lite 生命周期区

- Lite core 0.3.0，ownership 44 项：40 项匹配安装 hash，4 项为项目化保护文件，0 missing。
- 4 个项目化文件是 `plan.md`、`stage-01.md`、`MISTAKES.md`、`checks.json`；重复 `--upgrade --write` 保留它们，实际写 0、跳过/保护 46。
- 重复升级前后完整 status 指纹都为 `52269b8df192bd2170485f086c874fff96eb032d91120b511443ff00e3c7670c`。
- 15 个活动 JSON 解析通过，24 个 Lite JavaScript `node --check` 通过，两个 Git Hook `sh -n` 通过。
- `core.hooksPath=.githooks`；pre-commit 与 commit-msg 均为 0755；Claude/Codex 登记的 8 个 Hook 目标全部存在。
- `hl chain`、`progress`、`auth` 在收口前均通过；授权是新签发的 Stage 1 授权，不来自旧 READY、任务包或历史文本。
- `hl check quick` 通过；`hl check task AGENTS.md` 只选择并通过 `harness-lite-cli-syntax`。

Stop 的静态与运行时证据都表明没有产品测试：`stop.js` 不加载 checks、不调用 child process，也没有 cargo/npm/pnpm/pytest 命令；真实 Stop 前后 `check-results.jsonl` 都是 4 行，SHA-256 都为 `3f5686ca7b01eaab1e283589030c65eeefaeaa6b40e529ed6ed72c42aa28f1b9`，`prototypes/**` diff 前后也都为 0。

## 4. 项目检查区

- 本 leaf 直接相关的小检查只有 `git diff --check` 和 Harness Lite CLI syntax，均通过。
- PLH003 已对唯一代码 tree 跑过 `cargo check --lib --offline`，以及完整 lib 测试 1348 pass / 45 ignored / 0 fail；PLH006 没有重复冒充一次新的产品 full 验收。
- 未运行真实 App、real Codex/Claude 会话、provider、数据库、浏览器、部署或发布。

## 5. 回滚区

切换前快照 `/private/tmp/product-line-harness-lite-real-cutover-20260808-before.tar` 共 165 个 archive entry，SHA-256 为 `85c491ef837ffac3deddf719b2ffc41188749f37e5eb467932f77177cd937216`。它已在全新隔离目录解包验证：

- 旧 manifest SHA-256：`9460364c1676911c14f7a461dbb74bab1a9e91fabf733c3b450973558e2267b2`。
- 切换前 Lite ownership SHA-256：`89cfdd5432640ce1b5b8db7ae3718c2bb0de67c6284b7e4642228d52f2f96d39`。
- 旧 pre-commit Git blob：`068ca087f5d6cd3b0906269de2d30b2ac135ad4e`。
- 漏账 test Git blob：`17aeb5d4acdbcbf1c1814a2ed45a7238c16b99f7`。
- 恢复后的 pre-commit、commit-msg 模式均为 0755。

因此切换前状态可以由该快照逐字恢复；tracked 旧文件还可从 `c9d53a9` 恢复。隔离探针和临时 index 在最终提交复核后删除。

## 6. Git 与外部区

- 本分支没有 upstream；main 仍为本地 `ahead 33`，没有把本次迁移解释成已发布。
- Harness Lite 源仓保持 `codex/harness-lite-game-extension` clean。
- 没有 fetch/push、远端写入、合并主线、部署、发布、provider、数据库、浏览器、真实账号或真实消息。
- 最终本地提交必须实际包含本报告、六个 leaf、Stage 1 归档、plan 勾选、Lite runtime/ownership、项目适配和旧 carrier 删除；提交后用 `git show --name-only` 反查，而不是靠报告自称。

## 阶段关闭与下一入口

- PLH006 已移入 `done/2026-08/`；Stage 1 已移入同月 done，plan 中阶段 1 已勾选。
- 关闭后 `chain` / `progress` 显示 6 个 leaf 全完；`auth` 明确返回“授权不属于当前工作”，所以 Stage 1 authorization 没有继承。
- 当前 `leaves/` 没有任务、`stages/` 没有当前阶段，进入“等待用户指定下一项”。
- 下一项可以由用户新建 Stage 2 任务；不会从旧 Harness、历史 FND 计划、脏 WIP 或当前报告自动恢复产品开发。
- 已记录 `AUTH-DIRECTIVE-BRIDGE-GAP`：用户聊天中的明确整阶段授权需要人工落成 `authorization.json` 才能被硬门识别。该问题留给 Harness Lite 源仓后续单独设计，不在 product-line 迁移里顺手改核心。
