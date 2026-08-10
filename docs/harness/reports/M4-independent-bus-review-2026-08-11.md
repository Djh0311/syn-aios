# Syn M4 独立总线复核

日期：2026-08-11<br>
复核基线：`main@7f9c6da717f0ec49c22fcd76327431fcfff0cb4e`<br>
结论：`STAGE-06 PROCEDURALLY CLOSED / PRODUCT ACCEPTANCE WITHHELD / CORRECTIVE CLOSURE REQUIRED`<br>
权限：本报告只记录事实与验收决定，不激活阶段、任务包、代码修改、真实运行、Git 提交或下游阶段。

## 1. 结论

`stage-06` 的 Git 与 Harness 生命周期已经完整关闭，M4 的存储、状态机、日报、去重、协调状态/日报持久恢复和隔离合成验收也形成了扎实底座；但独立复核发现普通产品主路径仍有五个 P1 缺口。因此，本次不接受“M4 产品阶段已经完成”的结论，M5 保持 `PLANNED / NOT_ACTIVE`。

这不是推翻 C01–C10，也不是重问产品需求。需要补的是普通产品接线和能直接证明该接线的验收。

## 2. 已直接确认的事实

- `main` HEAD 为 `7f9c6da717f0ec49c22fcd76327431fcfff0cb4e`；取证结束、写入本报告前工作树洁净，本地相对未联网刷新的 `origin/main` 为 ahead 109 / behind 0。
- 从 M3 基线到该 HEAD 共 25 个单父提交，无 merge 形状；M4C01–M4C10 和 `stage-06` 均已归档，当前无活动 stage / leaf，历史授权不再有效。
- M1、M3、M4 冻结合同及三份 C09 JSON receipt 的 hash 与报告一致。
- 新鲜复核通过：M4 定向 98/98、C09 定向 3/3、typecheck、44 个离线入口、production build、`cargo check --lib`、定向 rustfmt 和 launcher 语法检查。
- 完整 Rust 在新建独立 `TMPDIR` 下为 1639 passed / 0 failed / 45 ignored。默认临时目录重复运行会被旧 fixture 污染，属于测试隔离债务，不是本轮新产品回归。
- C09 证明的是 synthetic fixture + fake model + isolated debug App，不代表真实资料、真实模型/provider、真实消息、connector、日常使用或发布。

## 3. 阻断 M4 产品验收的五项 P1

| 缺口 | 当前事实 | 用户实际会遇到什么 |
|---|---|---|
| 普通产品来源没有接入 | 正常 `AppState` 只打开 M4 store 和日报 scheduler；生产代码没有调用 `ingest_workflow_attention_source`，C09 在进入 UI 前直接注入 synthetic source | 正常打开 Syn 时，现有内部事项不会自然进入 Secretary 的 Inbox / OpenLoop |
| 到期唤醒没有接入 | `advance_open_loop_clock` 和 `fire_reminder` 只有单元测试调用；生产 scheduler 只跑 daily cycle | “一小时后提醒”可以保存，但到点后不会自动回来 |
| 精确回源没有接通 | 前端丢弃 owner / route ref，只导航到通用 Projects 页面，目标页也没有消费 navigation focus | 点击来源只能到项目大厅，不能定位原对象 |
| 持续 Secretary 对话没有接通 | `WorkbenchShell` 输入框明确 disabled；普通产品只有固定机械解释，没有用户消息、M3 Turn 写入、历史恢复链路 | 用户不能在首页连续和 Secretary 对话 |
| 旧读面兼容仍是 inventory-only | 五类候选的 exact join tuple 全为空，普通命令全部 quarantine；PARITY 测试使用手工构造候选 | 旧数据只被盘点和隔离，没有形成可验证的真实 shadow/parity/fallback |

对应源码证据入口：

- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_repository.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/m4_acceptance.rs`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`
- `prototypes/productized-desktop-shell/src/components/WorkbenchShell.tsx`
- `prototypes/productized-desktop-shell/src/components/SecretaryBoardView.tsx`
- `prototypes/productized-desktop-shell/src-tauri/src/m4_secretary_read_model.rs`

## 4. P2、已同步项与证据上限

复核基线严重度为 P0=0、P1=5、P2=4。四项 P2 中，架构正本陈旧现状和 M5 计划“M3 尚未激活”已随本次文档同步纠正；仍需修正阶段处理或留痕的 P2 为历史写域追溯与测试临时目录隔离两项。

仍在的 P2：

- C06 曾把根文件 `src/styles.css` 加入 leaf 写域，但没有按 Harness R3 标注 `[新增]`；这是历史追溯瑕疵，不改写已归档文件。
- 完整 Rust 在默认临时目录重复运行存在 fixture 清理/隔离债务；最终再验收必须使用新建专用 `TMPDIR`，并验证清理行为。

证据上限与未知，不计入上述 P2 数量：

- C09 两张截图只留 hash，仓库没有可重新查看的像素文件；原报告的可见性声明不能代替新的可携带视觉证据。
- 远端没有联网刷新，远端当前状态仍未知。

## 5. 决定与下一入口

- M4 进入独立修正收口，不重开或改写 `stage-06`、C01–C10、C09/C10 历史报告。
- 当前修正计划入口为 [`2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md`](../../plans/2026-08-11-syn-m4-independent-remediation-and-reacceptance-plan-v1.md)。
- 拟议的新 Harness 生命周期为 `stage-07`，任务包前缀为 `M4R`；计划存在不等于阶段已经激活。
- 只有五项 P1 均由普通产品生产调用链和反例闭合证据支持，且独立复核通过，才重新判定 M4 产品阶段完成。
- 验收后只给出 M5 建议，不自动激活 M5–M10。
