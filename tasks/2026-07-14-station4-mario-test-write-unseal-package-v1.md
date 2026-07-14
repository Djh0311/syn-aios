# 任务包:站4——mario test 单项目最小写解封 v1

日期:2026-07-14 · 档位:**代码=轻档;真跑=重档**(高危#1+#3,用户授权那一下不可省) · 拍板:`decisions/2026-07-14-station4-mario-test-minimal-write-unseal-v1.md` · **串行第二:M5-B 补遗包收口后才开本包**。

## 目标

用户在 `/Users/yoyi/Documents/mario test` 说「创建 test.txt 写入 12345678」这类单文件写需求,能走通:咨询方案写根=当前项目根 → 批准 → 主管编排链路真跑写入 → 独立复核 → report_user。其它一切项目行为零变化。

## 范围(四块)

**A·S1 闸第三支**(`commands.rs` 站3b 判定区 :2196-:2230 旁):
- 新 `const STATION_4_WRITE_PROJECT_ROOT = "/Users/yoyi/Documents/mario test"`(与 3b 同值但独立常量,语义不同);
- 新判定 `station4_write_project_unsealed(project_root, write_roots)` = 根精确相等 ∧ `write_roots == [同根]`(恰一条·精确串等·无子目录/尾斜杠/多写根宽展);
- `real_execution_authorization_complete` 加第三支:`supervisor_authorized ∧ station4_write_project_unsealed(...)`;
- **零放宽**:`workflow_engine_test_project_unsealed`/`require_test_project_path_lock`/`station3b_readonly_project_unsealed` 本体一字不动;legacy 封条照旧。

**B·咨询钳位白名单化**(`consultant_agent.rs::profile_edit_test_project_scope` :530 区):
- 白名单 = {`/Users/yoyi/codex-workflow-mario-test`, `/Users/yoyi/Documents/mario test`}(常量,不做配置化);
- 项目根在白名单内 → 编辑类方案写根=**当前项目根**;白名单外 → **写根留空降为只读方案**(复用 07-10 纯建议只读单机制)+方案 proposed_steps 首条人话注明「该项目未获写授权,已降为只读方案」;
- 函数签名带入 project_root(现无参),调用点同步;「不可由请求参数改写」的防滑坡本意保住:白名单外永远到不了任意写根。

**C·派发链路写根接线**(读透再动:`supervisor_session_launcher.rs`/`codex_local_runner.rs` 的 argv 构造):
- 站3b 只读单=read-only 零 `--add-dir`;站4 写单=workspace-write+写根仅 mario test(沿现有守卫升级路;守卫按 :4202 先见注释分流那套);
- 派发前守卫核:授权段写根必须通过 station4 判定,不过=拦+人话;
- `director_agent.rs:4769` ManualRelayJiaobanNewSessionCreator cwd 写死=经典 jiaoban 路,**不在站4 面上,零碰**(注记即可)。

**D·案发测试**(仿站3b 那组+写面新增):
- 闸:子目录/尾斜杠/多写根/空写根/其它项目/写根≠项目根 全拒;恰好形态放行;
- 咨询:白名单内项目→写根=项目根;白名单外→写根空+人话降级说明;固定测试项目行为回归不变;
- 端到端(temp 模拟面):方案→授权→prepared dispatch 的写根贯穿一致。

## 红线

安全闸改动仅限 A/B/C 点名面(改闸=高危#3,包内已获用户拍板,超出点名面即停);迁移面/存储模式/read_cut 零碰;live 根零写(测试全 temp);**真跑 mario test=重档窗口,包内只做代码+测试,不真跑**;不 commit;automation 5059 不破;gate 14/5/5 零净增;fmt 仅历史三。

## 验收(预写死)

- 上述案发测试全绿;全量基线只增不减;
- 静态:`real_execution_authorization_complete` 三支结构可读;白名单常量单点定义;
- 回传 10 项模板第 7 项 gate 三数必含;
- **真跑验收(包外·重档)**:总指导陪用户在场走「说需求→方案写根=mario test→批→真跑创建 test.txt(8 字节无换行)→复核→report_user」,证据=文件字节级核验+固定测试项目零写+其它项目仍 blocked。
