# M2 — 91 txt 分类 + 搬迁计划 + hash 清单 v1(**只出计划·不执行搬迁**)

> 资料状态（2026-08-09）：历史分类计划和来源证据，不是当前迁移入口或授权。下文数量、路径和哈希只对点名基线成立；当前事实必须重新核对。

日期:2026-07-13 · 基线 `d952a7d` · 配套:M0 合同 `docs/2026-07-13-sqlite-migration-completeness-contract-m0-v1.md` · hash 清单 `evidence/2026-07-13-sqlite-migration-m2-live-root-hash-manifest.txt`

> **状态门**:本件只分类 + 出计划 + 留 hash。**不执行任何真实搬迁/删除/移动**(红线 #1/#6)。真实根一个文件不动(M3 亲核 hash 前后不变)。

## 一、91 txt 精确分类

live 根 91 个 `supervisor:*.txt`,**全部 = runtime-transient(supervisor 逐步转录件)**,不属 JSON→SQLite 领域迁移:

| kind | 计数 | 内容 | 分类 |
|---|---|---|---|
| `supervisor:<workflow>:<run-id>[.step-N].last-message.txt` | 45 | supervisor/worker 会话某步的最后消息文本 | runtime-transient |
| `supervisor:<workflow>:<run-id>[.step-N].stderr.txt` | 46 | 该步 stderr 原文 | runtime-transient |
| 合计 | **91**(20 个 supervisor 会话·364KB) | | |

**判据**(勘察 + M3 亲核):
- preflight 判它们 `non_json_file`→阻断(`is_ignored_support_file` 只跳 dotfile/.tmp/.lock,不跳这些)——所以它们**从来不进** JSON→SQLite 域;不是迁移对象。
- 内容 = 运行期步进转录(供 run-history 人话上脸/调试),重启/重跑即再生语义,非领域真相。
- 命名 `run-id` = 纳秒时间戳,和某次 supervisor 编排绑定,非持久实体键。

## 二、搬迁计划(**建议·未执行**)

| 项 | 现状 | 建议 | 回滚 |
|---|---|---|---|
| 91 txt | 平铺在 workflow-state 根,污染 preflight | **搬到 `workflow-state/runtime-artifacts/`(或 `logs/`)子目录**,与领域 JSON 分离 | 目录搬迁可逆:留 hash 清单,逐文件校验后移回即恢复 |
| `exec-process-registry.v1.json` | 根目录,被 preflight 判 `unknown_json_file` | 归 runtime-transient;**保留原位**(app 启动读它收僵尸),迁移 preflight 应**显式忽略**而非拒绝 | 不动,无需回滚 |
| `backups/`(98 快照) | 根下子目录 | historical-archive;保留;preflight 已因非根级 .json 不扫(确认) | 不动 |

**preflight v2 建议(仅计划·不改码)**:preflight 的 unknown 拒绝应升级为「**显式忽略明确的 runtime 件**(runtime-artifacts/、exec-process-registry、*.txt 步进转录)+ 对真正未知项才 fail-closed」。当前是「一刀 unknown 全拒」,导致 live 根无法整体过 preflight。**此项触及 preflight 判定逻辑,属安全谓词相邻,需用户单独确认后另开包**(红线 #3/#4)。

## 三、执行前置条件(搬迁真做时·本包不做)

1. app 停写(避免搬迁中并发写 workflow-state)。
2. 逐文件:算 hash → 移动 → 目标 hash 复核 == 源 hash → 记账。
3. 更新 run-history 读取路径(若代码按平铺路径找 txt)——**需先 grep 消费方**,勿假设无引用。
4. preflight v2 落地(忽略 runtime-artifacts/)后再迁,否则搬完仍卡 preflight。

## 四、hash 清单

完整 SHA-256 清单见 `evidence/2026-07-13-sqlite-migration-m2-live-root-hash-manifest.txt`:
- 12 域 JSON(含 `workflow-state.v0.json` = `bf3e6f47…`)
- 91 txt 逐文件 hash
- `backups/` 98 文件整集 hash

**真实根未动证明**:M3 亲核(`sqlite_m3_live_snapshot_...` 测试)在读取前后对整个 live 树算 hash,前后一致(`3d8f962e…`);本 M2 只 `shasum`/`ls`(只读),未写未删未移。

---

## 五、⚠ M3 连带发现(headline·需用户决策)

M3 真机演练发现:**当前 live 主 store `workflow-state.v0.json` 被 importer 的敏感谓词 `contains_sensitive_value`(importer.rs:965)判 `rejected_sensitive`**,原因是它对 `SENSITIVE_KEY_PARTS` 的 `"token"` 做**子串键匹配**,误命中良性字段 `estimated_tokens`/`max_estimated_tokens`(LLM token **计数**元数据,非凭据)。真凭据类标记(secret/credential/prompt_body…)在主 store **并不作为键出现**——纯误报。

**影响**:一旦真翻闸(M5/M6),迁移会**静默把整个主 store 判敏感拒收**。这正是 M3「翻闸前先演练」要抓的。
**处置**:`contains_sensitive_value` 是**安全谓词**,红线 #3 禁改 → 本包**停下报**,不擅改、不绕行、不篡改 live 数据。
**建议(需用户单独授权另开包)**:该谓词键匹配改**词界/整键**匹配,或对 `*_tokens` 计数字段 allowlist,使 token-计数 不被当 auth-token。
