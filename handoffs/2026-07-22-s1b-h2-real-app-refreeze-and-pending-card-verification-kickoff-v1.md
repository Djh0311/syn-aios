# Kickoff：S1B-H2-R2 修后 binary 重冻结与真实 App Pending 卡验收 v1

- 日期：2026-07-22
- 状态：**待现场开工令；当前禁止执行**
- 权威任务包：`tasks/2026-07-22-s1b-h2-real-app-refreeze-and-pending-card-verification-package-v1.md`

## 给执行 Codex

先完整阅读权威任务包，再行动。不要把本 kickoff、历史“可以做”或任何旧授权当作现场开工令。

当前只知道：H2 代码与离线面已有通过证据，M5-F1-R1 与知识库生产写路修复已完成，但 H2 真实 App 验收仍未通过；源码修后，2026-07-20 的旧 debug binary/hash 已失效。历史 `proposal=74 / Pending=17 / chain=40` 不是当前基线，必须重新只读采集。

## 唯一开工入口

只有用户在后续新消息中明确给出以下文字，才可进入任务包 Gate 0：

> S1B-H2-R2 开工；授权重建并重冻结当前 debug binary，在真实 App 中发送两句验收话，只允许 supervisor_orchestrator.submit_proposal 生成一张 PendingUserConfirmation 卡；不得批准卡、不得启动 chain、不得改测试项目。

未收到时，只回复“任务包已就绪，等待现场开工令”，不得检查真实现场、启动 App、读取或写入真实 store、构建 binary 或发送验收消息。

## 开工后的硬顺序

1. **只读 Gate 0**：确认 Workbench/dev/holder/registry 清零；冻结 HEAD、脏项、相关源码 hash、真实 store 的 `B0/P0/C0`、thread/generation，以及固定测试项目 hash/git 状态。
2. **重建 Gate 1**：从 `prototypes/productized-desktop-shell` 运行：

   ```bash
   ../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
   ```

   冻结并只启动当前 `src-tauri/target/debug/codex-governance-workbench` 裸 binary。不得用旧 `.app`。
3. **第一回合 Gate 2**：发送 `我想给这个游戏里的标题改成小马里奥`；等 canonical 与主管自然回复完成。未回复前不发第二句。
4. **第二回合 Gate 3**：发送 `按这个出方案`；只允许 `supervisor_orchestrator.submit_proposal`，要求恰好新增一张 Pending 卡，chain 不变。
5. **幂等 Gate 4**：只刷新/重新进入一次观察，不重发第二句。
6. **收尾 Gate 5/6**：正常关闭 App，只读对账并写新 evidence/CURRENT；不 stage、不 commit。

## 一票停机

遇到以下任一情况立即停止并按事实回传：

- 未获得上述新开工令；
- 相关源码 hash 漂移或脏归属不清：`BLOCKED_DIRTY_OVERLAP`；
- App/dev/store holder 未清零：`BLOCKED_LIVE_HOLDER`；
- build 失败或启动对象不是新冻结裸 binary；
- 第一条 canonical 未记录、主管未回复、quota/transport 失败；
- 出现工具批准提示、第二个工具或扩大权限要求；
- 新增卡不是恰好一张、refresh 后重复；
- chain/worker 增长或固定测试项目发生变化；
- 原始 stderr/审计 detail 泄露到用户面；
- 需要修代码、apply、reseed、kill、批准卡或继续执行。

停机时不得自行重发、补卡、删卡、kill、修码、reset、clean、stash、stage 或 commit。

## 成功出口

成功只到“一组自然两回合 + 唯一 handler + 恰好一张 Pending 卡 + refresh 无重复 + chain/项目不变 + 进程/store 对账完成”。到达后立即停止，不点卡、不启动 chain。

建议新证据：

- `evidence/raw/2026-07-22-s1b-h2-r2-real-app/`
- `evidence/2026-07-22-s1b-h2-r2-real-app-pending-card-verification-v1.md`

