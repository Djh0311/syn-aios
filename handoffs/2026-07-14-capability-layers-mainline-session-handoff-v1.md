# 对话交接:蓝图能力层主线开局+存储线收官 v1

日期:2026-07-14 · 上一手 commit:`a717064` · 本会话按纪律换代(transcript 6.3MB/2 次 compact/2 次续接——事故会话 e60d492b 同款风险面,见 `evidence/2026-07-13-session-tool-contamination-scare-forensic-clearance-v1.md` 防复发条款)。

## 0. 一句话

存储线全链收官(M5 DB 主写 live·降级网在·M5-B 批 1 核复通过);主线已翻页为蓝图能力层(L1 记忆通血已入库待真机·L2 第一刀收口);**接手者的首件事=核收 M5-B 批 2+C 回传**。

## 1. 接手者先读(顺序)

1. `CURRENT.md`(唯一活正本);2. 本交接;3. `docs/plans/2026-07-14-post-m5-stage-plan-v2.md`(排布)+`decisions/2026-07-14-blueprint-capability-layers-mainline-pivot-v1.md`(为什么翻页);4. 遇记忆/harness 层工作时读对应设计材料档(docs/plans/ 同日两份)。**除 CURRENT 外一切文档按可能过期对待;核实物优先于任何转述——包括本档。**

## 2. 树上 WIP 分账(未提交≠未完成)

- **M5-B 二线 WIP(13 改+1 新)**:c4_c6/commands/director/lib.rs/plan_auth/proposal/automation/supervisor_action/repository/chain/execution/run_dispatch/lifecycle + `workflow_db_primary_wiring.rs`——批 1 已核复通过,**批 2+C 在做,收口时批 1+2 一起 commit**;
- **§7.3 七件照旧不动**:`.claude/`、`.playwright-cli/`、`docs/research/` 三稿、两 prototype 目录;
- 其余全部已入库(HEAD `a717064`,与远端 0 0)。

## 3. 在飞三件与核收口径

### 3.1 M5-B 批 2+C(二线·核复已发)

批 2 范围**已钉死**(核复原文,不许漂):lifecycle init(:76)+bootstrap(:170)｜offline 三件(wee:1383/:1504/:1603)｜permission_decision(wee:1228)｜store_hygiene(:277)｜operation_control(:374)｜pilot 回填(launcher:793)｜abandon(mcp:691)｜director 会话出生+role_loop(:3543/:3893)｜**c4_c6:908(包漏点·总指导已认账补入)**。C=reconcile 扩批 1+2 全写面+replay/projection/freshness+**m5a fixture 同步(不同步=七连测假红)**。
**回传核收清单**:第 7 项必含 gate 三数(缺/冒充=机械打回,ledger M-2026-07-13);豁免面反向 grep 亲验(生产 `write_validated_workflow_state(` 直调点应只剩:六流 M5-A 分支/桥自身/storage_mode 反向/测试区);automation 水线 5059 不许破;fmt 仅历史三(codex_db/codex_local_runner/mcp/storage);m5a 全族绿;真实根零写;红线(storage-mode 语义/迁移面/安全闸/read_cut)零碰;收口 commit=批 1+2 全部文件显式列名(共树禁 add -A)。
**桥设计口径**(批 1 已核形):`write_m5b_batch1_workflow_state`=mode-off 原样/mode-on DB delta+审计同笔 Immediate→JSON 投影;批 2 须同款显式隔离(防 flag 静默扩面)。

### 3.2 L1 记忆通血=已入库(`a717064`)·待真机

用户重启 App 那一次,**一口气验三样**:①记忆环四步(候选出现→inbox[确认属实/采纳]→记忆中心见正式记忆→下一单召回带上);②L2 双板活数据(58 项目/90 技能,索引已刷);③M5-B 接线在真数据上跑(降级网兜底:最坏=自动退回 json_only+一条 `storage_mode_degraded_json_only` 审计,零损失,恢复=重 seed)。
**义务触发器(用户拍过,不可漏)**:真机走环后的**第一单真实派发**完成时,立即做**渗出复巡②面**(store 反查;三查口径=`docs/evidence/2026-07-11-memories-leak-triple-check-rerun-v1.md`;②面出现工作台条目回声即升级后手 b=worker argv 禁 memories,需用户拍)。

### 3.3 L2 第一刀=已收口(`db24519`)

词表草案 `decisions/2026-07-14-skill-harness-vocabulary-draft-v1.md` **待用户拍板**(不挡事);第二刀=登记机器(候选→用户登记→harness_resources/capabilities 事实),前置=M5-B 落地。

## 4. 验证口径(接手即用)

- 全量基线:**925/45**(批 1 树上口径;每收口只增不减);
- **计时 flaky 家族三成员**(间歇挂·与功能无关):codex_local_runner `real_process_timeout_kills_and_reaps_mock_child`(部分修复·残余竞态)、manual_relay `gui_direct_running_poll…`、manual_relay `app_shutdown_kills…`(07-14 新登记)。处置=solo 复跑即准;**连续 solo 挂再升级**;二刀收敛包挂账(同款唯一 temp+轮询模板);并行编译/App 运行会放大发作率;
- shape gate 基线 **14 errors/5 warnings/5 info**(名单固定:5 ratchet+5 over-limit+5 unknown-sidecar);fmt 恰 3 历史文件;`cargo fmt` 全 crate 禁跑;rustfmt 单文件先 grep 无外部 mod 再裸跑(新版无 --skip-children);
- shape gate 必须在**仓根**跑(src-tauri 下跑=假 0,本会话栽过两次)。

## 5. 阶梯(收口后的路)

**维稳轨**:批 2+C 收口 → 用户重启验三样 → 观察期(每会话 reconcile 巡检全绿攒天数) → **M6 停写 JSON**(用户授权窗口;包含 SQLite 备份纪律[份数+字节预算]+正文外置正式判废+production-db 异地备份补齐)。
**主线**:L1 真机过 → 记忆层第二刀候选=板 1b 最小共享 filter(材料档);密度闸/词表理清等真淤积;L3 知识库=设计谈话先行(绿地不先建);**L4 agent 层红灯**(记忆/skill 成型前不动)。
**前端**:搁置(用户拍);交互正本提案+friction log(`docs/ux-friction-log.md` 已开档)攒实录。

## 6. 边界与忠告

- 高危五条照旧(尤其 push=每收口已成惯例但 commit 仍问一次);执行线不 commit;显式列文件;
- 三方共树时 gate/fmt 噪音先分账再定性(本会话两次差点误判——查 `git status` 按文件归属人);
- 执行线回传三犯史:第 7 项 shape gate(已 ledger 加硬机械打回);先核步二犯史(M-2026-07-11);**总指导也被抓过**(越权代杀进程/包坐标漂移×2/「已修」虚高)——催核实物文化对内对外同效;
- 别跑巨型会话:本档存在本身即执行该纪律。
