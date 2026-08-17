# SYN-M1I01 服务器持有的 project_index 与项目角色身份权威

日期: 2026-08-17
阶段: stage-14 仍开；唯一 current leaf 保持 M5R07
状态: 用户明确继续并要求 Grok 实际实现的独立 M1-owner 窄包

本包只补齐 M3O01 所缺的权威 canonical `ProjectId` 源与项目三角色身份记录。它不是 M3 RoleSession provision / load / restore，也不是 M5 / M6 施工。

## 授权边界

- 主工作树有用户 WIP：不 reset / stash / clean / `git add -A`。
- 不动六份已跟踪壳文档、未跟踪壳文档、`linux-schema.json`、任何 `m6_*.rs`、任何 M3 / M5 源文件。
- 不改冻结 M1–M3 正文 / hash，不改 schema 语义。
- 不改 Harness stage / leaf / authorization。
- 不改现有未跟踪 M3O01 文档。
- 只新增独立任务、unfinished 投影、增补合同、M1 服务器权威端口、普通 `AppState` 最小安装、定向离线测试。
- 不新增 renderer / Tauri command / 原始 registry 外露。
- 不接新网络 / provider / 真实 App 执行，不做 legacy / live 迁移。

## 产品结果

普通产品 `AppState` 安装一个仅服务器可见的 M1 `project_index` 权威端口。该端口：

1. 接受显式服务器侧隔离项目登记，写入本地 app-data registry；
2. 登记时签发不透明随机 canonical `project:<uuid>`，以及 `project_supervisor` / `worker` / `independent_reviewer` 三条互不相同的不透明稳定 actor / session-identity 记录；
3. 只原子持久化该 M1 registry，供重启稳定恢复；
4. 把 project root / locator / slug / scratch / caller boolean / M5 helper 只当别名或 resolver 输入，绝不由其派生或签发 ID；
5. 绝不自动导入 legacy index；
6. 只有预先登记的精确别名才能解析到已存储 ID；unknown / duplicate / malformed / stale / alias mismatch 一律 fail closed；
7. 角色身份只返回不可变的服务器-only project / role / actor / scope / current-object / channel / permission snapshot / owner fingerprint / revisions；reviewer 必须与另外两角不同；
8. 这些 session identity 只持 least-privilege 无能力 session profile；ExecutionGrant 仍在范围外。

没有事先显式登记的普通 legacy 项目保持不可用。未来普通项目创建 / legacy 迁移 owner 只在增补合同中记录为后续独立授权，不在本包猜测实现。

## 验证

- `cargo check --lib --offline`
- 定向 `cargo test --lib --offline -- m1_project_index m1_project_role_identity`
- `git diff --check`

不 push / merge / rebase / deploy / release。
