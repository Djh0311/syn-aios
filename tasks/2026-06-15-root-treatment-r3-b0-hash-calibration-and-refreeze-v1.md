# Root Treatment / R3 B0 Hash Calibration And Re-Freeze v1

日期：2026-06-15

状态：已完成

性质：B0 hash 口径校准 + B0 re-freeze 小包。本包不是 B1 production apply，不创建 production DB，不执行 read-cut，不停写 JSON / sidecar。

## 0. 背景

R3 Level B B1 Production Apply 首次尝试已按硬中止条件记录为 `failed_classified`。失败点为 source aggregate hash 不匹配：

- B0 execution record 冻结值：`2fbdb7bfdc71b30d5b4d2bec2dfdde50de98ab24942c8ba550d29b6b539d3b53`
- Level-B preflight 当前值：`31cdea623d928ea2dc13d0a02eaefd23f2df1a27f454d5d7ea17d51fe3b4b801`

只读复查显示真实源目录仍只有 `workflow-state.v0.json` 与 `plan-authorizations.v1.json`，两者单文件 SHA-256 与 B0 清单一致。因此当前分类为聚合 hash 算法口径分叉，不是已观察到的源文件内容漂移。

## 1. 目标

统一 R3 B0 / Level-B preflight 的 source aggregate hash 口径：

- 抽出或指定一个 canonical aggregate hash helper。
- `scan_workbench_state_root_preflight_with_config` 必须调用该 helper。
- B0 re-freeze evidence 必须引用同一 canonical 算法和当前 preflight 输出。
- 测试证明同一组文件通过 canonical helper 与 preflight report 得到同一个 aggregate hash。

## 2. 允许修改

- `prototypes/productized-desktop-shell/src-tauri/src/utils/hash.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/workbench_sqlite_preflight.rs`
- 本任务包
- `evidence/r3-level-b/b0-hash-calibration-*/`
- `handoffs/2026-06-15-r3-b0-hash-calibration-and-refreeze-v1.md`
- `CURRENT.md` checkpoint

## 3. 硬边界

- 不执行 B1 apply。
- 不创建 production DB。
- 不创建真实 backup / rollback manifest / production apply report。
- 不切 read path。
- 不停写 JSON / sidecar。
- 不写真实 `WORKBENCH_STATE_ROOT`。
- 不改 Level-A source / output guard。
- 不削弱 B1 enablement confirmed-path 守卫。
- 不改 Tauri command、UI、app startup 或产品全局读写路径。
- 不执行真实 Codex。
- 不读取或写入 `/Users/yoyi/.codex`。
- 不读取 secret / token / `.env` / keychain / OAuth / provider credential / full transcript / rollout / prompt body。

## 4. 实现要求

Canonical helper 必须：

- 输入为排序后的 source file entries。
- 对存在文件使用 `path_ref`、`file_hash` 和 `classification` 参与 hash。
- 不把缺失 optional sidecar 作为文件条目参与 hash。
- 对 rejected entry 不读取 body；若 entry 已进入 report，则以 path / classification 参与 hash，保持 preflight 阻断态可复现。
- 输出稳定 64 位 SHA-256 hex。

Preflight report 必须暴露或明确记录算法口径，避免 evidence 再写成另一套手工算法。

## 5. B0 Re-Freeze 要求

re-freeze 前只读确认：

- `workflow-state.v0.json` SHA-256 仍为 `4bd5434fdca9e82c8fafc42989e1a267ed7d677bfe2972273fb3afaa26829972`
- `plan-authorizations.v1.json` SHA-256 仍为 `6962e4781f49246525d4cde37d3133924a66faa12b8aab90db106c3c9f401b0e`
- `WORKBENCH_STATE_ROOT` 下仍只有这两个 allowed 文件

B0-refresh execution record 必须写清：

- 旧 B0 aggregate hash
- canonical aggregate hash
- 算法说明 / algorithm id
- 两个单文件 hash
- 未创建 DB / 未写 source / 未触碰 `.codex`
- B1 retry 的前置条件：用户在场重新确认 expected hash

## 6. 验证命令

必须运行并记录：

```bash
cargo fmt -- --check
cargo test --lib sqlite_preflight
cargo test --lib sqlite_production
cargo test --lib
node scripts/harness/workbench-shape-gate.js --mode check
git diff --check
```

## 7. 独立复核要求

复核线需确认：

- B0 / Level-B preflight 已共用 canonical helper，而不是只改 evidence 数字。
- 测试覆盖同一组文件同一 aggregate。
- 真实源只读 re-freeze，没有写 source。
- 未执行 B1 apply，未建 DB。
- Level-A 与 B1 confirmed-path guard 未削弱。
- B0-refresh evidence 不把本包声明为 B1 完成。

## 8. 不接受为

- B1 production apply 已执行。
- 真实 production DB 已创建。
- read-cut 已执行。
- stop-write 已执行。
- R3 Level B 完成。
- 多 agent 并行真实执行已解锁。
- 真实 Codex 执行或 `.codex` 接触已发生。
