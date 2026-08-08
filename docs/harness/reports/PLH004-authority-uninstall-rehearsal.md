# PLH004 唯一项目权威与旧 Harness 卸载演练报告

日期：2026-08-08

## 结论

在 detached 临时 worktree `/private/tmp/product-line-harness-lite-uninstall-rehearsal-20260808`，以 `c9d53a9` 为基线完成了一次完整切换演练。演练证明：可以只迁移 Harness 与入口事实，不改产品源码，不碰两个既有 dirty worktree，并得到一条可运行、可重复安装、可卸载和可恢复的 Harness Lite 路由。

PLH001 的“51 个删除目标”在 FND 合并后需要精确修正：

1. 旧 manifest 当前 SHA-256 是 `9460364c...e2267b2`，其中 50 个 `created + replace-managed` 文件仍全部匹配登记内容与模式。
2. FND 链新增了未入 manifest 的 `scripts/harness-v2/active-path-audit.test.js`，它是第 51 个旧 carrier。
3. `.harness/manifest.json` 必须最后删除，是第 52 项。
4. FND 链同时新增真实生效的 `.githooks/pre-commit`，它调用旧 `git-gate.js` 和 `codebase-map.js`；必须先适配为项目自有的 `git diff --cached --check`，否则删除 runtime 后所有 commit 都会被旧 hook 卡死。

因此 PLH005 的真实删除集是 52 个精确文件，不是递归猜目录；manifest 50 项清单由 `ownership=created && policy=replace-managed` 唯一生成，receipt 为 `eeddf05e...adf7`。

## 唯一权威适配

演练中的 AGENTS/CLAUDE 没有被通用模板覆盖，而是保留并压实了本项目规则：中文协作、五类真实边界、直接证据、Rust production 路径必须 non-test build、保护 dirty/untracked、禁止 reset/clean/stash/bulk staging、Git 精确暂存与 `catch:` 约定、主导线/执行线职责和 Syn 产品边界。

旧入口被替换为：

```text
AGENTS.md
→ docs/harness/plan.md
→ 唯一当前 stage
→ 唯一 current leaf
→ docs/harness/authorization.json
```

旧 `AUTHORITY.md` / `CURRENT.md` 移入 `docs/harness/history/adaptive-v0.5/` 并明确为不生效历史。README、`docs/project-context.json`、Code Map README/domain、`TASK_TEMPLATE.md`、`tasks/README.md`、`docs/plans/README.md` 和 `plans/v0.5.0/` 都改为不从旧 ACTIVE/READY/next 推导工作。

`.gitignore` 只放开项目拥有的 `.claude/settings.json` 和 `.claude/harness-lite/**`，避免把其他 Claude 本地内容纳入版本控制。Claude 与 Codex 宿主注册文件均由安装器 snippet 人工合并，演练路径下实际 JSON 与目标 runtime 对应。

## 卸载、重复安装与恢复

- Lite core：0.3.0，ownership 44 项。
- 重复 `install --write`：写 0，跳过/保护 46。
- Lite 卸载 dry-run：会删 40、保护项目定制 4。
- Lite 实际卸载：删 40、保护 `plan.md`、`stage-01.md`、`MISTAKES.md`、`checks.json`；之后从候选快照恢复。
- 候选范围内容+模式指纹：恢复前后均为 `dac474b4...874b`。
- 切换前快照：162 entries，archive SHA-256 `02b76077...d59d`。在独立目录解包后，旧 manifest SHA、pre-commit blob、Lite ownership SHA 和两个 hook 的 0755 模式均与 archive 内容一致。

恢复是有清单的文件级恢复，不宣称原子事务。真实切换仍保留 Git `c9d53a9` 作为所有 tracked 旧文件的直接回滚来源。

## 验证

- `hl chain` / `progress` / `auth`：PASS，指向 PLH004 和新的 stage-01 授权。
- 7 个项目/Hook JSON：解析 PASS。
- Lite runtime 全部 JavaScript `node --check`：PASS。
- `.githooks/pre-commit` / `commit-msg` shell syntax：PASS。
- 独立临时 Git index 运行适配后的 pre-commit：PASS。
- `hl check quick`：`git-diff-check` PASS。
- `hl check task AGENTS.md`：`harness-lite-cli-syntax` PASS。
- `hl tests AGENTS.md`：没有专用路径映射，给出通用建议；没有执行产品测试。
- 活动 Hook/宿主 carrier 的旧 runtime 引用：0。
- AGENTS/CLAUDE/README/project-context/Code Map README 等当前入口的旧路由引用：0。
- `git diff --check`：PASS。
- `prototypes/**` diff：0。
- 两处受保护 WIP status 指纹仍为 `9de7a6ac...98b` 和 `60f1395f...f399`。

没有运行 real Codex、真实 App、provider、数据库、浏览器、部署、发布或 push。

## PLH005 精确修正

PLH005 除原清单外，必须把以下文件加入精确允许面：

- `.gitignore`
- `.githooks/pre-commit`（适配，不能直接删）
- `.claude/settings.json`
- `.codex/hooks.json`
- `TASK_TEMPLATE.md`
- `tasks/README.md`
- `docs/plans/README.md`
- `plans/v0.5.0/README.md`
- `plans/v0.5.0/SYN-FND-001.md`
- `docs/code-map/README.md`
- `docs/code-map/domains/development-harness.json`

删除集另显式加入 `scripts/harness-v2/active-path-audit.test.js`。保留 `.githooks/commit-msg`、`docs/harness-catch-log.md`、其他 Code Map domain、产品 bridge 代码、合同、决定、证据、历史任务与 templates。
