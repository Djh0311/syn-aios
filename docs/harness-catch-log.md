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
| 07-13 | 同根幂等回归测试(M4·R5) | 初版 R5 metadata 刷新破坏「同 root 重复导入零新增」被回归拦下→改为同 hash Ok(0)·刷新 root 才替换 | metadata 语义带病进 M5 对账 |
| 07-13 | M3 演练×natural-key conflict 闸(第二战果) | 谓词误报清除后 live 主 store 暴露 audit_events 16 组重复 event_id(51 条·内容确不同·id 可见截断+同毫秒)＋总指导复核加抓 4 条 event_id=None——坐实评审§五 stable_id 截断病预言 | 带撞号/无号审计史静默进 SQLite·翻闸后对账假成功 |
| 07-13 | 法证包纠总指导 | 「4 条无号」实为旧代 schema(audit_event_id/event_type)且 importer 已 fallback 兼容(importer.rs:41)——非阻断;总指导上轮定性偏差 | 把非问题当阻断项·误导修复范围 |
| 07-13 | 总指导复数纠法证 | 法证 D 报 active audit_refs=141,实测 148(漏 formal-memories 2/memory-candidates 3/observations 2);撞号引用=0 结论不变 | 引用面清点不全(本次无决策影响·养成互核惯例) |
| 07-13 | 共树纪律(抓总指导)+WINDOW_START_HASH 闸正面验证 | a 窗口收口时总指导目录级 `git add evidence/raw/...` 盲提交执行线并行草稿 `repair_audit_events.mjs`(同包双线并行·事后核清:未执行·输出件全无·apply 前 hash 断言通过=盘上仅总指导手术) | 未经查验文件混入 commit;若无 hash 闸,同包双写可互踩——evidence 目录也必须显式列文件 |
| 07-13 | 执行线双修复器检测(硬停正确) | 执行线察觉 backup-before+第二份修复脚本并发出现→窗口前硬停·未写 live·请求仲裁;事后对账:其临时副本演练(51+4·M3 PASS)与总指导实产逐项一致=收敛双验 | 同包双线双写互踩;「止血已派」实为 a 包名字混淆·止血从未开工被此番对账揪出 |
| 07-13 | shape gate(抓止血包)+回传第 7 项缺口 | 止血把 project_workflow_automation.rs 顶破水线 1 行(5060>5059),且回传漏报验收必填的 shape gate 数——总指导补跑抓获·代收尾纯整形至 5059(use 并群/rustfmt 定稿/宏内并行·零逻辑) | 棘轮静默增长;验收项漏报成惯例=gate 白设 |
| 07-13 | shape gate sidecar 字面量扫描(抓总指导) | preflight v2 两个测试 fixture 文件名(`*.v1.json` 形)被判 unknown sidecar 种类(+2 error)→改名去 `.v1` 收平·测试逻辑不变 | 测试字面量混进 sidecar 种类清单·gate 基线被噪声顶高 |
| 07-13 | shape gate sidecar 字面量扫描(M5-A·同日第 2 次立功) | storage-mode.v1.json 字面量+1 error——包红线#4 预声明·按 gate 自己的要求「用户确认+决策留痕」入基线注释收平 | 运行时件字面量无痕混入 sidecar 基线 |
| 07-13 | 回传第 7 项漏报**二犯**(止血包+M5-A) | 执行线连续两包漏报验收必填的 shape gate 数,均由总指导补跑抓获(两次都补出 +1/+2 真问题) | 二犯已进 mistake-ledger;此后回传缺第 7 项=回传不完整直接打回 |
| 07-14 | M5 窗口步 0 pgrep(立功) | 用户以为 App 已关,pgrep 抓到 `cargo-tauri dev`(PID 2691)仍活→按协议停·交用户亲手清 | 带活写进程切库=撕裂风险 |
| 07-14 | auto-mode 权限分类器(抓总指导) | 总指导越协议代杀用户会话外进程被拦——协议明写「有即停」非「有即杀」·拦得对 | 总指导越权开先例;人闸哲学对内同样生效 |
| 07-14 | 回传第 7 项**三犯**(降级补丁包·错报 git diff 为 gate) | 执行线第 7 项答非所问,gate 三数未跑未报;总指导代跑=14 零净增(本次干净)但三犯坐实 | ledger 加硬:第 7 项必须原样含 Errors/Warnings/Info 三数·缺或冒充=机械打回不核收 |
| 07-14 | 总指导复核(自我纠错·撤虚高) | 昨判「竞态测试已修」被更长采样推翻——solo 五连发 1 挂=残余第二竞态;改判部分修复(temp 撞车半根除);执行线「隔离仍失败」属实还其清白 | 「已修」结论的采样量要配得上 flaky 概率;二刀候选挂账 |
| 07-14 | 层级开工盘点制(两连发战果) | 记忆层与 harness 层**钉板均落后代码**——生命周期九操作/协议字段+运行检查+完成闸,全都已建且真用过,钉板仍写「待建/没建」;盘点先行两次拦住按旧钉板重造 | 重造已存在的机器(四角色学费第二遍);Phase D 排期口径全错 |
| 07-14 | m5a 案发测试×C 面扩表(M5-B 批2·执行线自查) | 全量首跑检出 M5A supervisor fixture 只 seed audit、漏 seed agent_adapters→改复用完整 generic delta seed;总指导核收复跑 m5 族 15/15 证实 | fixture 半 seed 假绿·20 表扩面对账带病过闸 |
| 07-14 | 启动对账×Blocked 留痕降级(首次真实开火·立功) | 用户真机验收重启:对账抓 workflow_audit_events JSON 领先 2 条(00:51:45 M5-B 上线前的 `auto_dispatch_scope_checked` 未接线写)→按设计降级 json_only·审计人话留痕·数据无损 | 未接线写静默漂移进观察期·M6 前对不上账才炸 |
| 07-14 | 降级根因追查(抓 M5-B 勘察漏) | 勘察「62+9 全写面」漏两族 sidecar 写:`inspect_auto_dispatch_authorization`(plan_auth:677·正是降级肇事写点)+`supervisor_orchestrator::update_store` 家族 13+ 点(worker-report/final-mark/pilot/dispatch/follow-up)——反向 grep 只盯 `write_validated_workflow_state` 抓不到 `write_store_atomic`/`update_store`;CURRENT「全走显式桥」措辞按勘察口径限定·补遗包挂账 | 重 seed 恢复后一走批准流/真派发即再降级;DB 侧主管账本静默 stale 到 M6 才暴雷 |
| 07-14 | 包内红线「不擅自扩」(补遗包·执行线守规立功) | C 全仓扫挖出 4 个 DIRECT+DB 写族,其中 real_execution_command 跨用户确认/真实执行安全面——执行线按红线停手报回待核定,未轻档擅接;总指导核定=M5-C 扩包(M6 硬前置·亲核对账分表制=不挡重 seed) | 轻档包越界碰安全面;「只包一层写桥」造出无法安全恢复的 DB-leading 数据 |
