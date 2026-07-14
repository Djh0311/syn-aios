# 任务包:前端阶段 1 全面体检(只读诊断·不动码)v1

日期:2026-07-14 · 档位:轻档(纯分析零代码) · 执行者:体检对话(需可载入 impeccable/interface-design skill) · 上游:七阶段流程 `docs/plans/2026-07-14-syn-frontend-overhaul-process-plan-v1.md` 阶段 1。

## 先读(顺序)

1. `decisions/2026-07-14-interaction-model-canon-v1.md`(交互宪法·验收准绳——五态唯一问题/打断三级/布局恒定/空态/回顾面 §六);
2. `docs/ux-friction-log.md`(10 条用户实录·体检时逐条归位);
3. 流程规划 §一 阶段 1 + §二 交流协议(产出给用户的部分必须「亲身场景人话版」)。

## 范围(普查·一个不落)

前端根 `prototypes/productized-desktop-shell/src/`;导航正本 `src/lib/workbenchNavigation.ts`(视图清单:home/projects/agents/ideas/knowledge/memory/skills/harness/workflow 实验画布/command-console/proposal/tools/models/settings/secretary_board+右栏 rail 六项)。项目页内部子面(交办 Jiaoban/画布/历史栏/工单详情/交货面/卡住脸)逐一单独体检——`ProjectJiaobanPanel.tsx` 3845 行是重灾区。

## 每面体检行(固定格式)

`面名 | 归属(五态之一/回顾面/系统) | 该面唯一问题 | 违宪项(逐条:违反宪法哪条+证据[代码行/文案原文/结构]) | 严重度(P0 挡用户/P1 违宪/P2 打磨) | 依赖(改它要不要先拆巨石/等后端) | 档位建议(深磨/够用——对照后端演进:交货面等实证块上脸=够用档;记忆中心 L1/L2 演进中=够用档)`

## 体检手段

- 主路=读代码(视图/组件/文案/状态呈现);
- 辅路=vite dev(5173)真实渲染可看布局与静态结构(Tauri API 缺失面板会空,属预期,只看得到的);
- 离线 SSR 测试(`tests/` 24 套件)的 markup 可作文案/结构佐证;
- 不动任何源码;不跑 App;不碰 live 根。

## 产出(两份)

1. **《Syn 界面诊断与改造蓝图》**落 `docs/plans/2026-07-14-syn-frontend-stage1-audit-v1.md`:全部体检行+IA 审计(对象层级/导航深度/入口一致性——对照宪法 §二)+分批建议(按态/中心分组,预设顺序:交货面→记忆中心→方案卡→干态进度→卡住脸→其余,可按发现调整);friction log 10 条逐条归位到对应面;
2. **给用户的排序决策包**(对话里给,不落档):按交流协议——每批用用户亲身场景讲(「你说记忆中心没法看=这批」),打包问,决策 ≤5 次。

## 回传

给总指导:诊断档路径+体检面数+P0/P1/P2 计数+「用户已排序/待排序」一句话。总指导核收后按排序拉阶段 2/4 的包。
