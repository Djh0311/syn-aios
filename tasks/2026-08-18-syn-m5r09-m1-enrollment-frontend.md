# Grok 窄包：M5R09 M1 普通产品前端登记入口

后端生产 command 与 registry 已提交为 `599f555`，直接测试已提交为 `387e10e`。本包只补普通产品现有项目页内的最小显式入口，不改页面整体布局，不自动登记。

## 唯一允许修改

- `prototypes/productized-desktop-shell/src/lib/tauri.ts`
- `prototypes/productized-desktop-shell/src/App.tsx`
- `prototypes/productized-desktop-shell/src/components/ActiveWorkbenchView.tsx`

三文件当前均为干净 tracked 文件。不要修改任何其他文件，不要 git 操作，不要整文件格式化。

## 实现约束

1. 在 `tauri.ts` 新增严格 DTO：request 只有 `project_root`；response 精确包含后端返回的 `project_id`、`exact_alias`、`source_ref`、`source_revision`、`registry_revision`、`status`（`created | already_enrolled`）。新增 invoke wrapper，command 名精确为 `enroll_m1_project_identity`，不得传 project id、registry/source path、revision 或 entry id。
2. `App.tsx` 只把该 wrapper 作为回调传给 `renderActiveWorkbenchView`。浏览器预览模式不得调用 Tauri command；成功/失败的局部状态由入口组件展示，不在启动、reload、project list load、preview 或 effect 中自动调用。
3. `ActiveWorkbenchView.tsx` 在既有 `view === "projects"` 分支内增加一个小型独立 React 组件，位于既有项目内容旁，不能修改 `ProjectsView.tsx` 或重画布局。组件用 `snapshot.projects` 提供明确的项目选择，只有用户点击具名“登记项目身份”按钮才调用回调；busy 时防重复提交。没有项目、浏览器预览/无回调时按钮禁用并说明原因。
4. 状态展示至少区分：尚未登记动作、正在登记、首次创建成功、已登记幂等重放、失败；成功展示 canonical `project_id`、source/registry revision 和 exact alias，但不得由前端推导或缓存身份。失败显示安全错误文本。不得声称真实项目、真实窗口或发布已验证。
5. 给入口根节点与按钮加稳定、语义明确的 `data-*` 标记，便于默认 bundle 静态门验证 command 名、按钮文案与无自动调用边界。

## 验证

- `npm run typecheck`
- `npm run build`
- `rg -n "enroll_m1_project_identity|登记项目身份|data-m1" src/lib/tauri.ts src/App.tsx src/components/ActiveWorkbenchView.tsx dist/assets/*.js`
- `cargo test --lib --offline m5_ -- --test-threads=1`（每个产品任务包保留；若本包未跑必须明确报告，候选流程会重跑）
- 三个允许文件 `git diff --check`

完成实现与 typecheck/build 后立即退出；不要读取 harness、历史或无关文件，不接真实资料、账号、provider、凭据或外部业务。
