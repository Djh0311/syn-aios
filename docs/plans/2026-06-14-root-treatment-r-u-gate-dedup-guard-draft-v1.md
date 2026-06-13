# Root Treatment / R-U-Gate Dedup Guard Draft v1

日期：2026-06-14

状态：草稿。本文只定义查重门形态和推荐路径，不实现、不接入。

Planning baseline：`2813058`

## 1. 目标

R-U 已把一批重复 util 收敛到公共模块，但如果后续开发仍靠人工记忆，重复 helper 会重新长出来。U-Gate 的目标是给后续“防复发”任务包提供清晰设计：

- 写新 helper 前先能发现已有公共 util。
- 对明显重复的 helper 给 warning 或 blocking。
- 不误伤业务特化函数，不把不同语义强行统一。
- 不在今晚接入 harness / CI，避免把草稿变成未复核实现。

## 2. 保护对象

首批只建议保护 R-U 已收敛的低风险 util：

- Rust hash：`utils/hash.rs` 的 `sha256_hex`、`sha256_hex_bytes`、`short_hash`、`short_hash12`。
- Rust store path：`utils/store_paths.rs` 的 `sidecar_path`。
- Rust fs ops：`utils/fs_ops.rs` 的 `remove_file_if_exists`、测试用 `fixture_dir`。
- Rust normalization：`utils/normalization.rs` 的 `normalize_slash_lowercase`。
- 前端展示 primitives：`src/components/WorkbenchPrimitives.tsx` 的 `SummaryTile`、`DetailLine`。

暂不保护：

- store 模式：`load_store` / `empty_store` / `validate_store` / `write_store_atomic`。
- 业务校验函数：`validate_*`、状态机、权限、runner、SQLite migration / read-cut / stop-write。
- 业务特化 normalization：C4 symbol、director review decision、mature pattern key、path canonicalization、敏感路径检测、alias / token / candidate key。

原因：这些函数即使名字或形状相似，语义通常不同，误合比重复更危险。

## 3. 方案 A：文档规则 + 任务包清单

形态：

- 在 R5 或治理规则文档中新增“新增 util 前查 `utils/` / `components/WorkbenchPrimitives`”规则。
- 任务包模板新增一项：若新增 helper，必须声明“是否已有公共 util，若不用，理由是什么”。
- 复核线人工核对新增 helper 是否绕过现有公共 util。

优点：

- 最低风险，不改脚本。
- 适合立即落地到任务包和复核口径。
- 对业务特化函数更友好，误报少。

缺点：

- 靠人执行，容易漏。
- 不能自动阻止重复 helper 进入代码。

适用：

- R3 Level B 前后需要低风险治理约束时。
- 对高风险 store / SQLite 相关代码，先用文档门比脚本硬拦更安全。

## 4. 方案 B：Shape Gate 轻量正则扫描

形态：

- 在 `workbench-shape-gate.js` 或独立脚本中加入只读扫描。
- 首批扫描明确重复模式：
  - `fn sha256_hex(` / `fn short_hash(` 出现在 `utils/hash.rs` 之外。
  - `fn sidecar_path(` 且函数体包含 `parent().ok_or_else(...).join(SIDECAR_NAME)` 的重复店内实现。
  - `fn remove_file_if_exists(` 出现在 `utils/fs_ops.rs` 之外。
  - `function SummaryTile` / `function DetailLine` 出现在 `WorkbenchPrimitives.tsx` 之外。
  - `fn normalize(value: &str)` 且函数体为 `trim().replace('\\', "/").to_lowercase()` 出现在 `utils/normalization.rs` 之外。
- 初期建议只 warning，不 blocking；连续两轮稳定后再考虑 blocking。

优点：

- 成本低，能嵌入现有 shape gate。
- 对本轮已治理的重复最直接。
- 适合防“同名同实现”回潮。

缺点：

- 正则容易漏掉换行 / 格式变体。
- 可能误报业务特化 wrapper，例如 `normalize_symbol` 这种保留语义名的 wrapper。
- 如果直接 blocking，可能阻断合理局部 helper。

适用：

- 首批自动化防复发。
- 只针对已明确稳定的 util，不扫描 store 模式或业务校验。

## 5. 方案 C：AST / 指纹查重门

形态：

- 用 Rust 解析器或轻量 token normalization 对函数体生成指纹。
- 对 TypeScript 组件也可做 AST 或文本归一指纹。
- 与公共 util 指纹库比对，输出：
  - exact body duplicate
  - near duplicate
  - same name different body
  - same body different semantic wrapper
- 允许在配置中标记 deferred / allowed wrappers，例如 `control_core::normalize_symbol`。

优点：

- 比正则稳，能覆盖格式变化。
- 能把“同实现不同名称”和“同名不同实现”分开。
- 未来可扩展到成熟模式 / mistake-check / task-finish。

缺点：

- 实现复杂度高。
- 需要维护 allowlist 和指纹基线。
- 如果放进当前 Stage R，容易变成新的治理项目，拖慢 R3 Level B。

适用：

- R3 Level B、R5 收口之后作为后续治理硬化。
- 当重复 helper 再次出现，且正则门不足时升级。

## 6. 推荐路径

推荐采用分两段策略：

1. 短期采用方案 A + 方案 B warning-only。

   先把文档规则写进 R5 / task package lint 口径，再单独开一个小任务包把轻量正则扫描接入 shape gate 的 warning 区。warning-only 的理由是当前仍有 deferred / wrapper / 业务特化 normalize，直接 blocking 误伤概率偏高。

2. 中期视复发情况升级方案 C。

   如果 R3 Level B 后继续出现重复 helper，再开独立治理包做 AST / 指纹门。这个包必须自带 allowlist、baseline、fixtures 和不误伤业务特化的测试。

不推荐今晚直接做：

- 不推荐立刻改 `workbench-shape-gate.js`。
- 不推荐把查重门作为 blocking 接入。
- 不推荐扫描 store 模式或业务 `validate_*`。
- 不推荐把 `normalize` 全部统一，因为 U4 已证明不同 normalization 有真实语义差异。

## 7. 后续任务包建议

### U-Gate-A：文档规则接入

目标：

- 把“新增 helper 前查公共 util”写入 R5 文档对齐或任务包模板。
- 在复核清单加入“新增 helper 是否已有公共 util”。

验证：

- 文档 diff。
- 复核线确认没有声称脚本已接入。

### U-Gate-B：Shape Gate warning-only 扫描

目标：

- 在 shape gate 或独立 harness check 中加入首批正则扫描。
- 首轮只 warning，不 blocking。
- 加 fixtures 或最小测试，覆盖已治理 util、allowed wrapper、deferred normalization。

验证：

- `node scripts/harness/workbench-shape-gate.js --mode check`
- 相关 harness 测试。
- `git diff --check`

### U-Gate-C：AST / 指纹门研究

目标：

- 评估 Rust / TS AST 或 token 指纹的可行性。
- 明确误报处理和 allowlist 格式。
- 不与 R3 Level B 同时推进。

验证：

- 只读扫描报告或 prototype，不接 blocking gate。

## 8. 接受标准

后续真正实现 U-Gate 时，最低接受标准应为：

- 不读取 secrets、`.codex`、private transcript。
- 不执行真实 Codex。
- 不修改业务代码行为。
- 不把 deferred / business-specific helper 当重复强合。
- 每条 warning 有文件、行号、匹配规则、建议公共 util 和 allow / defer 理由。
- 默认 warning-only，blocking 需另行决策。

## 9. 本草稿不接受为

本文不接受为：

- U-Gate 已实现。
- shape gate / harness / CI 已接入查重门。
- 重复 helper 已自动阻断。
- R-U 全部防复发已完成。
- R3 Level B、SQLite 真实切换、R5 文档对齐或 Stage R 收口完成。
