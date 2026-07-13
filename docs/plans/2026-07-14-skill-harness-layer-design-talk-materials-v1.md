# skill/harness 层设计谈话材料 v1(L2 盘点实据+五块板·总指导附推荐·决定权在用户)

日期:2026-07-14 · 实据:同日 L2 全景盘点(五张网:锚点/数据面反查/功能形状/canon 交叉/运行痕迹;live 计数全实测)。盘点全文=盘点员报告(候选总表/skills 17 项逐项/血管图/坑清单),本档为拍板用摘编。

## 〇、总图(一句话)

**两本账物理完全分离**:账 A(开发流程 harness)活得很好——shape gate/checkpoint-audit/catch-log/commit-msg 钩天天在跑,外加一个 2026-05-31 整包安装的 82 脚本外装包(15 已接/11 休眠/42 未接/10 退役,正本=`docs/harness-catalog.md`);账 B(产品域)= 三件事:①**半通血管**——index 扫描器(能识别 harness 目录/候选/入口+53 个 skill)→只读展示板,但索引 **2026-05-31 停更**,展示面在看 1.5 个月前的世界;②**活着的协议字段**——任务包 `harness_requirements`/`acceptance_criteria` 在站 3a/3b 真跑里**真用过**(live 26 个任务包,字节级验收真实存在);③**从未写入的 store 数组**——`harness_resources=0`/`capabilities=0`,2026-05-28 决策设计了「索引候选→用户登记→事实」升格链,**登记机器从未建**。

## 一、盘点顺带抓到的两个真问题

1. **completion gate presence≠passed**(`workflow_read_model_entrypoints.rs:1543`):完成闸门把「配了 harness 要求」当「harness 通过」——隐性放行阀。canon 原文是「审查或 harness 是否**通过**」。
2. **canon 漂移**:CURRENT §四c「harness 系统=没建」按字面漏看已建六件套(协议字段+run check+completion gate+接口边界+双只读板+索引扫描);准确口径=「**登记/运行/管理闭环没建;协议字段与只读面已建且真用过**」——与记忆层同款病(钉板落后于代码)。

## 二、五块板(附推荐)

**板1·账 A 外装包处置**(82 件里 52 件未接/退役):a) **定格现状**(catalog 标 canon·零工时) / b) 移 archive(违「文件不删」惯例) / c) 继续接线(上游=脏快照未发布,先拍归属)。**推荐 a**;将来 L2 产品层想收编个别脚本(skill-recommend/task-package-lint 同题)时单独判。

**板2·harness_resources 通血路线**:a) **先修索引新鲜度**(重跑 build_index.py/接进 app,展示面先活) / b) 建「候选→登记」最小机器(兑现 05-28 决策;但=新增主 store 写面,**该等 M5-B 落地后骑在已接线世界上**) / c) 废顶层数组认字段为唯一真源(=改 05-28 决策,蓝图 HarnessAdapter 没落点)。**推荐 a 为第一刀、b 为第二刀(M5-B 后)、否 c**。

**板3·skill 层起步**:a) 收编 repo `skills/` 17 件当第一批内容(注意:全是旧重流程 skill,当**格式样本**可、当推荐内容需过 canon 信任/版本关) / b) 先接注入(scope→available_skills→worker prompt;canon 要求 skill text=不可信材料+版本 registry 前置) / c) 盘点面刷新。**推荐 c 随板2a 顺带 + a 仅作格式样本;b 等版本 registry 设计**。

**板4·三个 canon 未决+一个代码修正**:未决三题(多 harness 冲突/失败=阻断 or 警告 or 建议/输出如何进 UI)=设计谈话必答;代码修正=presence≠passed 的诚实化(最小改法:闸门条款如实呈现「已配置·未验证」而非视为过,「harness 结果一等对象」留设计)。**推荐:小修进第一刀,三题进谈话**。

**板5·词表拍死**:harness 四义混用(脚本包/索引资源/任务包字段/执行桥)+capabilities 五义+前端叫「运行器」——L2 动工前必须拍死统一词表,否则新面孔必造第五个名字。**推荐:谈话第一项,拍完落 decisions**。

## 三、L2 第一刀候选(若按推荐拍)=「盘面激活包」

①索引新鲜化(重跑 build_index.py+确认 Harness/Skills 双板显示活数据+记录刷新姿势);②completion gate presence≠passed 诚实化小修(+案发测试);③CURRENT §四c 口径修正(canon 漂移纠正);④词表决策落档。零新机器、零登记链(那是第二刀)、零 skill 注入。

## 四、遗留索引(不进第一刀)

登记机器(板2b·M5-B 后)/skill 注入+版本 registry(板3b)/三 canon 未决题/两个 stage-k .swift 探针未入 catalog/`tooling-and-mcp-registry.md` 陈旧行(agentmem 仍标 Preferred)/`harness.config.json` 死块(memoryIntegration 退役未删·两套 hook 系统并存注记)/索引弱信号误报(132 候选 vs 15 资源=升格链「用户确认」步的存在理由,勿自动采纳)。
