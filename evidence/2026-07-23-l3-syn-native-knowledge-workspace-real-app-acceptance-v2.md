# L3 Syn 原生知识工作区真实 App 验收停点 v2

- 日期：2026-07-23
- 计划：`docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md`
- 开发合同：`tasks/2026-07-23-l3-syn-native-knowledge-workspace-development-package-v2.md`
- 状态：**十二项均未执行；在启动真实 App、访问真实 vault 或调用真实 MCP 前安全停止。**

## 停止原因

N6 的 `knowledge_open` 已能以固定 vault、已验证 `relative_path` 返回 Syn 原生视图的安全意图，但它运行在 MCP stdio 子进程中。现有 trusted conversation binding 只保存生命周期和允许能力，不能把 tool payload 传给主进程；把路径写进 binding、静态全局或旁路文件都会扩大合同或制造第二真相源。

因此尚缺一个受信任、短期、host-owned 的 cross-process dispatch/ack relay：只有主进程收到已验证目标并确认原生工作区已聚焦后，`knowledge_open` 才能说 `opened=true`。该 relay 需要触及本包未授权的 transport/主进程/UI 写面，按停点没有实现，也没有用外部 Obsidian 代替。

## 十二项结果

| # | 计划场景 | 结果 | 证据 / 原因 |
| --- | --- | --- | --- |
| 1 | 新建目录、Markdown 笔记和属性 | 未执行 | 未启动 Syn App 或访问真实 vault。 |
| 2 | 双链在反链区出现 | 未执行 | 同上；离线索引/反链测试不替代 App 操作。 |
| 3 | 全文搜索和快速打开 | 未执行 | 同上。 |
| 4 | 分栏编辑和预览 | 未执行 | 同上。 |
| 5 | 全局/局部图打开目标笔记 | 未执行 | 同上。 |
| 6 | 新建、编辑、保存并重开 JSON Canvas | 未执行 | 同上。 |
| 7 | 导入允许附件并从笔记/Canvas 引用 | 未执行 | 同上。 |
| 8 | 模拟外部改动并确认冲突不覆盖 | 未执行 | 同上。 |
| 9 | 主管完成 search/read/open/cite，回复含真实引用 | 未执行 | 直接阻塞：缺少 host-owned native-view dispatch/ack，不能宣称 `knowledge_open` 成功。 |
| 10 | AI 写允许一次、拒绝一次，分别证明单审计写/零写 | 未执行 | 不用离线 pending-action 合同替代真实 App。 |
| 11 | 重启 Syn 后恢复知识文件和工作区 | 未执行 | 未启动或重启 Syn。 |
| 12 | 未安装 Obsidian时核心闭环成立 | 未执行 | 未做真实 App 闭环；离线兼容入口不等于本项通过。 |

## 已有离线证据（不替代本表）

- `evidence/2026-07-23-l3-syn-native-knowledge-workspace-offline-verification-v2.md`：N0-N5 离线实现、N6 只读 capability/binding 闭锁、格式/类型/离线 runner 和历史债单列。
- 强制运行 `trusted_host_dispatch_must_settle_before_native_view_can_be_claimed_open` 时按预期失败：返回明确保持 `opened=false`，证明当前没有伪造成功打开。

## 实物与安全边界

- 截图：无；没有真实 App 操作。
- 真实日志：无；没有启动 Syn、Obsidian 或 CLI。
- 未访问任何真实 vault、其他 Obsidian vault 或真实项目；未请求安装、CLI 注册、权限、登录或付费操作。
- 暂存区在停点核对时为空；没有 stage、commit、push、reset、clean 或 stash。

下一步不是重试或绕过：指导线应先核对本证据与离线红契约；只有另包精确授权 host-owned relay 的数据流、写面和真实 App 条件后，才能重新进入 N6。
