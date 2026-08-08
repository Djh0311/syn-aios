# PLH002 隔离代码收敛演练报告

日期：2026-08-08

## 结果

在一次性 detached worktree `/private/tmp/product-line-harness-lite-code-rehearsal-puHRSv` 中，以 `main@e5269557` 为起点执行：

```text
git merge --no-commit --no-ff syn-fnd-002-dev
```

Git 自动合并成功并按要求停在 commit 前：0 个冲突、0 个 unmerged entry、65 个 staged 文件。候选 tree 为：

```text
04dbe9e7ae546e55697b381d8ca9f2f83e94a5c9
```

这证明完整合入 FND 已提交链是比选择性 cherry-pick 更小、更可复现的路径。候选包含 FND 链开头两项旧 Harness 适配；它们留给 PLH004/PLH005 统一退出，不作为产品验收。

## 验证

- `git diff --cached --check`：PASS。
- `cargo check --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib --offline`：exit 0，693 warnings，0 error。
- `cargo test --manifest-path prototypes/productized-desktop-shell/src-tauri/Cargo.toml --lib --offline`：1347 passed、1 failed、45 ignored，exit 101。
- 唯一失败：`s1b_h2_real_initial_and_resume_consume_only_the_private_submit_proposal_config`，错误是沙箱读取 PID `lstart` 返回 `Operation not permitted`。
- 同一测试在授权通道精确重跑：1 passed、0 failed，证明失败属于进程观察权限环境，不是 merge 引入的代码回归。

没有运行 real Codex、真实 App、provider、数据库、浏览器、部署、发布或 push。

## PLH003 精确输入

- 目标：`codex/product-line-lite-migration@e5269557`。
- 来源：`syn-fnd-002-dev@2a7229b`。
- 动作：完整本地 merge，不带 `codex/syn-m2-g0-b2-harness-i5-binding-repair` 的后续 3 个提交。
- 期望候选 tree：`04dbe9e7ae546e55697b381d8ca9f2f83e94a5c9`。
- merge 后复跑 `git diff --check`、non-test build 和 lib tests；环境失败须按本报告的精确单测复核。
- 两个 dirty worktree 继续以 PLH001 指纹保护，不搬入迁移分支。

## 回滚

PLH002 只使用 detached 临时 worktree，没有提交、没有移动真实分支。验收完成后通过 `git worktree remove` 删除该临时 worktree；共享 refs、main 和既有 worktree 应保持不变。

实际回滚已完成：先撤销未提交 merge，再移除 `/private/tmp/product-line-harness-lite-code-rehearsal-puHRSv`；该路径和 worktree 注册均已不存在。

回滚后再次核对两处受保护 WIP：

- `product-line`：status `9de7a6ac...98b`、tracked diff `a9b688c3...6fad`、untracked 内容清单 `0214a578...941`。
- `product-line-syn-fnd-002`：status `60f1395f...f399`、tracked diff `996452ac...b2e`、untracked 内容清单 `00ea3446...29a`、staged 0。

以上均与 PLH001 冻结值一致。
