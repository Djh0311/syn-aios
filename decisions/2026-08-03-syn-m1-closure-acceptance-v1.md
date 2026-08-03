# SYN M1 关闭拍板 · v1

日期：2026-08-03
拍板人：用户
记录人：总指导线
分支：`syn-fnd-002-dev` @ `33079e5`

## 拍板

**Stage 1（合同与安全/作用域基础，M1）正式关闭。** M2 维持 `PLANNED / NOT_ACTIVE`——
按 stage plan §8，本拍板不自动激活 M2，激活需用户另行明确指示。

## 依据（证据均已落盘，非转述）

| 退出门条款（stage plan §8） | 状态 | 证据 |
|---|---|---|
| §3 十份合同冻结 | ✅ | `0b257db`（SYN-FND-001-R1） |
| FND-002/003/004A/B/C/005 聚焦测试 + non-test build | ✅ | `cargo check --lib` exit 0 / 599 warnings；`cargo test --lib` 1304 passed / 2 failed（sqlite 既有 + 进程夹具环境族，均零依赖坐实） |
| FND-006 隔离 Tauri 场景 + before/after 证据 | ✅ 5 场景运行时 + 正控；3/4/5 维持集成/单测级并明写原因 | `test-fixtures/fnd-006-acceptance/acceptance-record-2026-08-03.md` |
| 已知入口全部有状态；caller-controlled execution 入口全部 migrated/blocked | ✅ | `docs/execution-entry-inventory.md`（34 行明细重算自洽） |
| workflow/report/grant 精确 owner/join identity | ✅ | 004A project_id 精确归属；004B 绑定字段持久化（真机坐实） |
| 无 UI 隐藏代替后端拒绝 | ✅ | 拒绝均发生在后端（运行时 console 打到生产命令验证） |
| 无真实 secret/外部动作/真实项目写入 | ✅ | 真实 HOME 1098 项指纹两次终检 IDENTICAL |
| dirty WIP 保留 | ✅ | 六批提交，工作树干净，无未归属 WIP |
| CURRENT 回写 | ✅ | `docs/harness/CURRENT.md` @ 本批 |

## 残留（不阻关闭，但 M2 规划不得当作已有防御）

> 补记（2026-08-03）：用户拍板将以下残留项**直接划入 M2 阶段范围**，承接关系见
> `docs/plans/2026-08-01-syn-stage-2-fact-event-audit-transaction-foundation-plan-v1.md` §0.4。

1. **grant 校验是格式级**：无 grant store，活路径 grant_id = dispatch_id，`verify_grant`
   跑的是自铸通配 grant。M2 必须换真 mint/load/verify。
2. **场景 3/4（伪造 report/grant 全链）** 需 fake runner 夹具；**场景 5（Station 3b）**
   需 supervisor 会话夹具——列为 M2 前评估项。
3. `sqlite_production_preflight_blocked_creates_no_db_or_report` 稳定失败（既有，与本阶段
   改动零依赖）；进程夹具族在本沙箱环境性失败（codex_local_runner/obsidian/manual_relay
   轮流翻），合并排查未做。
4. code-map advisory（`MAP_UPDATE_REQUIRED`）自首批起持续告警，非阻断，待处理。
5. FND-001 合同 commit 未进 integration main（HOLD）。

## 提交序列（本阶段，均在 `syn-fnd-002-dev`）

`63c58c5`（FND-002/004A）→ `3488135`（004B）→ `89c62f2`（003/004C/005 基座）→
`ccfdadb`（状态记录）→ `a408997`（三模块接线 + FND-006 套件）→ `6a722d1`（验收偏离修复）→
`2d4a772`（attempt 白名单接线）→ `cca8146`（FND-006 部分运行时）→ `33079e5`（FND-006 真机轮）→
本批（M1 关闭记录）。
