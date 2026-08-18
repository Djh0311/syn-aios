# F3 M1 UNENROLLED 引导与状态投影

阶段：未来 `syn-shell` F3；当前不属于 stage-14 或 `syn-shell` 施工。

状态：`UNFINISHED` / `NOT_CURRENT` / `WAITING_FOR_F2_F3_AUTHORIZATION` / `SYN_SHELL_NOT_ACTIVE`。

来源：`M5R09-20260818-1836.verdict.md` 欠账 3。M5R09 已提供普通产品显式 enrollment command、现有布局内最小入口与操作后状态文案；首次启动的 `UNENROLLED` 尚未进入 snapshot/UI 的主动提示。该缺口不满足 18:40 纪律的“现在不修则普通产品对真实用户不可用”门，故不阻塞 M5 closeout，留给新壳 F3 正式消费。

未来做完的标准：

1. 新壳读取服务端权威 M1 enrollment 状态，不在前端猜测或按 path 派生。
2. `UNENROLLED` 在相关治理入口显示明确、可操作且不误导的登记提示；已登记、冲突、source 损坏和 authority unavailable 分型展示。
3. 登记只能由用户明确动作触发；不得自动导入、自动铸造身份或因提示而放宽 M1 业务 fail-closed。
4. 与 F3 acceptance-only 接线隔离；真窗口像素证据仍由 F5 另行获取。

本文件不授权进入 `/home/synadmin/workspace/syn-shell`、建立 F2/F3/F5 current leaf、运行真窗口、使用真实资料或发布。
