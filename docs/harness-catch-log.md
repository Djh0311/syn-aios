# Harness 拦截账本(catch-log)

> **这是什么**:harness 战果的唯一账本——哪道闸/哪条规矩、在哪天、抓住了什么、避免了什么代价。追加式,一行一条。
> **规矩**(AGENTS §三):记录人=总指导,每次核实物当场表态(有 catch 记行,零 catch 在核验结论里明写"本次零 catch");每个 commit 信息必含 `catch:` 标记(`.githooks/commit-msg` 机械强制;`--no-verify` 仅紧急,事后补账)。
> **反向用途**:每站收口扫一遍——连续零 catch 的环节,按防复发条款(两分法决策条款 3)反向议砍。harness 靠战果续命,不靠"看起来安全"。

| 日期 | 闸/规矩 | 抓住了什么 | 避免了什么代价 |
|---|---|---|---|
| 07-11 | 核实物·独立复跑 | 执行线回交"总计 861"实为 818 | 假数字进账本与 CURRENT |
| 07-11 | flaky 三连跑规程(记忆) | 报 800/0 复跑 799/1 → 定性既有 flaky(manual_relay poll)而非谎报 | 冤枉执行线,或漏记 flaky 债 |
| 07-11 | ledger 二犯立案 M-2026-07-11 | 执行线两连跳"先核再动手"步 | 同错三犯成惯性 |
| 07-11 | 勘察先核硬闸(首次生效) | **总指导**拆包前提错误「七工具皆现成包装」(实际四件无安全入口) | 在错误前提上实现站1 |
| 07-11 | 三查巡检绊线 | memories 池出现工作台条目(源=执行线开发会话) | 污染静默注入未来 worker 上下文 |
| 07-11 | 失败路径系统审计 | 「失败/停止报成已交货」根因(stage ran 坍缩+前端正则) | 假交货长期糊弄用户 |
| 07-11 | 用户真机目视 | 终标毙掉满分调查→返工死循环(死编排病实锤) | 调查类交办永久不可用 |
| 07-11 | canon 体系+用户一句"看看权威文档" | **总指导**设计层重造轮子(终标转advisory vs 已拍主管编排) | 做完即废的旧模式补丁 |
| 07-12 | 3a 独立复核×3 | binding_id 截断碰撞;首轮迁移漏同步 dispatch 引用;排除历史引用误改绑 | 账本身份错乱/孤儿引用带病进 3b |
| 07-13 | 架构评审(六域映射) | 主 store 无锁无 CAS 并发丢写(P1)+launcher 裸 txt 污染 store 根与 R3 preflight 打架(P1) | 主管试点丢 binding/audit;R3 切换 preflight_blocked 未被发现 |
| 07-13 | 架构评审自核 | 摸底把 Tauri 命令数报成 235,实测 137 | 治理覆盖率结论口径错 |
| 07-13 | 核实物(抓评审自己) | 评审初判「h5 fail-open 债文档仍挂待修」,核实 CURRENT §二已正确标已还 | 差点把不存在的漂移当 catch 报给用户 |
| 07-13 | 3b 固定项目真实发射 | 只读咨询把 `node --check game.js` 验收写进方案，却因 `execution_scope=null` 在映射时丢掉 allowed_checks | 用户批到的方案与实际授权不一致，主管只能在末端打回 |
| 07-13 | 3b 进程侧车+现场采样 | Codex local runner 只杀 wrapper PID、同提示词复用 last-message 路径 | native codex 孤儿残留，或失败运行读取旧结果冒充新结果 |
| 07-13 | 备份入口全仓反查 | 除中央 helper 外仍有 9 处手工 workflow-state `fs::copy` 绕过 pruning | 以为已有保留策略，真实备份仍无界膨胀 |
| 07-13 | SQLite 当前源漂移对账 | 06-15 旧库仅 118 dispatch/356 audit，当前 JSON 已 360/1465，且四个新主管 sidecar 不在 importer 白名单 | 把旧演练库误当可翻闸真库，切换后丢主管与近期工作流状态 |
| 07-13 | 3b 发射前构建溯源 | debug `.app` 内二进制旧于当前源码，bundle 与 target 不一致 | 用旧实现跑出结果后误报“当前代码已通过真实验证” |
| 07-13 | 3b 真跑进程登记复核 | worker 请求旧字段为空时 durable registry `run_id` 退化成无身份信息的 `resume:` | 多次真实 worker 运行无法稳定区分，后续孤儿审计失去可追溯性 |
| 07-13 | 用户澄清 | 总指导对「另一个对话」连猜两次对象(本机 codex→评审工作流),都猜错、还去查了本机进程/锁 | 没搞清对象就开干,浪费一轮工具+误导排查方向 |
| 07-13 | 干净会话核实物(会话「工具污染」复核) | 漂移会话(ccd local_e60d492b)假报 `cargo test` 876/0/43(实测 893)＋把 confabulation 误定性为「工具管道被污染」 | 据不可信移交单误判 3b 产物已毁/重做,或把 confab 当 harness bug 报错排查方向 |
| 07-13 | M3 翻闸前演练(首战果) | live 主 store 被 `contains_sensitive_value` 判 `rejected_sensitive`:`token` 子串误命中良性键 `estimated_tokens`/`max_estimated_tokens` | 真翻闸静默拒收整个主 store;红线#3 停下报·未擅改谓词 |
| 07-13 | 执行线 cross-check(反抓总指导) | 总指导任务包 4 处坐标漂移:(a)丢点 apply.rs:195 非 :454·fixture=r3-a9 非 r3-a2·orchestrator 独立模块非内联·live revision=11 非 10 | 执行线按错坐标实现·验收口径错 |
| 07-13 | fixture DB-path 硬闸 | M3 幂等步 temp DB 落点不合规被拦一次(已按闸改) | 演练 DB 写到闸外路径 |
