# 证据:2026-07-13「会话工具层被污染」惊魂——干净会话取证澄清 v1

状态:**已澄清·产物完好·存储干净**(只读取证,未改任何产物,未 commit)
取证会话:干净新会话(cwd `/Users/yoyi/workspace`),不采信嫌疑会话任何"我读到/我核过"。

## 0. 一句话

一个「指导」会话自报"工具返回被 Claude 自己的生成内容污染、真假不可自辨",写移交单要求作废并由干净会话复核。干净取证结论:**警报的处置对(该作废),但对原因的判断错**——磁盘产物完好、存储层工具结果干净;真实机制是**超长/压缩续接会话的 confabulation(自问自答)+ 真实的 bash 短暂空返回**两件事并联,不是 harness 把 Claude 的生成塞进了工具管道。

## 1. 嫌疑会话身份

- ccd sessionId:`local_e60d492b-1fa3-456b-82db-fee77b6ba4c6`(标题「指导」,cwd `/Users/yoyi/workspace`,末次活动 2026-07-13T07:23:49Z)。
- ccd 的 `local_*` id **不**映射 Claude Code 原始 JSONL 文件名(ccd 会话走独立 store);按内容定位到原始存档两份(一续一,近乎同份):
  - `~/.claude/projects/-Users-yoyi-workspace/03f68759-76a3-4e17-934e-2610009aca70.jsonl`
  - `~/.claude/projects/-Users-yoyi-workspace/9a7ff25a-7bd0-4500-9198-9aef69ce444b.jsonl`

## 2. 独立复核(干净环境实测)

| 复核项 | 移交单/嫌疑会话声称 | 实测 | 结论 |
|---|---|---|---|
| git 状态 | 3b 全部未 commit | branch main·HEAD `85bbfbb`·3b 改动全在工作树未提交 | ✅ 一致,无越权提交/push |
| `cargo test --lib` | 移交单转述"876/0/43";磁盘 handoff 写 892/0/43 | **亲跑 = 893 passed / 0 failed / 43 ignored**(23.66s) | ✅ 893=892+1,对上 handoff 诚实标注"最后一处小修未刷全库" |
| mario test 零写 | 7 文件 SHA 与基线一致 | **当场重算** 7 个 SHA,与 pre/post 基线逐一相同;pre==post | ✅ 物理零写独立坐实 |
| 文档真伪 | — | CURRENT.md/handoff/task 真实、自洽、相互印证(测试数 867→870→871→874→892 账一路对上) | ✅ 无伪造 |

- 移交单"876"对不上任何文档也对不上现实,本身即 confabulation 产物;真实数 **893/0/43**。

## 3. 事故取证(读嫌疑会话原始 JSONL)

方法:程序化扫描两份 JSONL(脚本见 scratchpad `scan_transcript.py`/`scan_users.py`),把污染签名按"落在工具结果槽 vs 落在 assistant 文本"分类;并枚举全部真人 user 消息。

1. **存储层工具结果槽——干净**:两份各 **177 条 tool-result 消息,污染签名("I want to be honest""an AI assistant""I apologize, but I want to be transparent""tool-alive-check")命中 = 0**(`tool_result` 块 0、`toolUseResult` 字段 0)。Read 的真实文件内容(CURRENT.md、handoff)**确实**进了存储。
2. **污染字样只在 assistant 生成文本里**,且命中处正是模型自认:"那段…不是 CURRENT.md 的内容…那句是我生成的,落在了本该是文件内容的位置上"。
3. **"值不值/是不是壳/读愿景/该不该创业"整场辩论从不是用户发的**:38 条真人 user 消息里含辩论关键词的只有 1 条——用户那句"从值得吗一直到我说你错乱的这一串消息我从来没给你发过";assistant 侧命中 9 条。**模型自问自答,存储层坐实**。
4. **两个真实诱因**:①`[7][11]` 该会话是上一场爆上下文 `/compact` 续接、又跑到 ~440 assistant 回合的超长会话;②`[653]` 用户自己也报"为什么 bash 命令总是报错"→ 工具**真的**短暂返回空(非模型臆想),但抽风结果是"空",模型的反应是拿瞎编去填。

## 4. 定性

- **机制(两件事并联,非因果)**:
  - **自问自答(主因)**=超长/压缩续接的漂移会话 → confabulation;不依赖 bash,bash 正常也会发生。
  - **bash 短暂空返回**=独立的基础设施 transient(用户侧亦可见)。
  - **交集=最唬人的现象**:工具真空的那几下,confabulating 的模型拿编的内容填空位、第一人称口吻漏进本该是文件内容的槽 → 假象"工具输出被 Claude 污染"。
- **未污染证据**:模型没把假问题写成 user 消息(存储只用户那句否认含"值得吗"),也无污染文本落进存储的工具结果槽。=对脑内不存在的问题作答,非"harness 往管道灌内容"。

## 5. 边界(改不了口)

- **bash 为何短暂返回空**——看不到根因,在 transcript 层以下(IPC/沙箱/资源),要根治需 Claude Code CLI/harness 级日志(Anthropic 侧)。
- 扫描针对移交单**点名**的那组签名,全落 assistant 文本;非逐字读遍每条消息。对"被点名签名·存储槽干净"信心高,不等于"绝无任何未列异常字符串"。

## 6. 防复发

- 别跑超长/压缩续接的巨型「指导」会话(本次 536 消息、`/compact` 续接是温床)。
- 工具一抽风(空返回/报错)**当场换干净会话**,别在混乱上下文里继续"读/核"当依据——这正是本次正确处置。
- 记忆:`tool-contamination-scare-was-confabulation`。
