# M5R07 项目 UI、隔离 App 与阶段候选报告

- 日期：2026-08-16
- 阶段：stage-14 / leaf M5R07
- 状态：**`AWAITING_INDEPENDENT_ACCEPTANCE`**
- 不宣布 M5 完成，不关闭 stage-14

## 候选载体

| 项 | 值 |
|---|---|
| Candidate tip（实现） | `20740a8654ddddea08717800d9be0536d4b0021d` |
| Series | `93ba9b0` M5R02 → `6b252a3` M5R03 → `177399d` M5R04 → `a5d93e8` M5R05 → `867fd20` M5R06 → `20740a8` M5R07 |
| Receipt | `docs/harness/reports/M5R07-isolated-acceptance-receipt.json` |
| Receipt SHA-256 | `acf536ec7cb903e6cd9a912898b4c31e23e8e23f6e088068ae05458533b9b86e` |

## 隔离场景

- scratch-a：只读 chat + 用户留下 DRAFT proposal，零 Grant / 零 spawn
- scratch-b：批准后 echo 白名单 → Syn-native workcell → RuntimeReceipt → 独立 Review → ResultUserDecision → fact → summary → 只读 global advice fixture；重放不产生第二份 claim
- 窗口截图 / Computer Use / 真实 Tauri 交互：`NOT_EXECUTED`（不得写成 PASS）
- 旧入口：`RUN-006` blocked，其余 guarded-legacy，未物理删除

## 新鲜验证（disposable checkout `20740a8`）

- `cargo check --lib --offline`：PASS
- `cargo test --lib --offline -- m5_`：80 passed / 0 failed
- 完整 `cargo test --lib --offline`：不宣称 PASS（既有非 M5 测试在本环境失败：conversation transport、process reaper、fix9）
- TypeScript typecheck / production build：`NOT_EXECUTED`

## 交给总线

- Git：本地 `main` 相对 `origin/main` 超前；未 push
- Harness：唯一 current leaf = M5R07；authorization closed；stage-14 仍开
- M6 / D0C04 / D0C05 / M1–M4 冻结合同未动
- 请总线只读复核 exact candidate SHA `20740a8`
