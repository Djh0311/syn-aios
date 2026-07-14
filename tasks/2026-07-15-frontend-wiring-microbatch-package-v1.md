# 任务包:前端接线微批——A/B 上脸+两条已拍未落 v1

日期:2026-07-15 · 档位:**轻档**(纯前端接线+文案落拍;后端零改·安全面零碰) · 执行者:前端施工对话 · 背景:②-⑧ 总包与后端读模型包均已总指导核收(见 CURRENT §三.4 07-15 段);A/B 两命令后端已挂载但前端零引用,另有两条已拍未落。

**先读**:`CURRENT.md`、`prototypes/productized-desktop-shell/DESIGN.md`、定稿 `docs/design/2026-07-14-stage-b-hifi-fullapp-v1.html`(C 首页/D 审计账本两段)、`handoffs/2026-07-15-backend-ui-support-readmodels-handoff-v1.md`(§1 两命令的精确出入参形状+两字段合读注意)。

## W1·系统状态接 A(首页+顶栏)

`src/lib/tauri.ts` 加 `load_system_status_read_model` 包装(类型照 handoff §1·A 形状)。首页「系统状态」区块接真数据:`storage_mode`/`observation_day`(第 N 天·0=没进观察期)/`last_degradation`(无=不渲染该行)/`gate_summary`/`warnings`(有则人话显示·软着陆不断面板);`recent_catches` 现阶段恒空=空态按定式。**顶栏健康点看 `storage_healthy`,不看 `storage_mode`**(降级态=`db_primary`+`healthy:false`,handoff 明写两字段合读)。

## W2·审计账本页接 B

`tauri.ts` 加 `query_audit_ledger_read_model` 包装(handoff §1·B 形状)。`AuditLedgerView` 换 B 为**主源**:分页(page_size 50)+类型过滤下拉直接用返回的 `kinds`+`total` 显真实总数+`human_summary` 主显+`raw_json` 进下钻(定稿 D 段:机器细节下钻不上卡面)。现有三源(工单账本/运行日志/健康诊断)与 B 的关系照定稿 D 段判:与 B 审计流重叠的去重、不重叠的(如运行日志)保留为并列类;**有含糊报回别自拍**。

## W3·两条已拍未落

1. **发令台(command-console)从导航下线**(07-14 拍板:宪法第二入口红线·不走补齐路线)。导航项摘除;view 文件先留、顶部加「已拍下线·待退役」注(死码处置另案)。先 grep 测试引用,牵连超预期(>30 分钟)→停下报回。
2. **导航「运行器」label → harness**(07-14 词表拍板:产品域 UI 名 harness 不译·译名废止)。只改 UI label,代码标识符/文件名零动。

## W4·卡住乙型=甲已裁(07-15 用户拍)

「直接回它一句」回话框**维持 disabled 占位**,不通电(丙=真接 worker 生命周期,将来单独立包)。只核对占位文案是人话(「通道接线中」类),零新做。

## 红线与纪律

照定稿零设计决策,有含糊报回;每步 `npm run typecheck`+`npm run test:offline-interaction` 全过;断言只许因旧语义锁死而更新、不许删;无 hooks 约定(状态提升父组件);`src-tauri/**` 零碰;不 commit;完成回传总指导核(shape gate 三数仓根跑,基线 13/5/5)。
