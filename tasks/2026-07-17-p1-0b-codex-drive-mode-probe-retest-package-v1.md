# 任务包:P1-0b codex 驱动方式选型补测(放网复测)v1

日期:2026-07-17 · 档位:**轻档·只读探针·执行线放网跑**(用户拍 b) · 执行者:执行线 · 上位:P1-0 包 `tasks/2026-07-16-p1-0-codex-drive-mode-probe-package-v1.md`(A 节探测定义全沿用,本包只写差异)+总执行计划防跑偏总则先读。

> **07-17 修订(用户拍·推荐案)**:首轮放网复测已证网通(API 401 应答),卡点=隔离 `CODEX_HOME` 无登录态。**认证改用真家登录态**——B 节第 4 条已重写,其余不变;本轮只补模型回合相关项(A1/A2 续轮/A4/A5/A6),A2 本地 discovery/git 零写证据首轮已核收,不必重做。

## 背景(为什么补测)

P1-0 上轮 A1–A6 模型传输全败,总指导核复定根因=**探针 codex 子进程困在执行线自身 exec 沙箱,外网被拦**——失败原文(整段):`stream disconnected before completion: error sending request for url (https://chatgpt.com/backend-api/codex/responses)`(A1)与 `Operation not permitted (os error 1)`(RSS 读取 `ps` 同拦);同时段(20:49–20:53)工作台派发的 worker codex 模型回合成功创建文件=同机网络通,唯沙箱之差。本包解法=**执行线以放网沙箱起**(用户开工时带 `--sandbox danger-full-access`),其余一切照旧。

## A·探什么(P1-0 包 A 节沿用,范围收缩)

- **复测**:A1(shell resume 续聊 ≤12 轮+并发)/ A2(mcp-server 对话续轮+2-3 并发实例)/ **A4(慢工具挂起 60/300/900 秒·选型关键)** / A5(转交+唤醒直证)/ A6(只读挂项目,顺带)。
- **不复测**:A3(app-server 浅探结论保留:实验性,值不值得深入=不值得现在深入)。
- 探针脚本优先复用 `/tmp/p1-0-codex-drive-probe.J7Nloe/`(a1/a2 各 probe.js、slow_mcp_server.js、handoff_mcp_server.js 现成);若 /tmp 已清,按 P1-0 包 A 节重写,工艺不变。

## B·红线(沙箱墙没了,纪律墙加倍;违者停手报回)

1. **放网只为连模型 API**:禁止利用网络下载安装任何东西(npm/brew/curl 拉包一律禁)、禁止上传任何本地内容到模型对话之外的目的地。
2. **仓库零改动**:开工/收工各跑 `git status --short` **全量落盘对比**(上轮用截断快照产生两文件误报,总指导已归属纠正——本轮禁截断,diff 全文留证)。
3. cwd 只许固定测试项目 `/Users/yoyi/codex-workflow-mario-test` 或 /tmp;**禁任何真实项目**;探针产物只落 /tmp(不入仓,回传引用路径)。
4. **认证=真家登录态(07-17 用户拍)**:内层探针 codex **不设 `CODEX_HOME`**,像工作台 worker 一样自然使用真家登录(与生产形态同构,worker 07-16 20:49 实证跑通的配方);探针脚本与执行线**禁读取/复制/打印任何凭据文件内容**(auth/token/secret);**禁改真家配置**——所有配置差异一律命令行临时覆盖(`--sandbox read-only`、`--config key=value`),`~/.codex/config.toml` 等文件零改;A2 的 `codex mcp-server` 同理真家起。测试会话历史自然写入真家 sessions=已知可接受(与 worker 同构);codex 特性(memories 等)零开关。
5. 全局 codex 配置零改;不新装全局包;报回不自拍;运行时输出整段原文,截断处显式标「[截断]」。
6. 单项卡死不硬泡(每项 ≤3 次尝试),记录现象跳下一项;总墙钟半天封顶。

## C·交付(P1-0 包 C 节照旧)

1. 实测对照表:模式 ×(每轮延迟/稳定性/**A4 挂起上限与超时报错原文**/并发/协议复杂度/接入 Syn 改动量估计);
2. 选型建议一页:主方案+备胎+理由(A4/A5 对「问你一句」实现的定量结论——挂起等答 vs 转交+唤醒);
3. 10 项回传模板(第 7 项 shape gate 三数原文,基线 13/5/5 仓根跑,本包应零变化;第 8 项含 `git status` 前后全量一致证据;第 10 项列被闸拦过的事)。
