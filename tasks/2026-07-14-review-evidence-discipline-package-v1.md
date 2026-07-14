# 任务包:复核实证纪律——终标只采实证·复核须交字节级证据 v1

日期:2026-07-14 · 档位:**轻档**(advisory 质量强化;不碰解封闸/S1/path-lock/沙箱) · 案发:站4 首单(`evidence/2026-07-14-station4-first-real-write-run-v1.md`)——worker 写出 9 字节带 LF,复核漏执行方案 checks 的「8 字节且末尾无换行」,主管终标以字符串口供判 pass,若无总指导人肉 xxd 用户将不知情收下错误交付。**「可信验收」是产品核心承诺,本包治它。**

## 病根(总指导已勘·坐标为 HEAD `fdc1851`)

`mcp/supervisor_orchestrator.rs::final_mark`(:1198)仅校验 `verdict ∈ {pass,needs_rework,blocked}`+reason 非空——**pass 无需任何证据**;inspect(:1016 区)回程 `acceptance_status` 亦为 worker 口供。方案 `allowed_checks` 从授权可取(`active_authorization`)但终标全程不看它。

## 目标(两层,机械层为主)

**A·final_mark 确定性证据闸(主刀)**:`verdict=pass` 时新增机械前置——本 run 的 workers 中须存在**只读复核 worker 的实证块**,且实证块覆盖授权 checks 中的字节级/尺寸类标准;缺失或不覆盖 → **拒绝 pass**(返回人话错误给主管:缺什么实证、去补复核;不自动改判,主管可改交 needs_rework/blocked)。实证块=结构化字段(建议形态:`evidence: {path, byte_count, sha256, trailing_newline: bool, read_method}`,执行线按现有 worker 回程结构定形状,原则=**机器可核对的数字/哈希,不是自然语言**)。

**B·复核 worker 派发要求+主管终标指令(prompt 层)**:主管 prompt 中「派发复核 worker」处加结构化实证块要求(输出该 JSON 块);「终标」处改为:pass 必须引用复核实证,禁止以执行 worker 口供或「内容一致」类字符串断言作 pass 依据。主管 prompt 构造在 launcher/orchestrator 侧,执行线勘察定位。

## 范围与红线

- 改动面预期:`mcp/supervisor_orchestrator.rs`(final_mark+inspect 回程结构)、主管 prompt 构造点、必要的回程解析;**解封闸(station3b/4/S1/path-lock)、沙箱、storage-mode/M5 面、L1 记忆区零碰**;
- checks→实证的覆盖判定用**保守白名单**:仅识别「字节/大小/换行/哈希」类 checks 需要实证块;其它 checks 不阻断(防误伤只读单/非文件类单);**只读单(零写根)finalize 行为不变**(站3b 回归案发测试);
- advisory 哲学不变:闸只拦「无实证的 pass」,不代替主管判断;人闸(用户终决)零碰;
- 不 commit;fmt 仅历史三;gate 14/5/5 零净增;automation 5059 不破;全量基线(942/45)只增不减。

## 验收(预写死)

- 案发测试复现今天:mock 复核回程无实证块 → final_mark pass 被拒(人话含缺失项);补实证块(byte_count=9,trailing_newline=true 等)→ pass 允许(证据在,判断归主管)/ 且实证内容与 checks 冲突时错误信息如实呈现;
- 站3b 只读单回归:零写根 run 的 finalize 行为一字不变;
- temp 端到端:派发→复核(带实证)→finalize 全链绿;
- 回传 10 项模板,第 7 项 gate 三数必含。

## 交货面(不在本包)

「已完成+噪音」的呈现层问题归 friction log 前端族(记忆中心/方案一大坨/交货面三件同族),另拍。
