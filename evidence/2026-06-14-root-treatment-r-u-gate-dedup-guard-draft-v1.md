# Root Treatment / R-U-Gate Dedup Guard Draft v1 Evidence

日期：2026-06-14

状态：草稿完成，独立复核 Hilbert `STATUS: CLEAR`。

Planning baseline：`2813058`

Task package commit：`599a35d docs: add r-u-gate dedup guard draft package`

## 1. 完成内容

本包只写 U-Gate 查重门方案草稿：

- 新增 `docs/plans/2026-06-14-root-treatment-r-u-gate-dedup-guard-draft-v1.md`。
- 列出三种形态：
  - 方案 A：文档规则 + 任务包清单。
  - 方案 B：Shape Gate 轻量正则扫描，首期 warning-only。
  - 方案 C：AST / 指纹查重门，中期研究。
- 推荐路径：短期采用方案 A + 方案 B warning-only，中期视复发情况升级方案 C。

## 2. 边界确认

本包没有：

- 修改 harness / shape gate / CI / pre-completion / task-finish 脚本。
- 新增查重扫描脚本。
- 修改源码、测试、CSS、Rust/Tauri/DB。
- 实现或接入 U-Gate。
- 进入 R3 Level B。
- 执行真实 `codex exec` / `codex exec resume`。
- 读写 `/Users/yoyi/.codex`。
- 解冻 backlog。

## 3. 验证

### 3.1 `git diff --check`

执行结果：通过，输出为空。

### 3.2 当前 git 状态

```text
?? docs/harness-script-audit-2026-06-14.md
?? docs/plans/2026-06-14-root-treatment-r-u-gate-dedup-guard-draft-v1.md
?? evidence/2026-06-14-root-treatment-r-u-gate-dedup-guard-draft-v1.md
```

说明：`docs/harness-script-audit-2026-06-14.md` 是本包开始前已存在的外部未跟踪文件，未读、未改、未纳入提交。

## 4. 独立复核

复核线：Hilbert (`019ec1e4-f49c-7d90-9166-ffd8b1bb1a42`)

结论：`STATUS: CLEAR`

复核线确认：

- 草稿包含 2-3 种查重门形态。
- 草稿给出推荐方案和理由。
- 草稿明确不实现、不接入。
- 未修改 harness / CI / 源码。

## 5. 不接受为

本包不接受为 U-Gate 已实现、shape gate / harness / CI 已接入查重门、R-U 全部自动防复发完成、R3 Level B 执行、R5 文档对齐、真实 Codex 执行、`.codex` 读写或 backlog 解冻。
