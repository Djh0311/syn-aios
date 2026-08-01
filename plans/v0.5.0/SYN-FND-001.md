---
id: SYN-FND-001-R1
kind: TASK
parent-id: PL-SYN-PRODUCT-LINE
lifecycle: ACTIVE
goal: 冻结 Syn M1 十份跨模块合同，并以 171 个 Tauri command 的 opening 事实完成入口、存储、迁移、测试与 M2 输入基线；仅合同工件；catch:none
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
  - 全入口 inventory 覆盖 opening baseline 的 171 个 Tauri command、8 个 Supervisor MCP capability、runner、background job，并标注 owner、scope、policy、bypass 与迁移状态。
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
  wip-commit: 71cc7606efce69e7f75f17a9880116f5cb8c9611
  no-product-change: false
  disposition: TRANSFERRED
  integrated-observed: false
relations:
  - type: REPLACES
    target-id: SYN-FND-001
    note: CONTRACT_INVALID
confirmations: []
---

# SYN-FND-001-R1 合同与迁移基线

## 负责哪块
承接原任务因 Tauri 注册入口从 170 漏算为 171 而失效的合同工作，以 opening 源码集合为准冻结十份合同及迁移输入。

## 边界（允许读写、禁止）
### 允许读写
仅写 `docs/contracts/**` 与本 matching package 的合同纠正。

### 禁止
产品源码、产品测试、App、真实 store、消息、workflow、connector、credential，以及 push、merge、发布均不进入本任务。

## 交付什么
交付十份合同、171/8 全入口 inventory、存储与旧对象迁移矩阵、HOLD registry、M1 测试矩阵、M2 输入和纯静态 verifier/fixtures。

## 怎么验证
以任务 front matter 的四条 required verification 为准；所有结论限定为静态合同证据。

## 遇到什么必须停
opening source hash、入口集合、owner、写面、Git binding 或 active package 任一无法唯一核对时立即停止。
