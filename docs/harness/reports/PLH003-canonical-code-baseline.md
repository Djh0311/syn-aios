# PLH003 唯一候选代码基线报告

日期：2026-08-08

## 结果

在 `codex/product-line-lite-migration` 上完整合入 `syn-fnd-002-dev@2a7229b`，生成本地 merge commit：

```text
c9d53a9063a076d409239a35cdd12e25332c1869
```

两位父提交分别为 `main@e5269557` 和 `syn-fnd-002-dev@2a7229b`。提交 tree 为 `04dbe9e7ae546e55697b381d8ca9f2f83e94a5c9`，与 PLH002 隔离演练冻结值完全一致，因此没有把 Lite untracked 材料、dirty WIP 或额外提交混入代码基线。

这一步收敛 65 个已提交文件、`+12784/-185`。FND 链中夹带的两项旧 Harness 适配仍在树里，统一留给 PLH004/PLH005 退出；后续 I5 repair 三提交没有合入。

## 验证

- `git diff --check HEAD^1..HEAD`：PASS。
- merge 后索引：0。
- `cargo check --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib --offline`：PASS，693 warnings、0 error。
- `cargo test --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib --offline`：PASS，1348 passed、0 failed、45 ignored。
- 未运行 real Codex、真实 App、provider、数据库、浏览器、部署、发布或 push。

## 受保护现场

- `/Users/yoyi/workspace/product-line` 的完整 status 指纹仍为 `9de7a6ac...98b`。
- `/Users/yoyi/workspace/product-line-syn-fnd-002` 的完整 status 指纹仍为 `60f1395f...f399`，staged 仍为 0。
- main、两个 dirty worktree 和远端均未移动。

## 下一入口

PLH004 只在隔离副本演练唯一项目权威和旧 Harness 卸载，不直接删除真实迁移分支中的旧 Harness 文件。需要先解决 `.claude/` 当前被项目 `.gitignore` 忽略、Lite 宿主 hooks 尚未接线，以及 AGENTS/CLAUDE 既有项目规则如何在 Lite 路由下保留。
