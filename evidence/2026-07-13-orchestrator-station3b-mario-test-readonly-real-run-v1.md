# 站 3b「mario test」只读真实闭环证据 v1

状态：`PASS__SINGLE_WORKER__ZERO_WRITE__NOT_COMMITTED`

## 结论

2026-07-13，Syn 在固定真实项目 `/Users/yoyi/Documents/mario test` 完成一次新的主管编排闭环：

`dispatch_worker → inspect_worker → finalize(pass) → report_user`

全程只有一个 worker，授权写根为空，主管和 worker 均使用 `--sandbox read-only`。worker 回程被控制核心解析为 `reported_completed`；主管终标为 advisory `pass`，未写工作流链态。项目前后 7 个内容文件的 SHA-256、文件清单和 `git status --short` 完全一致。

这只证明当前获批项目的只读单人路径，不外推其它真实项目、写单或多 worker。

## 本次身份

- supervisor run：`supervisor:workflow-users-yoyi-documents-mario-test-default:1783918485705864000`
- authorization：`plan-auth:project-users-yoyi-documents-mario-test-workflow-users-yoyi-documents-mario-test-default-node-node:1783918484464`
- work item：`work-item:workflow:users-yoyi-documents-mario-test:default:project-director:planned-task-supervisor-pilot-eb33d80132fa15315006376e`
- native worker thread：`019f59d4-1f7a-7a52-88f6-e46308dd9f09`
- dispatch / worker：`dispatch:workflow-users-yoyi-documents-mario-test-default:work-item-workflow-users-yoyi-documents-mario-test-default-project-director-planned-task-supervi:1783918513688`

五件套均为本次新建，未复用站 2、站 3a 或历史失败尝试。

## 链路证据

控制核心只接受四个动作：

1. `dispatch_worker`：唯一 worker 启动；`allowed_write=[]`。
2. `inspect_worker`：读到合法结构化回程；`acceptance_status=reported_completed`，`evidence_present=true`。
3. `finalize`：`verdict=pass`、`advisory_only=true`、`workflow_chain_state_written=false`。
4. `report_user`：用户可见报告落账；`user_decision_written=false`。

主管 session 中 `workers.len()=1`、`follow_up_count=0`、`final_marks.len()=1`。worker wrapper/native 位于独立进程组 `PGID 94133`，自然结束后登记已注销；主管临时 `CODEX_HOME` 每步均创建后清理。

完整 ID、构建 SHA、动作顺序与权威账本位置见 [attempt-4-run-ledger.md](raw/2026-07-12-station3b-mario-test-readonly/attempt-4-run-ledger.md)。

## 口供质量

worker 实际逐行读取：

- `README.md:1-20`
- `index.html:1-37`
- `styles.css:1-139`
- `game.js:1-346`

回程包含 README 承诺逐条判断、按影响排序的前 5 个问题、每条 `file:line` 与源码原文、50 字内总评，以及 `node --check game.js` 退出码 `0`、完整空输出。控制核心据此终标 PASS。

完整结构化回程见 [attempt-4-worker-report.json](raw/2026-07-12-station3b-mario-test-readonly/attempt-4-worker-report.json)。

## 零写证明

发射前与结束后均检查同一 7 个内容文件。结果：

- `git status --short` 文本一致；
- 内容文件清单一致；
- 7 个 SHA-256 全部一致；
- `node --check game.js` 前后均退出 `0`，stdout/stderr 为空；
- worker 报告 `outputs=[]`、`实际改动文件：无`。

前后证据：

- [attempt-4-pre-launch-baseline.txt](raw/2026-07-12-station3b-mario-test-readonly/attempt-4-pre-launch-baseline.txt)
- [attempt-4-post-run-baseline.txt](raw/2026-07-12-station3b-mario-test-readonly/attempt-4-post-run-baseline.txt)

## 工程风险清理

真跑前发现 debug `.app` 内二进制旧于当前源码；已重建并把 bundle 二进制与最新 target 二进制对齐，主管账本记录本次运行构建 SHA-256 为 `08163d25c5e696f6dfca6d2ff9d5ca1db47d5622d21b3c2cecbf3853869e4fd3`，避免拿旧实现冒充新验证。

真跑又暴露 durable process registry 的 worker `run_id` 在旧请求字段为空时退化成 `resume:`。已改为 `codex-local:<operation>:<stable SHA-256 identity>`，只改变进程登记可追溯性，不改 runner argv、派发闸或业务状态；新增稳定性/区分性单测。

## 验证

- 真 UI：单 worker、零写根、完整四动作、advisory PASS、用户回报已落账。
- 独立零写比对：7/7 SHA-256 一致。
- `cargo test --lib exec_process_registry::tests:: --quiet`：`9 passed; 0 failed`。
- 全库回归与前端检查在本轮最终收口时统一刷新；本文件不以早先回归结果冒充最终结果。

未 commit。
