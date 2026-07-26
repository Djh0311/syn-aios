# S1B-H2-R2 Gate 0 baseline (metadata-only)

- User-present retry began only after normal closure of the earlier accidental `.app` attachment and the bare binary. No acceptance message had been sent during that preflight; a fresh baseline was taken afterward.
- Relevant Workbench, resident Codex, Tauri/dev, Vite, and cargo-tauri process checks were empty. `workflow-state`, production DB, WAL/SHM (when present), and registry had no holder; registry entries were `0`.
- Product-line HEAD: `e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`; staged set empty. All seven task-package source SHA-256 values matched before Gate 1.
- JSON/DB reconciliation was green and SQLite integrity check was `ok`. Baseline: proposal `74`, Pending `17`, chain `40`; canonical target counts were recorded `8`, injected `3`, supervisor reply `3`.
- DB-primary health baseline: `storage_mode_initialized=35`, `storage_mode_degraded_json_only=11`.
- Fixed test project: HEAD `caa02ded684d9e1d92d00c367949fab6f83430d1`, complete non-`.git` manifest SHA-256 `f9c8867116851f688ee1311869c8703fd1f7f4f833cecd482eb42bb9115ad9a4`.

