---
id: SYN-FND-001
kind: TASK
parent-id: PL-SYN-PRODUCT-LINE
lifecycle: HISTORY
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
    status: BLOCKED
    run:
      head-oid: 71cc7606efce69e7f75f17a9880116f5cb8c9611
      exit-code: 1
      output-ref: node:#raw-output-contract-verifier
      command-digest: sha256:a7364c239cc8ccbf2f853f9ec17825d96d915327cafe9dd8a14d186e52b60497
      test-paths: ALL_TRACKED
  - id: contract-diff-check
    command: git diff --check 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9..HEAD -- docs/contracts
    required: true
    status: PASS
    run:
      head-oid: 71cc7606efce69e7f75f17a9880116f5cb8c9611
      exit-code: 0
      output-ref: node:#raw-output-contract-diff-check
      command-digest: sha256:9a439321694d145f1f1c0f12401149878ad86d0a8ed1ecc42e81861a460f8c64
      test-paths: ALL_TRACKED
  - id: product-source-unchanged
    command: git diff --quiet 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9..HEAD -- prototypes/productized-desktop-shell
    required: true
    status: PASS
    run:
      head-oid: 71cc7606efce69e7f75f17a9880116f5cb8c9611
      exit-code: 0
      output-ref: node:#raw-output-product-source-unchanged
      command-digest: sha256:9e62f6378780a8e10735f89f2b01b4587c019702568f181885c707fad9dbfc91
      test-paths: ALL_TRACKED
  - id: task-scope-review
    command: git diff --name-only 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9..HEAD
    required: true
    status: PASS
    run:
      head-oid: 71cc7606efce69e7f75f17a9880116f5cb8c9611
      exit-code: 0
      output-ref: node:#raw-output-task-scope-review
      command-digest: sha256:9087d8112149e4071372288a96ab1024d5656244c603fb35aa1bc53efd85c8d1
      test-paths: ALL_TRACKED
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
relations: []
result: SUPERSEDED
closed-at: 2026-08-01
confirmations:
  - action: local-integrate
    target: refs/heads/main
    task-branch: codex/syn-fnd-001
    task-worktree: /Users/yoyi/workspace/product-line-syn-fnd-001
    integration-worktree: /Users/yoyi/workspace/product-line-syn-integration-main
    integration-oid: 2bf9406bd688db8eb84d2138f9b3c6994dac2fb9
    consumed: true
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

### raw-output-contract-verifier

    BLOCKED: docs/contracts verifier 尚不存在；更早的 opening 静态复核同时证明激活包写 170，但同一 base blob 实际注册 171/171 unique，因此原合同必须替换，不能继续伪造通过。

### raw-output-contract-diff-check

    PASS: opening HEAD 对 docs/contracts 没有 diff，也没有 whitespace error。

### raw-output-product-source-unchanged

    PASS: opening HEAD 相对冻结 base 的产品源码 diff 为空。

### raw-output-task-scope-review

    PASS: base..HEAD 仅含 plans/v0.5.0/SYN-FND-001.md opening package。

## 退场记录

### 结果

SUPERSEDED

### 验收标准逐条结论

- 不适用

### 终止原因

Opening baseline 的 Tauri generate_handler 注册表实际为 171 条且全部唯一；原任务合同写成 170，合同根本事实失效，必须由纠正后的 successor 接续。

### 验收结论

主管已核验 170 与 171 的差异来自漏计最后一条 store_hygiene::sweep_canvas_run_residue；旧任务不作 M1 完成验收，仅保留 opening 与四条验证记录并转交给 SYN-FND-001-R1。

### 未决问题

- {"id":"TAURI-ENTRYPOINT-COUNT-CORRECTION","status":"TRANSFERRED","owner":"SYN-FND-001-R1","summary":"按 171 个 Tauri command 的 opening 集合重建合同 inventory 与 verifier。"}

### 测试与证据资产摘要

- 无

### Verification 追溯引用

- node:#raw-output-contract-verifier
- node:#raw-output-contract-diff-check
- node:#raw-output-product-source-unchanged
- node:#raw-output-task-scope-review

### Git 与资源去向

- disposition: TRANSFERRED
- product commit: NO_PRODUCT_CHANGE
- WIP commit: 71cc7606efce69e7f75f17a9880116f5cb8c9611
- retention owner: 不适用
- retention reason: 不适用
- retention review-by: 不适用

### 下一入口

successor SYN-FND-001-R1
