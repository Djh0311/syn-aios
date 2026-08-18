# M5C01 受保护 WIP 与载体分层归责 v1

观察时间：`2026-08-18T19:33:03+08:00`

状态：`PRESERVE_IN_PLACE / EXCLUDED_FROM_M5C01 / NO_CLEAN / NO_OWNERSHIP_INFERENCE`

## 1. 用户自有、已提交载体（不是 WIP）

用户自有开源门面已于 `c1025ba81b6c7885a16529b8f66c919655db48e4` / tree `f60a315ff743ebb24eea192378c388ea277bda75` 以精确 7 路径独立提交：`README.md`、`LICENSE`、`CONTRIBUTING.md`、`SECURITY.md`、`prototypes/productized-desktop-shell/package.json`、`prototypes/productized-desktop-shell/src-tauri/Cargo.toml`、`docs/harness/unfinished/OSS-01-public-push-and-codex-oss-application.md`。

该提交是 M5R09 记账 `8e6f59f` 与 lifecycle opening `b2429f6` 之间的独立用户载体；不属 M5R09/M5C01 候选。M5C01 候选 range `b2429f6..de98d69` 对这 7 路径零差异；OSS-01 继续 unfinished，未提升、未 push、未提交外部申请。

## 2. 活动 Harness runtime（只绑定观察时点，不承诺内容 hash）

| Git | path | 漂移边界 |
|:---:|---|---|
| ` M` | `docs/harness/usage/.observed.json` | Hook 可继续改写 |
| ` M` | `docs/harness/usage/.observed.jsonl` | Hook 可继续追加 |
| `??` | `docs/harness/usage/host-events.json` | runtime 可漂移 |
| `??` | `docs/harness/usage/host-health.json` | runtime 可漂移 |
| `??` | `docs/harness/usage/.turns/` | 观察时 1 个 turn 文件；runtime 可新增 |
| `??` | `docs/harness/reports/2026-08-18-01a01376-2915-7c40-acf9-899811b2da98-01a01376-29b2-7f22-966a-ce82ce91acab.md` | Harness 生成；不纳入候选 |
| `??` | `docs/harness/reports/2026-08-18-01a013e2-03bd-7d53-a4d4-e5fb506548e4-01a013e2-0443-70e1-8eca-ffb8ce4850fa.md` | Harness 生成；不纳入候选 |

以上 7 项只记录观察时点与路径；本报告不为它们声明稳定 SHA-256。

## 3. 静态受保护 WIP（承诺内容 hash）

| 类别 | 数量 | 当前 hash 承诺与处置 |
|---|---:|---|
| tracked legacy Rust WIP | 21 | 逐路径 SHA-256 与 `M5R09-protected-wip-attribution-v1.md` 静态表一致；全部原位保全 |
| dated Harness reports | 2 | `01a010f4...md` 与 `01a0130d...md` 的 SHA-256 与 M5R09 静态表一致；不纳入候选 |
| generated Linux schema | 1 | SHA-256 与 M5R09 静态表一致；未跟踪、原位保全 |
| untracked M6 candidates | 6 | 5 个 `.rs` + 1 个 `.bak` 的 SHA-256 与 M5R09 静态表一致；M6 未激活，不采纳 |

30 个精确 `path + SHA-256` 对仍以已接受的 `docs/harness/reports/M5R09-protected-wip-attribution-v1.md` 静态表为规范清单；M5C01 在观察时用该表生成 `sha256sum -c` 输入，30/30 返回 `OK`。无噪精确日志：`.syn-gates/evidence/M5C01-de98d69/34-static-wip-hashes-exact.log`，exit 0；输入清单见同目录 `34-static-wip-expected-input.log`。

`commands.rs` 整文件 hash 仍为 M5R09 表记录值；相对 HEAD 的候选外残余仍为 `59 insertions / 56 deletions`。本叶没有暂存、提交、覆盖或归责这些字节。

## 4. 结论

- 权威工作树观察时共 37 条 Git status 路径：7 项活动 runtime + 30 项静态受保护 WIP。
- M5C01 只提交 closeout 文档和 Harness 生命周期路径；没有 `git add -A`、reset、stash、clean、worktree prune 或产品源码写入。
- 历史 worktree 注册项只记录到 `ENG-01`，本叶只移除了自己创建且已验证的 `/tmp/syn-m5c01-verify-de98d69`；未处理其他会话载体。
