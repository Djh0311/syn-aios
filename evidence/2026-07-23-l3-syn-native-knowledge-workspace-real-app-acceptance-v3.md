# L3 Syn 原生知识工作区真实 App 验收停点 v3

- 日期：2026-07-24
- 上游任务包：`tasks/2026-07-23-l3-knowledge-open-host-owned-relay-and-real-app-acceptance-package-v1.md`
- 状态：**R4 在 fresh Gate 0 前安全停止：当前工作树 Syn 启动后自动渲染既有非验收 store 面，触及本包禁止的既有条目/真实项目边界。**

## 启动前冻结

- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`。
- porcelain 指纹：`fd3822524fff68c425598d3f87f8a65f8fcfc81bc16fba5e4638c2cb78ecae48`；保留既有大量脏改，未 reset/clean/stash。
- staged：空；启动后复核仍为空，`git diff --check` 通过。
- Gate 0 runtime cut 前，R4 承重代码 SHA 未漂移；其中 `index_host_app_entrypoints.rs` 为 `5ae1f355bc5f0c3f07c24bfa91be7479728affd275dc5259d328b5bf68182a8e`，`lib.rs` 为 R3 指导验收记录的 `828667f3f8631d3d6a9932f3abb93b9101e79c94817ff9b3e24f7167a9abf3ff`。入口文档随后仅按本停点写入事实记录。
- scoped 进程基线无法由 `pgrep` 获得，系统返回 `sysmond service not found`；因此没有作全系统零残留声明。

## Gate 0

1. 通过当前工作树的 `npm run tauri:dev` 启动 Syn；构建完成，沿用 R3 已知的 598 条历史 warnings。
2. App 在进入知识页、读取固定 vault manifest 或创建任何验收命名空间之前，首页自动呈现既有非验收的项目/待办等 store 面。为避免继续接触该面，立即停止。
3. **未发送主管首句**；未观察或读取 durable binding、natural reply 或 `tools/list`；未启动本包所需 Codex CLI/MCP 会话；`submit_proposal` 与四项 knowledge tool 均为零调用。
4. 未读取 fixed vault manifest、未进入知识页、未读取任何 vault 正文，也未创建验收目录或文件。

最早 blocker：**`BLOCKED_REAL_APP_PRE_GATE0_EXISTING_STORE_SURFACE`**。它对应任务包第 8 节“真实 App 触及既有非验收条目、真实项目、卡/chain/worker”停止条件；这不是 relay、binding、自然回复或五工具面的失败，故不混写为 `BLOCKED_EXISTING_CONVERSATION_BINDING_REAL_APP`。

## 十二项结果

| # | 计划场景 | 结果 |
| --- | --- | --- |
| 1 | 新建目录、Markdown 笔记和属性 | 未执行：Gate 0 前安全停止。 |
| 2 | 双链在反链区出现 | 未执行：Gate 0 前安全停止。 |
| 3 | 全文搜索和快速打开 | 未执行：Gate 0 前安全停止。 |
| 4 | 分栏编辑和预览 | 未执行：Gate 0 前安全停止。 |
| 5 | 全局/局部图打开目标笔记 | 未执行：Gate 0 前安全停止。 |
| 6 | 新建、编辑、保存并重开 JSON Canvas | 未执行：Gate 0 前安全停止。 |
| 7 | 导入允许附件并从笔记/Canvas 引用 | 未执行：Gate 0 前安全停止。 |
| 8 | 模拟外部改动并确认冲突不覆盖 | 未执行：Gate 0 前安全停止。 |
| 9 | 主管 `search/read/open/cite` 与真实引用 | 未执行：Gate 0 前安全停止。 |
| 10 | AI 写允许一次、拒绝一次 | 未执行：Gate 0 前安全停止。 |
| 11 | 重启 Syn 后恢复知识文件和工作区 | 未执行：Gate 0 前安全停止。 |
| 12 | 未安装 Obsidian时核心闭环成立 | 未执行：Gate 0 前安全停止。 |

## 实物、清理与下一步

- 未保留截图或原始 UI 日志，避免把既有非验收内容写入 evidence；`evidence/raw/2026-07-23-l3-native-knowledge-real-app/` 未创建。固定 vault manifest 也没有生成。
- 本次启动终端句柄 `18150` 已以 Ctrl-C 正常退出；随后仅核对本 App 的运行状态为未运行。系统进程表仍不可读，故只报告该自建句柄的清理，不宣称全系统无残留。
- 未发出 vault/store、binding/DB/JSON、能力/权限 allowlist 或代码的业务写操作，也未创建验收命名空间；但默认 App 启动并非只读，且本包未授权独立比对真实 store，故不宣称零隐式启动副作用。没有 stage、commit、push、reset、clean 或 stash；没有真实 catch，未修改 catch-log。
- 若要重启 R4，最小下一包应先提供并验证：不会在首屏渲染既有真实 store 的隔离 app-data/store 启动边界、唯一允许的 test project/workflow 身份、无敏感内容的 actual `tools/list` 证据路径，以及 manifest 内部读取/条目呈现的数据最小化口径；在这些边界获明示授权前，不重试 Gate 0。
