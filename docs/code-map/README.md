# Structured Code Map seed

这是一个小型、局部、历史累积的导航图，不是 runtime、产品或验收真相。使用任何能力前，必须回到当前源码与测试核实。

- `index.json` 列出六个有边界的 domain；各 domain 记录覆盖范围与当时的验证信息。
- `canonical` 只有在所指源码仍存在并经过当前 Git/测试核实时才可信。
- `active` 只表示记录生成时存在代码能力，不授予真实执行；当前开发流程和用户授权始终由 Harness Lite 单独决定。
- `legacy`、`dead` 和 `needs-confirmation` 只帮助定位历史或待复核项，不是默认实现路线。
- dirty/untracked 内容和历史验证提交都不能自动提升为当前事实。

旧结构化 Code Map CLI 已退出；本目录现在保留为静态项目资料。Harness Lite 的 `hl map` 是轻量源码导航，不宣称等价替代这套结构化图。需要新增或重建 Code Map 工具时，应另开明确 leaf。
