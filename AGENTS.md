# Harness Lite 协作入口

先读 `docs/harness/plan.md`，再读 `stages/` 里的当前阶段，最后读 `leaves/` 里唯一的当前 leaf。`unfinished/` 放未开始、暂停或受阻的工作；`done/YYYY-MM/` 只归档已经完成的工作。根 README 只做介绍，不是执行授权。

开始或压缩后运行：

```text
node .claude/harness-lite/bin/hl.js chain --target <项目目录>
node .claude/harness-lite/bin/hl.js progress --target <项目目录>
node .claude/harness-lite/bin/hl.js auth --target <项目目录>
```

用户授权决定可以做什么，Harness 决定授权范围内怎么推进。只从 `docs/harness/authorization.json` 读取当前用户授权；没有 `authorize` 命令，模型不得写“用户已授权”给自己放行。同一 stage 授权可供主 agent 和子 agent 使用。

只有五类事进硬门：进入远端/服务器/生产/真实世界；删除或难恢复；改变或结束当前工作；修改 Harness 的守门、授权或审计；项目额外声明的真实凭据、设备等边界。明确授权已经覆盖的具体动作直接通过，不重复问；未授权才停。普通开发由模型判断并继续。

`leaves/` 必须恰好一个 current。未完成用 `hl park <leaf> <原因> --write` 放回 `unfinished/`，不能归档；整阶段授权下可用 `hl resume` 恢复。`hl done <leaf> --write` 是“当前 leaf 已完成”的流程声明，随后归档；整阶段授权会自动进入下一项，只授权当前 leaf 时则退出并答九件。

完成一个 leaf 先说四样：做出什么、验证跑了什么、改了哪些文件、遗留什么。最后一个 leaf 完成后运行 `hl close-stage --write`，用原整阶段授权只归档这个已完成 stage 并勾总计划，不恢复其他权限。随后停止并回答：入口、实际结果和验收、遗留与接手人、改动位置、是否并主线、分支/worktree、测试材料、下一入口，以及记录是否真的在提交里并列出该提交文件。

break-glass 只处理已经授权却被适配器误拦的动作，必须引用授权里的专用 grant、给出原因并写审计。不得记录原始命令、凭据或敏感参数。

这是轻量协作门。已注册的 Codex/Claude hook、CLI 和 Git hook 会在各自入口阻断，但同一系统账号可以停用或修改它们；不要把它说成防本机蓄意绕过的安全边界。push、服务器、生产、真实凭据和物理动作仍按项目授权边界处理。
