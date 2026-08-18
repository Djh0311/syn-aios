# M1 普通产品项目登记增补合同 v1

日期：2026-08-18
状态：M5R09 当前增补；不修改、替换或重写任何 M1–M4 冻结合同正文与旧 hash。

## 1. 目的与边界

本增补只关闭普通产品已有 M1 canonical ProjectId 消费面“有消费者、无生产者”的缺口。它不把路径变成身份，不从 M5 helper、renderer cache、cwd、线程或真实资料自动导入项目，也不建立第二个 Project owner。

普通产品固定 product index 仍只提供“用户当前能看见并明确选择哪个项目”的来源记录；M1 project index 仍是 canonical `ProjectId` 与 exact alias 的唯一登记/解析 authority。

## 2. 首次启动与 fail-closed

- ordinary identity source 缺失且 M1 registry 尚未 established 时，普通 Tauri AppState 可以进入 `UNENROLLED` 可恢复状态，以便显示现有 project index 并接受用户的显式登记动作。
- `UNENROLLED` 不创建空 registry、marker、identity source 或 canonical `ProjectId`；所有需要 M1 identity 的治理写入与解析继续返回固定 unavailable/unknown 错误，零业务写。
- identity source 损坏、unsupported、symlink/非普通文件、不可读，或 established registry 丢失/损坏时继续启动 fail-closed；不得把这些状态降级为 `UNENROLLED`。

## 3. 显式登记命令

- 普通 UI 只在用户点击具名“登记项目身份”动作时调用 enrollment command；启动、列表加载、preview 或 memory/mature consumer 不得自动调用。
- command 输入只含用户选择的 `project_root`。服务端必须从已安装的固定 product index 重新读取并得到恰好一条 exact root 记录；零条或多条都在任何 M1/source 写入前拒绝。
- 服务端把该 exact root 写为 M1 exact alias，并把 `product-index:<exact-root>` 记录为显式 `source_ref`；canonical `ProjectId` 只由 M1 authority 以随机 opaque UUID 分配，不由 root、name、hash 或调用方提供。
- 前端不得传 canonical id、registry path、source file path、source revision 或 entry id。

## 4. 可重放与幂等顺序

- enrollment 先以原子 replace + file/dir sync 持久化/更新 ordinary identity source，再 replay/materialize M1 registry；若后半段失败，下一次启动或同一显式命令可从 source 恢复，不能声称登记完成。
- identity source 既有相同 exact alias + 相同 `source_ref` 时为幂等重放，不新增 entry；相同 alias 指向不同 source 或同一 source 指向不同 alias 时 fail-closed。
- source revision 只在 source 内容新增时单调递增；重复相同登记不改变 source revision。registry 中既有同 alias 时返回同一 canonical `ProjectId`；重复调用和重启都不得 mint 第二个 id。
- 并发相同登记最终只留下一个 source entry、一个 registry project 与一个 established marker；任何冲突不得部分追加。

## 5. 证据边界

本叶只用临时 app-data 与合成固定 product index 验证首次登记、重复登记、重启恢复、未知/重复 source 拒绝、损坏与 symlink fail-closed。它不读取或写入真实用户项目，不证明真实窗口、真实账号/provider/凭据、外部业务写、部署或发布。
