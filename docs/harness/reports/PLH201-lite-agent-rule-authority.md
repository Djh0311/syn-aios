# PLH201 Lite 标准 Agent 规则纠正与认领

日期：2026-08-08

## 结果

Syn 的 AGENTS/CLAUDE 已从迁移期厚适配改成 Lite 源仓 `19be06b` 的 0.4.0 标准规则，逐字一致并进入 Lite ownership。agent 的生命周期、授权、硬门和退场规则现在由 Lite 统一维护，不再由 Syn 复制一套通用规则。

原先讨论的三类项目内容已经从 agent 规则退出：中文偏好不在 AGENTS；Rust non-test build 继续留在 `docs/harness/checks.json`；Syn 产品方向继续留在产品资料。`catch:` 仍由未改动的 `.githooks/commit-msg` 机械执行，不成为另一套 agent 宪法。

## 规则与 ownership

- 纠正前 AGENTS SHA-256：`6479cfcbfa87a57f9ce68922e3599b5c7ebd85418240ffc2fa7e742e8a55a5b2`。
- 纠正前 CLAUDE SHA-256：`f72027c69cf153d53c40ed58b1669bf53045f53e4889f16fdbb2a919545aa1d3`。
- Lite 标准 AGENTS SHA-256：`c86445f67d5d7af1d33f18716cad7871f2b34d32968c9fad3129befc5c28ffba`。
- Lite 标准 CLAUDE SHA-256：`b7ca0575486c601cc1fc90460c7946e3068733f4e9e0cd2eed47b8f494564162`。
- `cmp` 对源仓两份规则均为 0；旧项目特有文字扫描为 0。
- ownership 从 core 0.3.0 / 44 项变为 0.4.0 / 46 项；新增 AGENTS、CLAUDE 两项。
- 当前 46 项中 42 项匹配记录 hash，3 项是项目化 plan/MISTAKES/checks，1 项是 Stage 1 关闭后主动归档而缺失的旧 stage 路径。该缺失继续按 Lite 原有“主动删除不补回”语义 HOLD。

首次显式 `--upgrade --adopt-agent-rules --write` 更新 1 个干净 runtime 文件、认领 2 个规则、保留 43 个；随后普通 `--upgrade --write` 写 0、跳过/保护 46，完整 status 指纹前后都是 `590a77906d09df3e67f9ba9b19f94f66434545668ea969d43c1e4c6cc064db8b`。

## 验证

- Lite `chain` / `progress` / `auth`：Stage 2 / PLH201 / 新用户授权有效。
- `hl check quick`：`git-diff-check` 通过。
- `hl check task AGENTS.md`：只选择并通过 `harness-lite-cli-syntax`。
- 新 runtime `ownership.js` 与 `hl.js`：`node --check` 通过。
- `.githooks/pre-commit`、`.githooks/commit-msg`：`sh -n` 通过，内容无改动。
- `git diff --check`：通过；`prototypes/**` diff 为 0；staged 为 0。
- 没有运行产品 full、真实 App、provider、数据库、浏览器、部署、发布或 push。

## 受保护 WIP

两个脏 worktree 的三重指纹和 staged 与 Stage 1 冻结值完全相同：

| worktree | status | tracked diff | untracked 内容清单 | staged |
| --- | --- | --- | --- | --- |
| `/Users/yoyi/workspace/product-line` | `9de7a6ac5e561ec61b6404204f960d78ddf9cfad5c51bf74716126b539bb598b` | `a9b688c3aaacb7fc51d0f9dd7fdbb7c66b75aec77ee0a7f26be72a060cf16fad` | `0214a578acc5852537d8b52d64cd5bbb0736a821a7ee44a3be1f9a05b2ee5941` | 0 |
| `/Users/yoyi/workspace/product-line-syn-fnd-002` | `60f1395f2df3b0e7355952b65f23bae4c121fbe63090802d8c4793178385f399` | `996452ac558c938d8558bd77c6d233be0d26f00d4989aa67a0dc773475086b2e` | `00ea3446b3bc465de304018f2ca803b048ace695619e6e066e516c1fce89629a` | 0 |

## 改动和边界

实际产品仓改动只在 AGENTS、CLAUDE、一份 Lite runtime、ownership 与 Stage 2 的 Harness 记录。`checks.json`、`policy.json`、`.githooks/`、产品代码和产品资料都没有修改。Lite 源仓保持 `19be06b` clean；两边都只做本地提交，不 push。

PLH201 和 Stage 2 已归档，plan 已勾选；关闭后 7 个 leaf 全部完成，Stage 2 authorization 已失效。当前没有 stage/leaf，等待用户指定下一项。
