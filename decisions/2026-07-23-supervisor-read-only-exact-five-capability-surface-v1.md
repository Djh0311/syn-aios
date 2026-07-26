# 决策：`supervisor-read-only` 精确五项 MCP capability surface v1

- 日期：2026-07-23
- 状态：**ACCEPTED**
- 适用范围：共享 Conversation Transport 的项目主管 profile 与后续真实 App 验收

## 决策

当前 `supervisor-read-only + project_supervisor` 的 `tools/list` 验收面固定为以下五项，集合必须精确相等：

1. `submit_proposal`
2. `knowledge_search`
3. `knowledge_read`
4. `knowledge_open`
5. `knowledge_cite`

历史真实 App v1 包中“只看到 `submit_proposal`”的谓词，适用于知识 capability 接入前的快照；它不再用于新的验收。新验收不得为了满足旧谓词而隐藏四项知识能力或新造单工具 profile。

## 依据

- `decisions/2026-07-22-shared-conversation-transport-and-syn-mcp-capability-plane-v1.md` 已冻结 MCP 为整个 Syn 的统一能力层，并要求 profile/role 精确 allowlist。
- `docs/plans/2026-07-23-l3-syn-native-knowledge-workspace-small-stage-plan-v2.md` 的 N6 明确要求主管只读 `search/read/open/cite`。
- 当前 `mcp/capability_registry.rs` 只允许上述五项进入 `supervisor-read-only + project_supervisor`，并以精确集合测试拒绝空集、子集、重复项、未知项和变体名。

## 使用约束

- 五项只在可信 binding 为 Active、project/root/workflow/run/thread 全部一致时可见；`tools/list` 与 `tools/call` 必须走同一个服务端 gate。
- 三句对话替代性验收中，第一句只验证五项可见，不调用知识工具；第二句只允许调用一次 `submit_proposal`；第三句不得调用任何工具。
- 知识能力仍只读固定 Syn vault；不得公开 `knowledge_write`、`canvas_write`、`attachment_write`、通用 filesystem、shell、URL 或外部 App 能力。
- `knowledge_open` 是否返回 `opened=true` 仍受独立 host relay/UI ack 合同约束；工具“可见”不等于打开已成功。
- 本决策只纠正能力面和验收谓词，不授权修改代码、启动真实 App 或操作真实 store。
