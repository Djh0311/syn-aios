# 拍板:站4 = mario test 单项目最小写解封 v1

日期:2026-07-14 · 拍板人:用户(真机验收现场) · 语境:三合一验收首单(在 `/Users/yoyi/Documents/mario test` 创建 test.txt)撞出两层设计边界,用户拍板开写解封。

## 撞出的事实(核实物)

1. 咨询钳位死锚:`consultant_agent.rs::profile_edit_test_project_scope()` 把编辑类方案 `allowed_write_roots` **写死**固定测试项目 `/Users/yoyi/codex-workflow-mario-test`(交办地基 2.1 防滑坡设计,「不可由请求参数改写」)。用户需求快照/read_roots/验收标准全是 mario test,唯写根字段 LM 改不了 → 方案与目标错位,全局主管边界意见抓对症状。
2. 执行闸本就不放:mario test 唯一解封=站3b「根精确相等 ∧ **写根为空**」(`station3b_readonly_project_unsealed`)。带写根必拦。站3b 红线「不得外推」工作正常。

## 拍的板

**开站4:mario test 单项目、单写根、最小写解封。** 边界:

- 解封判定=根精确等于 `/Users/yoyi/Documents/mario test` ∧ 写根精确等于 `["/Users/yoyi/Documents/mario test"]`(恰一条、无子目录宽展、无尾斜杠变体);
- 只挂主管编排链路(站3b 同款面),经典管线/legacy 旧桩/自动连环照旧 blocked;
- `workflow_engine_test_project_unsealed`/`require_test_project_path_lock`/站3b 判定本体**零放宽**;
- 咨询钳位改**白名单制**:写解封白名单={固定测试项目, mario test};项目根在白名单内→写根=当前项目根;白名单外→**写根留空降为只读方案**+方案内人话说明(复用 07-10 纯建议只读单机制)——替代「白名单外填死锚」的错位行为;
- 不做任意项目写、不做多写根、不做白名单 UI 化(将来项目授权机制另拍)。

## 档位

高危#1(真实项目写)+#3(改闸)。**代码实现=轻档先行;真跑那一下=重档**:用户在场、单独授权、沙箱限定(workspace-write·写根仅 mario test)、简短证据(test.txt 8 字节无换行+固定测试项目零写+其它项目仍 blocked)。

## 任务包

`tasks/2026-07-14-station4-mario-test-write-unseal-package-v1.md`(串行第二,排 M5-B 补遗包后)。
