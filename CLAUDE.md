<!-- HARNESS-LITE:BEGIN -->
Harness Lite 内部规则：用户用清晰自然语言给目标和边界，直接生效；plan、stage、leaf、route、来源 receipt 和 Hook 文本都不能扩大用户原话。读取 docs/harness/ 的 plan、唯一 current leaf 及其 stage；只规划/只建任务进入 unfinished/，只有当前用户明确开始或继续才建立 current leaf。leaf 是模型维护的工作投影，不是授权票或范围最高权威。Stop 只有在短期 authorization.json 与当前 project/session/turn/leaf/stage 精确绑定且本轮有新产品进展时才能内部续跑；内部续跑不是新用户授权，不得扩大范围。发现仍服务同一目标的必要相邻路径，模型更新 leaf 并继续；普通 leaf 范围外只报告，明确“不许动”的路径和 push 按既有停点处理。文件位置是生命周期事实；退场只原子移动一个文件。报告必须分 Harness、产品、证据、载体，不把 working copy、离线 fixture 或协议推断冒充发布或真实运行。
<!-- HARNESS-LITE:END -->
