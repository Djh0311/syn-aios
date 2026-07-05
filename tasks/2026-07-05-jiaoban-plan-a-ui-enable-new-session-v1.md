# 实现任务包:方案a 前端解禁(「开个新的」从置灰变真能用)· UI 专线 v1

日期:2026-07-05　性质:**轻档·前端·极小**(`src-tauri` 0-diff;后端已落 `014c254`)。

## 0. 一句话

后端 `confirm_and_start_authorized_run` 的 `session_choice:"new"` 已真能用(先生后绑·真建会话~15-60s)。本包:`JiaobanSessionPicker` 里「开个新的」解禁(去 disabled/「下一阶段支持」标签),选它时合流请求传 `session_choice:"new"`、**不传 session_id**;"正在干"脸的等待文案在 new 时补一句「正在为这单活新建会话(约 1 分钟)…」(后端 outcome warnings 也会带一句,照常显)。existing 路径一行不动。

## 硬线

`src-tauri` 0-diff;不碰预拆/所批即所跑/画布;只动 SessionPicker + 请求组装 + 等待文案。

## 验收(真机·测试项目)

- 选「开个新的」→ 允许并开始 → 等待文案在 → 跑到交货;去智能体页能看到新生会话(带初始化消息 + worker 任务);
- 「接现有」回归不变;三闸绿;`git diff` 仅前端。

## 回交

改动 diff + 真机证据 → 主导线核。**子线不 commit。**
