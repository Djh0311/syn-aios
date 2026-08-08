# PLH006 分区验收、回滚证据与阶段收口

阶段：stage-01 代码事实收敛、唯一权威与 Lite 切换

目标：证明唯一代码基线、唯一项目权威、旧 Harness 退出、Lite 生命周期和脏工作保护同时成立。

干完的标准：代码、控制面、生命周期、项目小检查、回滚、Git/外部边界六区都有可重放结果；阶段关闭后进入“等待用户指定下一项”，authorization 不继承。

允许动：

- `docs/harness/audit/` [新增]
- `docs/harness/reports/` [新增]
- `docs/harness/plan.md`
- `docs/harness/stages/stage-01.md`
- `docs/harness/leaves/PLH006-cutover-acceptance.md`
- `docs/harness/done/2026-08/` [新增]

## 验证

1. 代码：候选 tree、来源提交、两个脏现场哨兵和 staged。
2. 控制面：旧 runtime/manifest/config/活动引用为零。
3. Lite：ownership、chain、progress、auth、quick/task、Stop 零产品测试、重复安装零写入。
4. 项目：只跑直接相关小检查；未跑 full、运行时和真实 App 明说。
5. 回滚：隔离恢复切换前快照，hash/mode/入口一致。
6. Git/外部：无 push、远端、部署、发布、provider、数据库、浏览器和真实消息。
