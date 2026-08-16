# D0C05 重启后自动恢复与长期稳定验收

阶段：stage-12 阶段12 Syn 5600X/WSL C2 长期 SSH 开发通道
目标：在用户指定维护窗口中验证长期 SSH 通道在 WSL/Windows 重启或重新登录后能自动恢复。
干完的标准：另获精确重启与必要自动启动机制授权后，完成重启前冻结、受控重启、服务/规则/portproxy 恢复检查和 Mac 全新 SSH 登录；签发 `PERSISTENT_SSH_RESTART_STABLE`。未实际重启不得归档。

允许动：

- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-12.md
- docs/harness/leaves/D0C05-syn-5600x-wsl-persistent-ssh-restart-validation.md [届时由 lifecycle 恢复]
- docs/harness/unfinished/D0C05-syn-5600x-wsl-persistent-ssh-restart-validation.md
- docs/harness/done/2026-08/D0C05-syn-5600x-wsl-persistent-ssh-restart-validation.md [新增]
- docs/harness/done/2026-08/stage-12.md [新增]
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn

## 步骤

1. 等 D0C04 完成后保持 parked，等待用户指定维护窗口和精确重启/自动启动授权。
2. 冻结当前 SSH、firewall、portproxy、WSL/systemd 和 Mac 连通状态。
3. 只执行获准的 WSL shutdown/restart、Windows reboot/login 或唯一命名自动启动机制。
4. 重启后重新核服务、规则、portproxy 和 Mac SSH；任何漂移先停止并回滚，不扩大方案。

未完成原因：等待 D0C04 完成以及用户指定维护窗口；当前禁止重启和计划任务
