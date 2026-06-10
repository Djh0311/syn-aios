# 索引内核边界异常补测证据

## 结论先说

薄弱点：

- 本轮没有修改 `build_index.py`。依据：任务包允许写入不包含内核脚本，本轮只补测试和验证文档。
- 权限拒绝只覆盖了 `.codex-global-state.json` 读取失败，不等于覆盖所有路径的权限拒绝。依据：测试通过 `chmod 000` 触发 `read_failed:<path>:PermissionError`。
- 大 JSONL 测试证明正文没有进入索引输出，但 `session_index.jsonl` 仍会逐行 `json.loads()`。依据：`parse_session_index()` 只取 `id`，但会解析整行 JSON。
- 本轮开始时执行过一次 `/Users/yoyi/.codex` 目录 `stat` 元数据读取，没有读文件内容、没有写入。后续测试和验证命令没有再读取真实 `.codex`。

可用结果：

- 新增边界测试：`product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`。
- 原 12 个测试仍通过。
- 新增 5 个边界测试通过。
- 联合运行 17 个测试通过。
- 测试命令不依赖网络。
- 测试夹具全部在 `tempfile.TemporaryDirectory()` 里创建，不把夹具放进 `/Users/yoyi/.codex`。

## 本轮读取范围

按任务包读取：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/evidence/2026-05-27-index-kernel-hardening.md`
- `product-line/handoffs/2026-05-27-index-kernel-hardening-result.md`
- `product-line/handoffs/2026-05-27-index-kernel-hardening-review.md`

补充说明：

- 读取了任务包 `product-line/tasks/2026-05-27-index-kernel-edge-validation.md` 和任务队列 `product-line/tasks/README.md`，用于确认目标和状态。
- 本轮没有读取或打印 `auth.json`、`.env`、密钥、令牌。
- 本轮没有读取会话正文、工具输出、命令输出、输入历史或记忆正文。

## 本轮写入

- `product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py`
- `product-line/evidence/2026-05-27-index-kernel-edge-validation.md`
- `product-line/handoffs/2026-05-27-index-kernel-edge-validation-result.md`

未修改：

- `product-line/prototypes/index-kernel/build_index.py`
- `product-line/prototypes/index-kernel/tests/test_build_index_failures.py`
- `product-line/prototypes/index-kernel/codex-index.json`

## 新增测试

新增测试文件：

```bash
product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

新增 5 个测试：

- `test_corrupt_sqlite_file_degrades_with_sqlite_warning`
- `test_unreadable_global_state_records_warning_or_skips_when_permissions_are_not_enforced`
- `test_rollout_symlink_inside_sessions_to_outside_file_is_blocked`
- `test_large_rollout_jsonl_body_is_not_opened_or_serialized`
- `test_large_session_index_payload_is_not_serialized`

测试夹具做法：

- 用 `tempfile.TemporaryDirectory()` 创建临时假 Codex home。
- 用 `IndexSources.from_codex_home(self.codex_home)` 指向临时假目录。
- SQLite、session_index、rollout、global state、skills、plugins、memories 都在临时目录内生成。
- 不访问真实 `/Users/yoyi/.codex` 下的测试夹具。

## 边界场景结果

已覆盖并通过：

- SQLite 损坏文件：把 `state_5.sqlite` 写成非 SQLite 字节，索引器不崩，返回空线程，warning 以 `sqlite_open_failed:` 或 `sqlite_read_failed:` 开头。
- 权限拒绝：把临时假 `.codex-global-state.json` chmod 为 `000`，当前环境触发 `read_failed:<path>:PermissionError`，索引器不崩，`global_state.loaded=false`。
- rollout 符号链接绕过：在允许的 `sessions/` 目录内放 symlink，目标指向允许目录外文件；`is_relative_to(resolve)` 阻断，统计 `outside_allowed_session_dirs=1`，线程 warning 为 `rollout_path_outside_allowed_session_dirs`。
- 大 rollout JSONL 正文：创建包含 `EDGE_ROLLOUT_BODY_SHOULD_NOT_APPEAR` 的大 rollout 文件，并用 mock 防止该文件被 `Path.open()` 打开；索引器只做 `exists()`，不读正文，sentinel 没进入索引输出。
- 大 session_index JSONL：创建包含大 `first_user_message`、`preview`、`payload.content` 的 JSONL 行；索引器只把 `id` 纳入统计，sentinel 没进入索引输出。

无法稳定模拟：

- 本轮没有无法稳定模拟的测试项。权限拒绝在当前环境中可稳定触发；测试里仍保留 skip 分支，原因是不同用户权限模型可能绕过 `chmod 000`。

## 验证命令和结果

原 12 个测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py
```

结果：

```text
Ran 12 tests in 0.024s
OK
```

新增边界测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

结果：

```text
Ran 5 tests in 0.012s
OK
```

联合测试：

```bash
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

结果：

```text
Ran 17 tests in 0.031s
OK
```

语法检查：

```bash
python3 -m py_compile product-line/prototypes/index-kernel/build_index.py product-line/prototypes/index-kernel/tests/test_build_index_failures.py product-line/prototypes/index-kernel/tests/test_build_index_edge_cases.py
```

结果：通过，命令退出码为 0。

## 不读取或写入真实 Codex 目录

依据：

- 测试文件用 `self.codex_home = self.fixture_root / "fake-codex-home"`。
- 测试文件用 `build_index.IndexSources.from_codex_home(self.codex_home)` 注入临时数据源。
- 验证命令只运行测试和语法检查，没有运行真实环境索引生成。
- 测试里所有 SQLite、JSONL、manifest、skill、global state、rollout 文件都在临时目录里创建。

边界：

- 本轮开始时曾执行一次 `stat -f '%m %N' /Users/yoyi/.codex`，只读取目录元数据，没有读取文件内容，也没有写入。后续没有继续用真实 `.codex` 做验证。

## 是否建议修改索引内核

不建议因本轮通过项立即修改内核。

依据：

- SQLite 损坏、global state 权限拒绝、rollout symlink 越界、大 rollout 正文不读取、大 session_index payload 不输出，都已有降级或阻断表现。

仍建议后续评估：

- `parse_session_index()` 对超大 JSONL 行仍会整行解析。若未来 session_index 可能变得很大，建议增加行长上限或只抽取必要字段。
- 权限拒绝只在 global state 上验证过，后续可扩展到 plugin manifest、skill、memory 路径。
- `rollout_path.exists()` 会跟随 symlink，但当前在 `exists()` 前已经用 `resolve` 归属检查阻断了越界 symlink；建议保留这个顺序。

## 风险和下一步

风险：

- 大 rollout 测试用 mock 阻止 `Path.open()`，能证明当前代码路径没有打开 rollout 文件；如果未来改用其他 API 读正文，这个测试可能需要同步加强。
- 大 session_index 测试只证明正文不被序列化进索引，不证明解析成本可控。
- 权限拒绝在不同系统或用户权限下可能表现不同，测试保留 skip 分支是为了避免伪失败。

下一步建议：

- 把新增边界测试纳入索引内核固定验收命令。
- 若进入性能验证线，再单独测超大 `session_index.jsonl` 的时间和内存，不和本轮异常补测混在一起。
- 若进入安全验证线，再补符号链接链、目录 symlink、plugin skill symlink 的绕过测试。
