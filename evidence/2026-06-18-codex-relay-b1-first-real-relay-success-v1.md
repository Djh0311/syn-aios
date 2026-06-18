# 里程碑：甲·中转 relay ③b-1 第一次真发 codex 成功 v1

日期：2026-06-18
性质：**里程碑记录——Syn 第一次真启动 Codex 干活**。
执行：用户 2026-06-18 在场明确授权（选「授权·我替你跑」），咨询线（Claude）用 Bash 跑 ③b-1 真发入口。

## 结果：成功

- 命令：`MANUAL_RELAY_REAL_CODEX_CONFIRM=CONFIRMED_USER_PRESENT_REAL_RELAY cargo test --lib manual_relay_b1_real_codex_runner_entry_requires_user_present_env -- --ignored --nocapture`（无沙箱，真执行）。
- 结果：`test result: ok. 1 passed`，**用时 32.75s**（真调模型的延迟；mock 路径为 0.02s）。
- **实物铁证**：codex 真在临时项目创建文件 `/var/folders/nj/.../T/manual-relay-b1-real-fixture-b393a89872e98183/hello.txt`，**内容 `hi`**，改动时间 **Jun 18 14:47:35 2026**（即时）。

## 这意味着

- **甲·中转真活了**：Syn 第一次真把一条指令送进 Codex → codex 真启动 → 真干活（创建文件）→ 回执验证通过。从"看得见、动不了"破局。
- relay 阶梯第一级（甲·中转）的"真执行"腿打通。

## 安全边界（这次真发守住）

- 打**临时项目**（`temp_dir` + `git init`）、**最小无害 prompt**（创建 hello.txt 写 hi），**未碰 mariotest / 任何真项目**；
- 用户在场明确授权；可观察（实物文件）、可回滚（temp）；
- 旧闸 5 文件未动、未解锁通用真实执行、未接前端（仍 `#[ignore]` test 入口、非 GUI）；
- codex 由其 CLI 自己跑、用其自身 `.codex` 认证（等价于用户直接用 codex）。

## 边界 / 下一步

- 这是 **③b-1**（test 入口 + temp + fixture prompt）的真发验证；**不是 GUI 真用、不是发 mariotest**。
- 下一步候选：**③b-2**（前端接"真发"按钮，GUI 真用）/ 发 mariotest（真用体感）。
- 不得据此声称：GUI 已真用 / mariotest 已真发 / 通用真实执行已解锁 / 乙·自动连环已开。
