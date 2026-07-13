# 任务包:竞态测试修复——real_process_timeout 收割测试去不确定性(小包)v1

日期:2026-07-14 · 档位:**轻档**(仅测试代码)· 基线 commit `9cb5d13` · 基线 909-920/45 家族(该测试为唯一 flaky 源之一)。

## 背景(总指导 07-13 夜实测定性)

`codex_local_runner::tests::real_process_timeout_kills_and_reaps_mock_child` **独跑也偶挂**(panicked at codex_local_runner.rs:1982;同晚独跑 2 过 1 挂 2 过),定性=本体竞态非纯负载。已知温床:
1. **固定 temp 路径**:`:1900` 附近 `std::env::temp_dir().join("workflow-node-real-run-last-message.txt")`——跨次运行/并行套件共享同一文件,脏残留互踩($TMPDIR 已见 13 个测试残留);
2. **kill/收割计时窗**:~2s 竞态,断言对真进程退出时序过紧。

## 目标

1. 固定 temp 路径 → **每次运行唯一**(进程 id+纳秒或 tempdir 句柄),用完清理;
2. 计时断言 → **轮询式**(poll until 条件或超时预算,预算放宽到明确上限),不赌单次 sleep;
3. 连跑验证:该测试 `--test-threads` 默认与单跑各 **20 连发全绿**,并在全量套件里 3 连发不再出现该失败。

## 红线(此文件=沙箱,格外严)

1. **只许动 `#[cfg(test)] mod tests` 区域内该测试及其直接 helper;生产代码零 diff**(`git diff` 里非测试区一行不许有——codex_local_runner=沙箱文件,0-diff 死线意识);
2. 该文件在 fmt 历史漂移名单:**不整文件 rustfmt**,只保证新改动行不加剧漂移;
3. 不动其它测试;不碰 live 根;不 commit。

## 验收

20+20 连发全绿+全量 3 连发无此失败;`git diff` 证明只有测试区;回传 10 项(第 7 项 shape gate 必报——缺=打回,见 mistake-ledger M-2026-07-13)。
