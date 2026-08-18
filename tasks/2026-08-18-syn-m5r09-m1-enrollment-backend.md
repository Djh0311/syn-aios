# Grok 窄任务包：M5R09 M1 普通产品登记后端

状态：`READY_FOR_SINGLE_WRITER`
写者：只允许 `grok -m grok-4.6 --reasoning-effort high` 这一轮进程
上位 leaf：`docs/harness/leaves/M5R09-m1-enrollment-and-pre-closeout-hardening.md`
增补合同：`docs/contracts/m1-project-enrollment-addendum-v1.md`

## 目标

只完成 M5R09 标准 1 的后端部分：普通产品缺少 ordinary identity source 时可以启动到“未登记但所有 M1 业务写 fail-closed”的可恢复状态；新增一个普通产品真实注册的 enrollment command。command 必须从服务器固定 product index 精确确认项目来源，持久化可重放 source 后再 materialize M1 registry，并对重复登记/重启保持同一 canonical `ProjectId`。

本包不做前端按钮、不处理 nested legacy row、不迁既有 governance tests、不处理 duplicate-effect，不改 Harness/报告/合同。

## 唯一允许修改的产品源码

- `prototypes/productized-desktop-shell/src-tauri/src/m1_project_index.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/lib.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/commands.rs`
- `prototypes/productized-desktop-shell/src-tauri/src/command_registry.rs`

其中 `commands.rs` 在开包前已有用户未归属 working-tree WIP；必须基于当前字节做最小插入，不覆盖、格式化整文件或整理无关 diff。不得修改、删除或重写其他产品文件。不要 `git add`、`git commit`、reset、stash、clean。

## 必须实现

1. 在 M1 authority 内提供显式 ordinary enrollment 能力，复用现有 source v1 schema：
   - exact alias = 服务端确认的固定 product-index `project_root`；
   - source ref = `product-index:<exact-root>`；
   - source 先原子持久化并 sync，再 replay registry；
   - 同 alias/source 重放不增 revision/entry/id；冲突 fail-closed；并发不产生重复。
2. ordinary Tauri constructor 只把“source 缺失 + registry 从未 established”识别为可恢复未登记；损坏、unsupported、unreadable、symlink/非普通文件、established registry 丢失或损坏仍返回原固定错误。未登记状态不得创建 source、registry 或 marker。
3. 新 command 输入只接受 `project_root`。服务端重新读取 `state.product_index_path()`，必须恰好匹配一条 `ProjectRecord.project_root` 才能调用 M1 authority；零/多条在 M1/source 写入前拒绝。调用方不得提供 canonical id、source path/revision/entry id。
4. command 注册进普通 `workbench_command_handler!`，输出至少包括 canonical project id、exact root、source revision、registry revision 与 `created|already_enrolled` 等幂等结果；错误保持稳定机器码或现有错误码。
5. 不从 path 派生 canonical id，不自动调用 enrollment，不静默创建 identity source，不让 UI/renderer 获得 registry/source 文件路径。

## 必须新增/调整的直接测试

- 首次无 source 的 ordinary AppState 能构造；此时 authority resolve 仍 unavailable/unknown，source/registry/marker 均不存在。
- 显式 enrollment 首次创建 source + registry，返回 opaque UUID；同一请求重复与 AppState 重建后返回相同 id，source/registry revision 不增长。
- source 已写而 registry materialize 中断的可重放恢复路径。
- product index 无匹配、重复 root、损坏 source、symlink/非普通 source、established registry 丢失/损坏均在写前 fail-closed。
- 静态/调用图测试证明 enrollment command 已注册，且 ordinary startup/preview/consumer 不自动调用它。

测试名统一使用 `m5r09_m1_enrollment_` 前缀，便于定向运行。

## 交付前验证

在 `prototypes/productized-desktop-shell/src-tauri` 运行并原样汇报命令、exit code、passed/failed 数字：

1. `cargo fmt --check`
2. `cargo check --lib --offline`
3. `cargo test --lib --offline m5r09_m1_enrollment_ -- --test-threads=1`
4. `cargo test --lib --offline m1_project_index_ -- --test-threads=1`
5. `cargo test --lib --offline m5_ -- --test-threads=1`
6. 仓库根运行 `git diff --check --` 加本包 4 个产品路径

完整 `m5_` 矩阵是本包交节点前的必跑项，不能省略或以定向套件替代。若仓库既有 warning 不影响 exit 0，按事实报告；任一失败不得声称完成。

## 交付格式

- 改了什么，逐文件说明；
- 新行为的 fail-closed/幂等顺序；
- 测试命令、exit code、计数；
- 明确仍未做的前端、nested legacy、canonical test migration 与其他 M5R09 标准；
- 不自评 leaf/stage/M5 通过，不写节点请求。
