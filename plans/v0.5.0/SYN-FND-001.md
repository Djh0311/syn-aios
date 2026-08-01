---
id: SYN-FND-001
kind: TASK
parent-id: PL-SYN-PRODUCT-LINE
lifecycle: ACTIVE
goal: 冻结 Syn M1 十份跨模块合同、全入口与存储迁移基线、测试矩阵及 M2 迁移输入；仅合同工件；catch:none
profile: STRICT_LOCAL
write-scope:
  - docs/contracts
  - plans/v0.5.0/SYN-FND-001.md
forbidden-scope:
  - prototypes/productized-desktop-shell/src-tauri
  - prototypes/productized-desktop-shell/tests
  - prototypes/productized-desktop-shell/scripts
  - prototypes/productized-desktop-shell/src/lib
  - prototypes/productized-desktop-shell/src/views
  - prototypes/productized-desktop-shell/src/styles.css
exclusive-resources:
  - docs/contracts
acceptance-criteria:
  - 十份版本化合同均包含唯一 owner、真源、合法状态、跨 scope 规则、command/event/audit/outbox、敏感字段、幂等、失败、回滚、兼容、fixture 与明确不做。
  - 全入口 inventory 覆盖 170 个 Tauri command、8 个 Supervisor MCP capability、runner、background job，并标注 owner、scope、policy、bypass 与迁移状态。
  - 旧对象与 store/table/sidecar/projection 迁移矩阵、M1 测试矩阵、M2 shadow-write/parity/rollback 输入及全部 HOLD 完整可追溯。
  - 合同 owner 依赖无环；每个正式动作可回答 command、policy、state、event、audit、outbox；secret/raw transcript/tool output 禁止字段通过机械 fixture。
  - 任务 diff 仅包含 matching package 与 docs/contracts；产品代码和产品测试零修改。
verification:
  - id: contract-verifier
    command: node docs/contracts/verify-syn-fnd-001.mjs
    required: true
    status: UNKNOWN
  - id: contract-diff-check
    command: git diff --check 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9..HEAD -- docs/contracts
    required: true
    status: UNKNOWN
  - id: product-source-unchanged
    command: git diff --quiet 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9..HEAD -- prototypes/productized-desktop-shell
    required: true
    status: UNKNOWN
  - id: task-scope-review
    command: git diff --name-only 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9..HEAD
    required: true
    status: UNKNOWN
git:
  base-branch: main
  base-oid: 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9
  task-branch: codex/syn-fnd-001
  worktree: /Users/yoyi/workspace/product-line-syn-fnd-001
  local-commit-allowed: true
  push-allowed: false
  product-commit:
  wip-commit:
  no-product-change: false
  disposition: RETAINED
  integrated-observed: false
relations: []
confirmations: []
---

# 任务

## 负责哪块
按 proposal 中冻结的目标执行。

## 边界（允许读写、禁止）
### 允许读写
只修改已声明 write-scope。

### 禁止
不触及 proposal 的 forbidden-scope。

## 交付什么
满足 proposal 中的 acceptance-criteria。

## 怎么验证
运行 proposal 中的 required verification。

## 遇到什么必须停
身份、范围、授权或 Git 现实无法唯一核对时停止；integrate、push、发布和物理清理仍需分别确认。
