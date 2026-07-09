# 调研任务包:能力普查地图(接旧图·补全刷新)· 主导线 → 执行线(便宜模型) v1

日期:2026-07-09　性质:**纯只读调研·产一份地图文档**(grep+读+填表·零代码改)。缘起:仓大(200 文件/12 万行)、多线并行、易重造轮子(C0 已实证)。要一张**当前的、全仓的"能力→在哪个文件"地图**,以后写"加新能力"的包先对图。**便宜模型可做**:机械 grep pub 面+一句话角色·不判架构。

## 0. 接手须知(冷启即读·本包自包含)

- 你是**执行线**(调研·**便宜模型**)。**只读·产一份新文档·不改现有代码/文档·不 commit。** 全程中文。
- **不从零造·接旧图**:本仓已有半张图,**先读它们、覆盖到的部分直接引用+标"是否还准",别重写**:
  - `docs/plans/2026-06-11-root-treatment-r2-lib-rs-code-map-v1.md`(lib.rs 代码地图·6 月·可能旧);
  - `docs/plans/2026-06-24-s2-1-seam-wiring-map-v1.md`(接缝布线);
  - `docs/evidence/2026-06-18-conversation-module-native-p0-contract-inventory-v1.md`(会话模块契约);
  - `docs/harness-catalog.md`(harness);
  - `docs/2026-07-08-workbench-current-feature-inventory-for-prototype-v1.md`(产品面盘点·非代码级)。
- **你补的 = 这些图没覆盖的 + 覆盖了但可能过期的(标注)**。仓根 `/Users/yoyi/workspace/product-line`。

## 1. 要产什么:一张"能力→文件"地图

**新文档**:`docs/2026-07-09-codebase-capability-map-v1.md`。按区域分组,每个文件一行:文件名 · 一句话角色 · 关键 pub 面(2-5 个 `pub(crate) fn`/`struct` 名)。目标粒度 = **"想找某能力在哪,查这张图能定位到文件"**。

## 2. 怎么做(机械·便宜模型照步骤)

### 2.1 后端(99 .rs·`prototypes/productized-desktop-shell/src-tauri/src`)

- 逐文件:`grep -n 'pub(crate) fn \|pub fn \|struct \|enum ' <文件>` 抽 pub 面;读文件头注释/前 20 行取"角色一句话";
- **按区域分组**(建议·按文件名+内容归):① 会话/relay(codex_db/manual_relay/codex_local_runner/session_*)② workflow 编排(workflow_*/director_agent/workflow_chain_*)③ 派发/执行(*dispatch*/execute/real_execution)④ 记忆五层(memory_*/formal_memory/observation/*candidate*)⑤ 治理/闸(c4_c6/control_core/*governance*)⑥ 读模型(*read_model*)⑦ 类型/工具(types/utils/*)⑧ 其它;
- `lib.rs` 太大:**引用 6-11 那张 lib.rs 代码地图**,只补"那张图之后新增的大块"(如 B2 的测试/命令),别重画。

### 2.2 前端(101 .tsx/.ts·`prototypes/productized-desktop-shell/src`)

- 逐文件一行:组件/模块名 · 一句话它渲染/管什么 · 挂在哪个视图(grep import 关系粗判即可);
- 按面分组(对现状说明书六面:首页壳/项目页/智能体页/记忆/秘书/审计 + lib 工具)。

### 2.3 旧图核对(顺手·别深挖)

- 读那 5 份旧图,每份标一句:**"还准 / 部分过期(哪块)/ 已被取代"**——凭你 grep 到的现状粗判即可,拿不准标"待主导线核",别下死结论。

## 3. 死线

- **只读 + 只新增那一份地图文档**:不改任何 `.rs`/`.tsx`/现有 `.md`,不 commit,不跑写盘命令;
- **不判架构对错、不判重复该不该合**(那是主导线)——地图只**如实记"有什么、在哪"**;
- 不碰 `.codex`/沙箱/不真跑 codex。

## 4. 回交格式(地图文档结构)

```
# 代码库能力地图 v1（2026-07-09·便宜模型普查·主导线待核）
## 旧图核对（5 份·还准/过期/取代）
## 后端能力（按 8 区域）
### 区域①·会话/relay
- `codex_db.rs` — codex sqlite 读写 — `read_threads_page`/`find_thread_by_id`/...
- ...（99 文件逐行）
## 前端能力（按六面+工具）
- `WorkbenchShell.tsx` — 工作台外壳+秘书入口+浮钮 — ...
- ...（101 文件逐行）
## 普查中撞见的"疑似两套"（顺手记·不判·交主导线）
- 若 grep 时撞见明显同义的两处实现，单列一节让主导线看（如 C0 的 SubagentReport 那种）
```

## 5. 回交 → 主导线

- 新地图文档路径 + 一句话覆盖率自评(99+101 文件覆盖了多少)+ "疑似两套"清单 → 主导线核+收编进正本。**你不 commit。**

## 7. 不接受为

- 从零重画 lib.rs 那部分(旧图有·引用+补增量)/ 改现有代码或文档 / commit / 判架构或"该不该合"(主导线判)/ 粒度太粗(要能定位到文件+pub 面)/ 漏掉"疑似两套"的顺手记 / 跑写盘命令。
