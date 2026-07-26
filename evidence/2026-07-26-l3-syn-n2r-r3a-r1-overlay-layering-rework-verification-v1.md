# L3 Syn N2R-R3A-R1 Overlay 层叠关系窄返工验证 evidence v1

- 日期：2026-07-26
- 执行结果：**`PASS_N2R_R3A_R1_OVERLAY_LAYERING / NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED`**
- 指导复核：**`ACCEPTED_N2R_R3A_R1_OVERLAY_LAYERING / ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED`**
- 证据级别：真实 React + 真实生产 CSS + pure-synthetic fixture 的 fresh localhost browser context；**不是**真实 Syn/Tauri App、真实 vault/store、I5、Gate 0、完整 R0 或发布验收。
- 执行线只回交改动与证据，不自行验收，不继续 R3B。

## 1. 开工 preflight、权限与 provenance

- 工作目录：`/Users/yoyi/workspace/product-line`。
- HEAD：`e9ad7f3a204a1ebb11ce26c1e8c05b19c04c0991`，与任务包冻结值一致。
- `git diff --cached --name-only`：无输出，staged 为空。
- `lsof -n -P -iTCP:5173 -sTCP:LISTEN`：无输出，开工时 5173 无 listener。
- 写所有权：`CURRENT.md` 与本任务包均把 `src/styles.css` 的本窄写面独占给 R3A-R1。开工时 Codex task registry 中，同一 `product-line` 项目只有当前执行任务 `019f9aee-fed7-72f0-9c10-db730e43edf5` 为 active；指导任务 `019f9078-3271-71a1-a49f-169085f1c38e` 与旧 R1 任务 `019f99b2-aa25-77d0-93b2-766088679137` 均为 idle，未见并行 writer。
- 未启动 Syn、Tauri、Obsidian、Codex CLI/MCP、真实 App 或对话三句重验；未读取真实 store/vault。

全部冻结 SHA-256 在任何写入前逐一匹配：

| 冻结文件 | 实测 SHA-256 | 结果 |
| --- | --- | --- |
| `prototypes/productized-desktop-shell/src/styles.css` | `e040da8b5e4fb18cc0f8b5df1e5a78a70094c239a370704267659fc192a06134` | MATCH |
| `prototypes/productized-desktop-shell/src/views/knowledge/NativeKnowledgeWorkspace.tsx` | `f4f1c5aed802e66ae3418460f3b5ff1a9a2fe33fddd773ab748e9c2f63025fe5` | MATCH |
| `prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeWorkbenchShell.tsx` | `2ab66a5b00941e3cf0a083e3ed3d24b309f8b7fe5037dd41d8743cf9d2d41c21` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-shell.test.tsx` | `30727a05000561f0d9812c385a8a29fb4d199a35abbd1ba2d33ecbe552aebad2` | MATCH |
| `prototypes/productized-desktop-shell/tests/native-knowledge-workspace.test.tsx` | `fdcdfe54fb8f6dd022d4b018635d3bca1f2cd75874f2a835859dacfaa406408e` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.tsx` | `1e590ac9d040d4e9486b8afe8171526f0c9dbecb38a419bfb0d1d9ad07523b30` | MATCH |
| `prototypes/productized-desktop-shell/tests/knowledge-workbench-visual-fixture.html` | `8a58ac0d02d9c33a9ca151f4ec1e653ccd3c6d79096581efd339a62c1fcec23c` | MATCH |
| `prototypes/productized-desktop-shell/scripts/run-offline-interaction-test.mjs` | `798daf13722875befa22eb233c476b957d241005a3b08267ed30eb54e0363082` | MATCH |
| 上游 R3A task | `a30b701d0b2121aada767638810972884dadbb82e0f956b1dd7ee10ae10ce517` | MATCH |
| 上游 R3A evidence | `dd8e67c953004786fc1eac08946e7364fe3b9b5d4322cc71febc8bd21ba003a0` | MATCH |
| R0 设计参考 | `141ff343312fff91346e6e170f1aea2d07cda5fa2abdb3fde8b840b16f9ed4d9` | MATCH |

白名单 provenance：

| 写面 | 开工事实 | 本包处理 |
| --- | --- | --- |
| `src/styles.css` | 上游 R1/R3A 的既有 tracked dirty 基线；冻结 hash 匹配 | 只保留一个最终 selector/value 变化 |
| `evidence/raw/.../r1-overlay-layering-*`、`02-r1-*`、`03-r1-*` | 上游 raw 目录已存在且为 untracked evidence | 只新增本包前缀/文件名，不覆盖上游 02/03 |
| 目标 evidence | 开工时不存在 | 新建本文件 |
| 本任务包 | 指导派发的 untracked task 文件 | 只回写实际状态、写入和结果 |
| `docs/harness-catch-log.md` | 既有 tracked dirty 账本 | 只追加 §9 的一个新 catch |

## 2. Red → Green 浏览器合同

新合同：

- [r1-overlay-layering-browser-contract.mjs](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-browser-contract.mjs)
- red 原始报告：[r1-overlay-layering-red.json](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-red.json)
- green 原始报告：[r1-overlay-layering-green.json](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-green.json)

### 2.1 Red

当前未修改 CSS 上，合同为 **33 项断言 / 4 项失败 / 29 项通过**。四个失败名称精确为：

1. `quick-open: computed backdrop alpha is strictly between zero and one`
2. `quick-open: screenshot outside the overlay retains a subdued workspace`
3. `command: computed backdrop alpha is strictly between zero and one`
4. `command: screenshot outside the overlay retains a subdued workspace`

浏览器 computed style 与实图像素分析：

| 场景 | computed backdrop | alpha | 底层基准 edge energy | overlay 外区 edge energy | 保留比 | 色桶 |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Quick Open | `rgb(240, 237, 230)` | `1` | `3.4142598909736437` | `0` | `0` | `1` |
| Command | `rgb(240, 237, 230)` | `1` | `3.8220364281763` | `0` | `0` | `1` |

red 实图：

- [r1-overlay-layering-red-quick-open.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-red-quick-open.png)
- [r1-overlay-layering-red-command.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-red-command.png)

两图的 overlay 外区域都是单一不透明纸面，底层活动栏、侧栏和中央工作区完全不可辨认。red 中行为、焦点、ARIA、overflow、read allowlist 与五类零值仍通过，证明失败只落在本包视觉层。

### 2.2 Green

最终同一合同为 **33 / 33 PASS，0 失败，2 个 fresh context**。

| 场景 | computed backdrop | 实际 alpha | 底层基准 edge energy | overlay 外区 edge energy | 保留比 | 色桶 | 前景 alpha |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Quick Open `1180×760` | `color(srgb 0.941176 0.929412 0.901961 / 0.72)` | `0.7215686274509804` | `3.4142598909736437` | `0.9695372160243371` | `0.28396702271772706` | `18` | `1` |
| Command `900×760` | 同上 | `0.7215686274509804` | `3.8220364281763` | `1.064834488905267` | `0.2786039612430793` | `16` | `1` |

- backdrop alpha 严格满足 `0 < alpha < 1`。
- 外区 edge energy 明显低于原工作区但不为零，底层轮廓/内容得以保留且降噪。
- 前景 `background: rgb(255, 255, 255)`、alpha `1`、茶色顶边 `3px`、`box-shadow: none`。
- backdrop 的 `background-image / backdrop-filter / filter` 分别为 `none / none / none`；无 gradient、blur 或玻璃效果。

## 3. 精确 CSS hunk 与最小性

最终产品改动只有 `.syn-knowledge-overlay-backdrop` 的一个 value：

```diff
 .syn-knowledge-overlay-backdrop {
-  background: var(--panel-soft);
+  background: color-mix(in srgb, var(--panel-soft) 72%, transparent);
 }
```

- 仍由既有 `--panel-soft` token 派生，没有新增裸色或新 token。
- 没有改 `.syn-knowledge-overlay`；既有不透明 raised surface、hairline/茶色边界已足够把前景抬起。
- 没有改尺寸、位置、padding、字号、列表、选中态、动画、React/TS、fixture 或 runner。
- `styles.css` 最终 SHA-256：`1e846799b94724a26f3175a24dbf6c5751527113d3b22c57add0c520a6762484`。

## 4. 两张新图与 R0 05/06 对照

| 新图 | R0 参考 | 本包判定 | 可见事实与保留 GAP |
| --- | --- | --- | --- |
| [02-r1-1180-quick-open-results.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/02-r1-1180-quick-open-results.png) | R0 06 Quick switcher | **PASS（仅 R3A-R1 overlay 层叠）** | 活动栏、左侧目录、中央工作区与右侧上下文仍可辨认但明显退后；前景输入、16 条真实结果与当前项保持唯一焦点。R0 全界面的活动栏图标、标签组/分栏和右栏细节仍是既有范围外 GAP，不在本包冒充关闭。 |
| [03-r1-900-command-filter-results.png](raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/03-r1-900-command-filter-results.png) | R0 05 Command palette | **PASS（仅 R3A-R1 overlay 层叠）** | 底层三段工作区在 veil 后仍可定位；前景命令面板不透明、边界清楚，一个真实当前命令与提示不受背景竞争。完整 R0 高保真仍未验收。 |

逐图人工判断与浏览器像素合同一致：这是 `workspace base → translucent veil → opaque raised overlay`，不是第二张空白页面；也没有 glassmorphism、重 blur、强阴影、渐变或戏剧化动画。

## 5. 键盘、焦点、ARIA 与 overflow 未回归

- Quick Open：`⌘O` 后焦点为 `#native-knowledge-quick-open`；输入“合成”得到 `16` 项、唯一 `aria-selected=true` 当前项；ArrowDown 后 `aria-activedescendant=native-knowledge-quick-open-option-1`；提示含 `↑↓ / Enter / Esc`；Escape 关闭并回到 `aria-label=搜索` 的触发按钮。
- Command：`⌘P` 后输入“目录”，结果为唯一“新建目录”；焦点保持 `#native-knowledge-command-filter`；提示含 `↑↓ / Enter / Esc`；Enter 只进入既有受限相对路径表单，路径 input `1` 个、创建按钮 disabled，未创建任何内容；Escape 回到 `aria-label=Syn 命令` 的触发按钮。
- Quick Open/Command 的 Enter、Arrow 边界、Escape、`⌘O / ⌘P / ⌘⇧F`、combobox/listbox/option 关联、成功打开后的中央 tab 焦点继续由两个聚焦合同与完整 runner 覆盖；本包未改相应 TS/测试，冻结 SHA 保持不变。
- `1180×760` Quick Open：document/body `1180/1180`，shell `1178/1178`。
- `900×760` Command：document/body `900/900`，shell `898/898`。
- 两个场景的 document、body、shell 均 `scrollWidth <= clientWidth`，无新增横向 overflow。
- 本包没有新增动画；既有 `prefers-reduced-motion` 合同与完整 runner 保持绿色。

## 6. Fresh context、mock allowlist 与五类零值

每个 context 在 mount 前均为 `localStorageEmptyBeforeMount=true`。精确 read mock allowlist 为：

```text
knowledge_workspace_graph
knowledge_workspace_read_canvas
knowledge_workspace_read_markdown
knowledge_workspace_search
knowledge_workspace_snapshot
```

| context | 实际 read 调用 | write | unknown | external request | console error | page error |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| Quick Open | `snapshot=1, search=1` | 0 | 0 | 0 | 0 | 0 |
| Command | `snapshot=1` | 0 | 0 | 0 | 0 | 0 |

## 7. 必跑门禁

| 验证 | 实际结果 |
| --- | --- |
| 聚焦 `knowledge-workbench-shell` 合同 | PASS：`knowledge workbench shell static convergence contract passed`。 |
| 聚焦 `native-knowledge-workspace` 合同 | PASS：`native knowledge workspace N0 optional compatibility and N1/N2/N3/N4/N5 fixed client contract tests passed`。 |
| `npm run typecheck` | PASS，exit `0`。 |
| `npm run test:offline-interaction` | PASS，exit `0`；runner 当前登记 **37** 个 entry。首个测试打印的 `offline interaction tests passed: 15` 是该测试自身断言口径，不是 runner 数量。 |
| green browser evidence | PASS：`33 / 33`，2 个 fresh context，两张新图。 |
| shape baseline | PASS：`Errors 17 / Warnings 5 / Info 5`。 |
| shape check | 预期非零 FAIL：仍为 `17 / 5 / 5`；与 baseline 的类别、finding、行数完全相同，零新增类别/finding。 |
| hardcoded-hex selftest | PASS：`13 / 13`。 |
| machine-face selftest | PASS：`18 / 18`。 |
| `git diff --check` | PASS，无输出。 |
| `git diff --cached --name-only` | PASS，无输出，staged 为空。 |
| 进程收尾 | 只向本包启动的 Vite session 发送 `Ctrl-C`；两次 headless Chrome 均由合同自身关闭；最终 5173 无 listener。 |

## 8. 实际写入文件

1. `prototypes/productized-desktop-shell/src/styles.css`：只改目标 backdrop 的一个 background value。
2. `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-browser-contract.mjs`。
3. `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-red.json`。
4. `evidence/raw/2026-07-26-l3-syn-n2r-r3a-search-overlay/r1-overlay-layering-green.json`。
5. 两张 red 取证图：`r1-overlay-layering-red-quick-open.png`、`r1-overlay-layering-red-command.png`。
6. 两张新 green 图：`02-r1-1180-quick-open-results.png`、`03-r1-900-command-filter-results.png`。
7. 本 evidence。
8. 本任务包：只回写实际状态、写入与结果。
9. `docs/harness-catch-log.md`：只追加 §9 的一个新 catch。

未修改 React/TS、测试、fixture、runner、Rust/Cargo、Graph/Canvas/Maintenance、活动栏、右栏、依赖、CURRENT、AUTHORITY、计划、决策、真实 store/vault 或 `.codex`。未 stage、commit、push、reset、clean、stash 或 checkout。

本包 raw 关键产物 SHA-256：

| 产物 | SHA-256 |
| --- | --- |
| 浏览器合同 | `b1be6f022b8d315c902d871612e27e6d4d19830557961ef24bcfadaf10a55cc2` |
| red JSON | `9ef0502250252181c08bc21ee120b9ccc6d7766eb47445dd4a08fa96b93c0293` |
| green JSON | `fa5683ee86f80a42fcd91770c916afe48e7e393687035c12c2c18b40481902d1` |
| red Quick Open | `984aab712297045d387e263324cc2066e3625cd0cfb27d607fe538707e050f49` |
| red Command | `f1ecf1bfd282941ced7d450d70d8cd654e582a0a73e9bb4b95e119cbf9aa8b62` |
| green Quick Open | `9e0314747030830fa2a5584cbe65a1d95603d24b0af5c3c8c1d92e6258153ab3` |
| green Command | `5c909ffbca3892ee07db54b51304bb7338c78b3df9e99a013920c07b492cb5da` |

## 9. 新 catch

发现并记录 **1 个新 catch**：

- 首个最窄补丁只带同值声明上下文，误命中较早的 `.obsidian-integration-details > div`，目标 backdrop 未改变。首次 green 合同继续精确失败 `4 / 33`（alpha `1`、外区 edge ratio `0`），因此没有形成假绿。误命中行原样恢复后，补丁改为绑定完整 `.syn-knowledge-overlay-backdrop` selector 上下文；最终 `33 / 33`。
- 已追加 `docs/harness-catch-log.md`。预防口径：长 CSS 的单声明补丁必须绑定完整 selector，并由浏览器 computed style + 实图合同确认真实命中。

## 10. 遗留与停止边界

- 本包 overlay 层叠候选在执行线回交时保持 `NEEDS_GUIDANCE_REVIEW / NOT_ACCEPTED`；指导线随后已完成 §11 独立复核并接受本包 synthetic 范围。
- shape `17 / 5 / 5` 是上游既有全仓债务，本包零净增，不在本包修。
- 未开始、未授权、也不声称完成：R3B、活动栏/标签组/分栏/Canvas/Graph/右栏后续视觉收敛、完整 R0 高保真、真实 Syn/Tauri App、真实 store/vault、Home-only discovery、Gate 0、十二项或发布验收。
- 没有出现需要修改 React/TS/fixture/runner、放宽安全边界、启动真实 App 或写真实数据的停止条件；不自行续跑后续 R3。

## 11. 指导线独立复核

- 指导线逐张对照 R0 06/05：两图的底层工作区仍可定位但被充分降噪，前景面板保持唯一焦点；本包 overlay 层叠判定接受。
- 目标一行经只读替换回旧值后，`styles.css` hash 精确恢复派发冻结值；其余冻结文件 hash 全匹配。raw 合同与七项关键产物 hash 也和本 evidence 登记一致。
- 独立复跑 `npm run typecheck` 与正式 37-entry `npm run test:offline-interaction` 均 exit `0`；shape 为既有 `17 / 5 / 5`，两项 selftest 为 `13 / 13`、`18 / 18`。
- green 合同确实读取 `.syn-knowledge-overlay-backdrop` 的 computed style，并同时检查 alpha、截图外区 edge retention、前景 surface、无 gradient/blur/filter、焦点、overflow 和零错误审计，不是源码字面假绿。
- 执行线 catch 成立；指导最终复核零新 catch。精确结论为 **`ACCEPTED_N2R_R3A_R1_OVERLAY_LAYERING / ACCEPTED_N2R_R3A_SEARCH_OVERLAY / NOT_REAL_APP_ACCEPTED`**，不外推 R3B、完整 R0 或真实 App。
