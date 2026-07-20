# 决策：shape gate 新增 hardcoded_hex_on_ui 机械规则 + hex 白名单 v1

日期：2026-07-20
任务包：`tasks/2026-07-20-g1-token-truth-restoration-package-v1.md`（G1 token 归真 §六）
法源：`prototypes/productized-desktop-shell/DESIGN.md` §一（零换皮拍板值）

## 拍板

1. shape gate 新增 `hardcoded_hex_on_ui`：**UI 样式禁新增裸 hex，一律走正典 token（styles.css 单 :root）**；
   裸 hex、`var()` 回退位 hex（错误回退实证）、`%23` 转义形均 error 级，新增零容忍。
2. 规则本体拆 `scripts/harness/lib/hardcoded-hex-rule.js`（gate 本体只 +3 行：require/挂载/打印；
   492→495 行，不破 500 软限）。
3. 豁免：正典定义行（`^\s*--[\w-]+\s*:`）、注释行、白名单（`hex值|path` 粒度）。
4. 白名单预登记上限 86 条（勘察 §6：styles.css 49 + sidePanel 25 + ActiveWorkbenchView 3 +
   canvasNodeData 6 + ProjectWorkflowCanvasView 1 + WorkflowCommandConsoleView 2）；
   **G1 施工后实际 42 条，治平一批核销一批，只减不增**；不得为过关塞新违规。

## 白名单 42 条明细（与 lib/hardcoded-hex-rule.js 一一对应）

| 组 | 条目 | 理由 |
|---|---|---|
| styles.css 26 | boot 诊断屏 6 值（`#1a1c1a`/`#f7f1e3`/`#f1ead9`/`#e8dfcd`/`#5c3a1f`/`#2a2419`） | 启动失败屏一次性面（c 类·随③清单另包治平） |
| | SVG data-URI 转义 2 值（`%231c1f24`/`%23a14242`，9 处） | url() 内转义形无法写 var（c 类） |
| | 无等值 token 的 live 零散值 18（`#d8d3c5`/`#faf8f3`/`#ccc`/`#8a8275`/`#ddd`/`#c9bfa6`/`#faf7f0`/`#b14422`/`#c8a05a`/`#8a7f6a`/`#f7f1e6`/`#fffdf8`/`#18211f`/`#edf7f1`/`#cfc8b6`/`#3f5235`/`#7a2e2e`/`#f7e8e8`） | b 类兜底：正典表为合同不扩列，就近 token 会调色（禁），值取 live 登记 |
| sidePanel 9 | `#2e7d4f`/`#4caf72`/`#666`/`#6b6b6b`/`#a86a00`/`#b0b0b0`/`#b23b3b`/`#d9a441`/`#e05656` | 状态色零散值·同上 |
| canvasNodeData.ts 6 | `#c8602b`/`#5a6f4a`/`#3a6a77`/`#8a7f6a`/`#b9b3a6`/`#a14242` | 节点调色板**数据**非样式（c 类） |
| ProjectWorkflowCanvasView.tsx 1 | `#9aa0a6` | SVG 依赖边 stroke·无等值 token（§八「替换」与 §六.3 白名单冲突，取零调色侧并披露） |

ActiveWorkbenchView / WorkflowCommandConsoleView 预登记 5 条全部治平核销（0 残留）。

## 纪律

- 新增裸 hex = error，check 模式 fail；白名单只减不增（后续包治平一条核销一条）。
- selftest：`scripts/harness/workbench-shape-gate.hardcoded-hex.selftest.js`（13 断言：
  裸 hex/回退位/转义/TSX 内联→error；定义行/注释→不误伤；白名单→deferred；干净树→0）。
- 施工后实测：violations 0、deferred 75 处出现（42 值 × 各文件），shape 13/5/5 零净增。
