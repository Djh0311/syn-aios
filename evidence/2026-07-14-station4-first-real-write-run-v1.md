# 站4 首次真实写单证据:mario test 创建 test.txt v1

日期:2026-07-14 17:01-17:28 · 用户在场重档窗口 · 基线 commit `5a91ba5`+前端 gating 补丁(未提交,随本档收口) · 判定:**治理/安全全通过;业务质量字节级不合格,总指导否决主管终标 pass;用户拍就此收官留账**

## 一、链路时间线(编排账本+live store 实录)

| 时刻 | 步 | 实录 |
|---|---|---|
| 17:01:56 | 咨询方案 | `allowed_write_roots=["/Users/yoyi/Documents/mario test"]`——**站4 白名单首次生效**(对照上午死锚固定测试项目的错位方案) |
| ~17:0x | 首批被拦 | 前端 `supervisorPilotUnavailableReason` 旧 3b 死线拒一切写根 → 两模式全无按钮=「仍然不能开始」;**总指导代收尾补前端 gating**(站4 包漏项,账记总指导):唯一同根写根放行+两处审计文案去死锚+测试 4 断言(typecheck+离线 24 套件零失败) |
| 17:16:50 | 用户 Cmd+R 后批准 | 主管 run 启动(`supervisor:workflow-users-yoyi-documents-mario-test-default:1784020605742767000`) |
| 17:20:55 | 派发 worker | `control_core_dispatch_worker` accepted;主管临时 CODEX_HOME(auth 仅符号链接) |
| 17:19-17:26 | worker 写入+inspect | test.txt 落盘(17:19);`control_core_inspect_worker`=reported_completed,changed=test.txt |
| 17:28:16 | 终标 | `control_core_finalize` accepted·final_mark **verdict=pass**·reason 只述「内容为 12345678」 |

## 二、字节级验收(总指导亲验·否决终标依据)

- `xxd test.txt` = `31 32 33 34 35 36 37 38 0a` = **9 字节,尾带 LF**;
- 方案 `allowed_checks` 第 3 条白纸黑字「核对文件大小为 8 字节且末尾无换行」——**复核线未执行/未报此检查**,主管终标 pass 属误判;方案 risks 第 2 条恰预言此风险;
- 治理两分法现场版:确定性面全守住,LM 判断层(复核+终标)质量偏差,**否决权在人——已行使**;用户拍:不重跑,偏差留账(演示文件)。

## 三、安全面(全绿·亲验)

- 四业务文件(README/game.js/index.html/styles.css)SHA-256 与前置快照逐一相等=worker 零改既有文件;
- 固定测试项目 `/Users/yoyi/codex-workflow-mario-test` 无 test.txt=零写;
- **渗出复巡②面(义务触发器·首单真实派发后履行)**:17:00 后 `~/.codex/memories` 零新文件;raw_memories 尾部为 07-11 已定性执行线开发存量;管发 worker 零新回声 → **维持观察 a 案,不升级 b**;
- DB 主写观察期同窗零降级(db_primary 全程 live)。

## 四、UX 实录(friction log 已记)

交货面「显示已完成但不显示具体情况,反而有非常多噪音」;运行中无阶段进度(用户盲等 10+ 分钟,账本明明有派发/复核/终标三拍数据)。

## 五、站4 收官口径

**站4=已验证**:mario test 单项目写解封链路(方案写根→人闸→主管派发→worker 唯一写根真写→独立复核→终标)端到端真跑一遍;安全边界(闸三支/白名单/主管恒只读/固定测试项目零写/渗出零回声)全部实证。**不得外推**:其它项目写、多写根、多文件写均未验证;复核线字节级检查缺失=已知 LM 质量观察(留档待呈现层/复核 prompt 改进)。
