# M5C01 stage-14 closeout 与证据报告 v1

日期：2026-08-18

状态：`M5 SCOPED PRODUCT-CHAIN PASS / STAGE-14 CLOSED / NOT_RELEASED / M6_NOT_ACTIVE`

## Harness

- 最新独立结论 `M5R09-20260818-1836.verdict.md` 放行 M5 产品内容锚 `c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`、M5R09 记账 `8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0` 与本 leaf 的生命周期收口。
- M5R09 lifecycle opening `b2429f63e5167fc834b967d6e7d2f7a6894eae21` 只归档 M5R09、建立唯一 M5C01；M5C01 内容候选为 `de98d69a363ff82281330fb3b82de82c03a9b484` / tree `b90244a8535c829e96341d42fef39602ef499f6d`，parent 精确为 `b2429f6`。
- M5C01 候选精确 5 路径：3 个 unfinished 路由、1 份 closeout 输入报告、1 份 M5→M6/新壳交接更新。零产品源码、合同、构建配置或用户 OSS 门面变化。
- REC-00、M5R01–M5R09 唯一位于 `done/2026-08/`；M5R00 按其独立 verdict 明确的旧惯例唯一位于 `archive/`，未为目录统一而重写。M5C01 与 stage-14 在本记账中归档；authorization 保持精确 closed 两字段；stage-15 未建立，M6 未激活。

## 产品

- M5 在已接受范围内形成持久 Project Supervisor、完整授权/执行/回执/独立审查/结果决定链、恢复与 duplicate-effect、ProjectSummary/QueryPort、普通 Tauri command graph，以及显式 M1 enrollment 生产入口。
- M5R09 verdict 的 8 项非阻断欠账已按用户 18:40 纪律路由：用户 OSS 载体单列；canonical ProjectId 扩面与 relation owner 类型化进入 `M6P00`；`UNENROLLED` 主动引导进入 F3 unfinished；测试 helper、死代码、warning 和历史 worktree 卫生进入 `ENG-01`。没有第四个加固 leaf，也没有把记录项冒充已修复。
- M6 输入交接精确指向 ProjectSummary 合同/QueryPort、完整 execution identity envelope 与 `new-grant / guarded-legacy / blocked` compatibility/rollback 边界；现有未跟踪 `m6_*.rs` 不被采纳。

## 证据

原始证据根：`/home/synadmin/workspace/.syn-gates/evidence/M5C01-de98d69/`

| 检查 | 结果 | exit |
|---|---|---:|
| candidate SHA/tree/parent 与 5 路径 stat | 精确匹配 | 0 |
| `git diff --check b2429f6..de98d69` | 无输出 | 0 |
| candidate write scope 与 5 路径白名单 exact diff | 一致 | 0 |
| 唯一 current M5C01、活动 stage-14、无 stage-15 | closeout 候选时成立 | 0 |
| REC-00、M5R00、M5R01–M5R09 生命周期唯一性 | 按各自放行目录唯一 | 0 |
| stage-12、D0C04/D0C05、OSS-01 相对 `c1025ba` | 零差异 | 0 |
| authorization | 38 bytes，逐字节 closed 两字段 | 0 |
| `docs/contracts/` 相对 M5R09 接受记账 `8e6f59f` | 零差异 | 0 |
| 用户 OSS `c1025ba` 精确 7 路径与后续差异 | 7 路径匹配；M5C01 零差异 | 0 |
| M6 输入文件/合同存在性 | 全部存在 | 0 |
| M5C01 候选 product path diff | 零路径 | 0 |
| 静态 WIP exact SHA-256 | 30/30 `OK` | 0 |
| `commands.rs` 候选外残余 | 59+/56-，保持 | 0 |
| `.syn-gates/open/` 写前 | 0 个文件 | 0 |
| 本叶 detached worktree 移除 | 精确目录与注册项均不存在 | 0 |

第一次证据脚本因主管手抄错误的 `b2429f6` 完整 SHA 导致 4 个 range 检查失败；第一次 lifecycle 重跑又因同名 M5R00 report 被误计为 leaf 副本而失败。这些原始 `.log/.exit` 均保留。用 Git 实测 base `b2429f63e5167fc834b967d6e7d2f7a6894eae21` 和只限 `archive/leaves/unfinished/done` 的生命周期目录复跑后，最终检查全部 exit 0；`summary.txt` 明确保留 `initial_overall=1`、`rerun_overall=1`、`final_overall=0`，没有覆盖或伪造红灯。

产品回归没有在 M5C01 重复执行，因为 `b2429f6..de98d69` 零产品路径变化。直接复用最新独立验收官对 `c91d8fc` 的 detached 新鲜复跑：cargo check 0；`m5r09_` 23/23；memory/mature 各 14/14；ordinary source 4/4；完整 `m5_` 188/188；typecheck/default build 0；默认 bundle gate 和两段 diff check 符合预期。其原始主管 evidence 保留在 `.syn-gates/evidence/M5R09-c91d8fc/`，独立复跑命令与边界记录在 verdict。

## 载体

- M5 产品内容：`c91d8fc72bcbf80186736caff841cb7a9b0660d1` / tree `fe2d982267d474631ca4ea7b3f90ed846f72a89d`。
- M5R09 接受记账：`8e6f59f48d2d90891d3c02396378921e4a2f5d6e` / tree `2043660c9547c6c102ae24414674918ca8215eb0`。
- 用户 OSS 门面：`c1025ba81b6c7885a16529b8f66c919655db48e4` / tree `f60a315ff743ebb24eea192378c388ea277bda75`，独立 7 路径，不属本叶。
- M5R09 → M5C01 lifecycle opening：`b2429f63e5167fc834b967d6e7d2f7a6894eae21` / tree `ab385650ddc3f887f9fcea7668f582f4cb3c58ab`。
- M5C01 closeout 内容：`de98d69a363ff82281330fb3b82de82c03a9b484` / tree `b90244a8535c829e96341d42fef39602ef499f6d`。
- 最终 lifecycle 记账 SHA/tree 由包含本报告、权威状态同步和 stage/leaf 归档的提交形成，并写入仓外节点请求；报告不预填自引用 SHA。

证据上限：Linux x86_64 WSL 的 detached/local/synthetic/ordinary Tauri 组合与静态调用图；不是发布、真实用户资料、真实 provider/账号/凭据、外部业务写、macOS/BSD 实机、真窗口像素、新壳运行或长期真实日用。
