# Syn M1 contract baseline

> 资料状态（2026-08-09）：M1 独立验收过的冻结工程合同基线。它约束点名对象、所有权和安全边界，但不定义 Syn 最终产品，不证明当前代码全部实现，也不提供后续阶段授权。当前产品、实现和施工分别看 `../product/syn-product-canon-v1.md`、`../current-state.md` 与当前轻量开发护栏链。

This directory is the independently accepted, frozen static contract baseline for `SYN-FND-001-R1`.
It freezes vocabulary, ownership, failure behavior, compatibility boundaries, opening
inventories, and the minimum input handed to M2. It does not switch a runtime source of
truth, create database schema, enable a connector, run the desktop application, or prove
production behavior.

## Evidence level

- `CANDIDATE_V1` means the contract text is under independent M1 review and is not yet frozen.
- `FROZEN_V1` means the contract text has passed independent M1 acceptance and is the accepted design baseline.
- `PARTIAL_LEGACY` means opening source contains related behavior, but not the complete
  target contract.
- `ABSENT` means the target primitive is not implemented at the opening baseline.
- `HOLD` means a named later owner must decide the item before implementation.
- Source counts, blobs, and SHA-256 values are tied to base OID
  `2bf9406bd688db8eb84d2138f9b3c6994dac2fb9`.

## Artifact map

- Ten `*-v1.md` files: cross-module contracts and action-flow examples.
- `manifest.v1.json`: contract registry, unique owners, exports, and dependency DAG.
- `source-opening-manifest-v1.json`: immutable opening-source evidence for 30 fixed-base source files.
- `entrypoint-inventory-v1.json`: 171 Tauri commands, eight Supervisor MCP capabilities,
  and the known runner/background entry set.
- `legacy-migration-inventory-v1.json`: legacy object/store/projection migration routing; canonical
  sidecars are source-derived and the seven contract projection categories have exact source anchors.
- `storage-opening-inventory-v1.json`: all 68 opening SQLite tables, 18 canonical sidecars,
  and seven derived projections with ownership, join, truth, and migration routing.
- `open-design-holds-v1.json`: explicit unresolved decisions and their next owners.
- `m1-test-matrix-v1.md`: M1 static acceptance map.
- `m2-shadow-write-parity-rollback-input-v1.json`: interfaces and safety invariants handed
  to M2 without premature persistence decisions.
- `fixtures/syn-fnd-001/*.json`: positive and negative mechanical cases.
- `verify-syn-fnd-001.mjs`: deterministic, offline verifier.

Run `node docs/contracts/verify-syn-fnd-001.mjs` from the repository root. The verifier
reads Git objects and repository files only; it does not start App, Vite, a browser, a
store, a workflow, a connector, or any credential path.
