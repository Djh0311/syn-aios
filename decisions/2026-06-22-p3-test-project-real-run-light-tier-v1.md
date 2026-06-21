# 决策 · P3 真跑（固定测试项目）下放为轻档（2026-06-22）

## 拍板
用户明确授权：**P3 真跑（节点真起 codex）就在固定测试项目里跑 = 轻档**，不再重档、不再逐次授权。

## 为什么
固定测试项目 `/Users/yoyi/codex-workflow-mario-test` 是**专用开发测试靶子**（git 仓、可回滚、不是用户的真实/生产仓），**随便读写没有真实不可逆损害**。高危#1 本意是保护真实/生产项目；测试项目等同沙箱，不该按高危对待。

## 精确边界（轻档成立的前提，必须守住）
1. **path-lock 不动**：真跑目标仍锁死该固定测试项目——这是挡住用户真实仓的那道，**不能松**。
2. **沙箱不外溢**：codex 仍被关在测试目录（workspace-write + add-dir 限定，已验过 `沙箱只动测试目录`）。
3. ①②任一松了 → 立刻回高危。

## 没松的
- **真跑进任何非测试真实项目（用户实际仓/生产）= 仍高危#1、仍锁着**，用户授权那一下不可省。
- 高危#1 的"真实项目"从此重定义为"**非测试的真实项目**"。

## 实现含义（P3 落地时）
- 去掉 env 闸 `WORKFLOW_ENGINE_TEST_CONFIRM`（重档时的"你确定"belt，现多余）；**保留 path-lock**。`workflow_engine_test_project_unsealed` 改成只查 path（不查 env）。这是改闸代码，属本次授权的松闸；主导线核时只确认"去 env、path-lock + 沙箱字节未松"。
- P3 从"重档·逐次授权·远期" → **轻档·可现在做**（P1/P2 顺了即接）。

## 影响面（被推翻时要复核的文件）
- `AGENTS.md` 高危#1 + §五（已细化）。
- `CURRENT.md` §三（P3 挪轻档）+ §四a（测试项目轻档）。
- `docs/plans/2026-06-21-workflow-canvas-two-surfaces-one-engine-v1.md` §6 P3/§7 + `...-session-and-scope-model-v1.md` §7。
- `handoffs/2026-06-21-canvas-p1-kickoff-v1.md` §4（P3 不锁）。
- 代码（P3 时）：`src-tauri/src/commands.rs` 的 `workflow_engine_test_project_unsealed`（去 env、保 path）；`codex_local_runner` 沙箱不动。
- 关联：`decisions/2026-06-21-next-step-unseal-workflow-engine-for-test-project-v1.md`（第一刀解封，本决策在其上下放档位）。
