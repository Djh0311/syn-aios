# S1B-H2-R2 Gate 1 build and binary freeze (metadata-only)

Command, from `prototypes/productized-desktop-shell`:

```text
../tauri-capability-probe/.tauri-cli/bin/cargo-tauri build --debug
```

- Exit: `0`.
- The seven task-package source hashes were rechecked after the build and still matched Gate 0.
- Frozen launch target (bare executable, not a bundle app):
  `/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell/src-tauri/target/debug/codex-governance-workbench`
- SHA-256: `5669ee86e7edcad3273efdf1f8ee8b7e3fcb5307fbc9767d019d7421b51cccbc`
- Size: `66503272` bytes; mtime epoch: `1784666565`.
- Exactly this bare executable was launched. Healthy startup advanced initialized `35 -> 36`; degraded remained `11`.

