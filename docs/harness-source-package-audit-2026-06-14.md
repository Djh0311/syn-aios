# Harness 上游源码包审计（agent-safety-harness）

> 资料状态（2026-08-09）：旧开发护栏上游包的日期快照与历史审计，不是当前产品、当前工具清单或施工入口。

- 审计日期：2026-06-14
- 目标：`/Users/yoyi/harness engineering/harness-engineering`（即 product-line `.harness/manifest.json` 里记的 `sourceRoot`——product-line 就是从这里 install 的）
- 性质：纯只读。未改动/未删除/未运行任何脚本（含 `npm test` / `self-test.js`），未提交。本文件写入 product-line 仓库 docs/ 作为 [上一份安装副本审计](harness-script-audit-2026-06-14.md) 的姊妹篇。
- 与上一份的关系：**脚本是同一批文件、同一批 hash**（manifest 里 sourceSha256==installedSha256）。所以「每个脚本干什么」的功能桶位与上一份一致，本份**不重复逐脚本功能表**；本份回答的是上一份回答不了的问题——**这套库在它自己老家是什么健康状态、为什么装到下游会变成一堆惰性文件**。

---

## 拍板摘要（先看这段）

- **审计对象**：上游 npm 包 `agent-safety-harness@0.1.0`（MIT，`private:false` 但从未发布）。
- **一句话体检结论**：这是一个**设计精良、但只提交过一次、整套现代架构全在未提交工作区、自带测试从没被证明跑绿、从未发布、~5/21 冻在 agentmemory 阶段**的早期原型。
- **最刺眼的事**：一套专门**强制 agent 做 commit 纪律 / 完成前验证 / 证据留痕**的 harness，**它自己**是：1 个 commit、53 个未提交改动、0 CI、0 测试证据、0 发布的脏仓库。补鞋匠的孩子没鞋穿。
- **它如何解释下游**：installer **不拷 `package.json`**。源码包的真实接线（`npm test/lint/build/doctor` + self-test 夹具）全在 package.json 里，install 时被丢掉；下游只拿到「被剥掉运行架的脚本」+ 一份 advisory 检查清单。这就是上一份审计里「下游 77/79 脚本零落地」的上游成因。
- **要你拍的**：这套上游是「继续投入做成真包」还是「明文定格为内部原型」？删改由你拍，本审计只给证据。

## 一句话判据

> 判一个「源码包」健不健康，不看它 README 写得多漂亮，看三件实物：**① 它最新的架构进了 commit 没？②它自带的 test 有没有任何一次被证明跑绿（CI/提交产物）？③ 它 bin/package.json 声称的入口，install 之后下游还在不在？** 这三条本包答案全是「否」。

---

## 一、核心结论

### 1. 整个仓库只有 1 个 commit，现代架构全部未提交

| 实物 | 数值 |
| --- | --- |
| 总 commit 数 | **1**（`Initial release`，2026-05-13） |
| 「Initial release」里 committed 的顶层脚本 | **52** 个，且 **没有 `lib/`** |
| 当前工作区脚本 | **64** 顶层 / **77** 含 `lib/` |
| 未提交改动路径 | **53**（18 Modified + 35 Untracked） |
| 与发布版有差异的脚本文件 | **35 / 64** |
| 仅 Modified 的 18 个 tracked 文件 diffstat | **+1264 / −29 行** |

**整个 `lib/` 共享层（13 个模块）100% 是 untracked**——从没进过任何 commit。连同**整套 agentmemory 子系统**（`memory-agentmemory-*`、`memory-candidate-*`、`memory-review`、`memory-stale-check`、`memory-maintenance`）、`eval-runner`、`security-scan`、`context-pack`、`evidence-compact/retention`、`task-package-*` 等，全是 untracked。

> 含义：**装进 product-line 的那套「现代 77 文件 harness」，在上游从未以任何 commit 形态存在过。** product-line 是从一个**脏工作区**快照出来的；manifest 里的 sourceSha256 是未提交文件的 hash。这套库没有一个可复现的发布点。

### 2. 没有 CI，从未发布

- 源码包内**无 `.github/workflows`**（只有它要分发给别人的 `templates/ci/*` 和一个 fixture 项目自带的 CI）。`policy.ci` 在它给的示例 config 里也是 `required:false`。
- `package.json`：`private:false`、有 `bin`（`agent-safety-harness`→`harness.js`）、有 `files` 白名单——一副「准备发 npm」的样子，但 **ROADMAP 把「Explore npm distribution」放在 `Later`**，即从未发布。

### 3. 自带测试很扎实，却没有任何「跑绿」证据

- `npm test` / `check` / `self-test` 全指向 **`self-test.js`**（73KB，**325 个 check/assert 点**），它在临时目录里造假项目、对着 **15 个 fixture**（node/python/go/existing-ci/malformed-manifest/security-redaction/stale-evidence/task-package-schema…）跑大半个脚本套件。**这是认真做的集成测试，不是摆设。**
- 但：单 commit、无 CI、`self-test.js` 自己还带未提交修改（`[M]`）——**没有任何一次它被证明通过的实物**（无 CI 记录、无提交产物）。能力造好了，验证从没发生。下游也从没跑过（上一份审计：0 命中）。
- 同理 `npm build`→`fixture-check.js`、`typecheck`→`config-schema.js`、`lint`→`rules-lint.js`、`doctor`→`harness.js doctor --strict`：**接进了 npm 生命周期，但没有被任何自动化跑过的证据。**

### 4. 开发在 agentmemory 阶段戛然而止——正是下游被禁用的子系统

`plans/` 的开发弧线：phase-1..5（5/12–13）→ v2（5/16）→ v2.2 eval/audit/context（5/16）→ **`2026-05-21-codex-harness-agentmemory-integration-plan.md`（27KB，最大、最后一份）**。之后停摆。

- ROADMAP 的 Near-Term 仍写着「**Continue** governed agentmemory integration: live backend test coverage…」「Add end-to-end memory eval fixtures **once** the test environment can safely bind localhost」——即 agentmemory 是**进行中、未完成**。
- 而下游 product-line 的 config 把 `memoryIntegration.enabled=false`。**上游最后的大动作，恰好是下游关掉的那一块。** 这套库的「最新一层」从设计到落地都没闭环。

### 5. installer 丢掉运行架——连接上一份审计的关键齿轮

`install-harness.js` 的 install 集合（实物读自源码）：`AGENTS.md`、`codex-multi-agent-safe-collaboration.md`、`harness.config.example.json`、`skills/`、`scripts/harness/`、`templates/`、`templates/docs→docs/`。**唯独不含 `package.json`。**

- 所以源码包里真正让脚本「跑起来」的接线（`npm test/lint/build/typecheck/doctor`、`bin`、self-test 夹具）**全部留在上游、装不到下游**。
- 下游拿到的是「被剥掉运行架的脚本文件」+ 一份 `harness.config.example.json`（变成 advisory 检查清单）。再叠加下游 hooks=off、CI=none、`harness.js` 没人调、流程文档不指向它——**惰性是结构性的，不是下游偷懒**。

---

## 二、源码侧四桶视角（简版；逐脚本功能桶同上一份）

把「用/没用」换成源码包自己的口径——「是否接进本包生命周期（npm/router/self-test）且 committed/可验证」：

- **承重（本包真正的运行面）**：`harness.js`（router + bin）、`self-test.js`（test）、`fixture-check.js`（build）、`config-schema.js`（typecheck）、`rules-lint.js`（lint）、`config-init.js` 及被 self-test 经 15 夹具拉起的大半套件。
  - 与下游最大的不同：**在源码包里，绝大多数脚本是「能被 self-test 触达的」**——不像下游那样彻底孤立。但「能被触达」≠「被验证」，因为 self-test 从没被证明跑绿。
- **本阶段休眠（设计中/未完成）**：agentmemory 整套（uncommitted + roadmap「continue」）、browser/UI 证据流（roadmap「Next」）、embedding skill retrieval（roadmap「Later」）、memory e2e eval 夹具（「once localhost…」）。
- **从没接上 / 真死**：在源码包层面**不成立逐脚本判定**——self-test 触达了大多数脚本，真正的缺陷不在「某个脚本死了」，而在**仓库级**：未提交、未验证、未发布。唯一与上一份一致的具体重复仍是 `capability-map.js` ↔ `capability-scan.js`。

> 一句话：下游审计的失败模式是「脚本没接到外部世界」；上游审计的失败模式是「整个包没接到自己的 commit/CI/发布」。同一批脚本，两种「没接上」。

---

## 三、为什么会这样（根因）

1. **原型速度优先，工程纪律靠后**：phase-1→agentmemory 十天冲刺，产出量大（77 文件、325 断言、15 夹具、8 份计划），但全程只 commit 过一次种子版，把「提交/CI/发布」这些它自己布道的纪律留到了「以后」——而「以后」没来。
2. **设计先行、一次性铺全**（与下游同根）：先把通用面铺满，再指望被采用；结果通用面在上游没验证、在下游没接线。
3. **install 模型把「成品脚本」与「运行架」切开**：只分发脚本+规则+模板，不分发 package.json 的跑动方式，默认下游用 hooks/CI 自己接——而下游把这些都设成 off。切口设计本身就预设了「下游大概率不接」。
4. **冻结点尴尬**：停在 agentmemory（最重、最未完成、最依赖外部 backend）那一层，导致「最新一层」既没收口也没退役。

---

## 四、Meta：可发现性与可复现性

- 上一份的 meta 是「下游缺一份能用的脚本索引」。本份的 meta 更靠前一层：**上游缺一个可复现的发布点**。没有第二个 commit、没有 tag、没有 CI 绿，意味着任何人（包括维护者）都无法回答「哪个版本是好的、self-test 现在还过不过」。下游 manifest 锚定的是一串未提交 hash——上游一旦工作区再变，这个锚就成了孤儿。
- 想救这套库，顺序建议（均待你拍）：
  1. **先固化一个可复现点**：在上游把当前工作区 commit（哪怕叫 v0.2-snapshot）+ 实际跑一次 `npm test` 留下绿色证据。没有这一步，下面都是浮沙。
  2. **决定 agentmemory 的命运**：收口完成，或明文降级为「实验/默认关闭」并从 Near-Term 移走。
  3. **修 install 切口**：要么把最小运行架（一个能 `npm run doctor/self-test` 的 package.json）也装到下游，要么在下游 AGENTS.md 里指一行「这些脚本怎么调」（即上一份建议的「能用的脚本索引」）。
  4. **删，最后再说**：和下游同理——在「固化+验证」之前，源码包里没有可靠的「真死」判定；唯一具体的删/并候选仍是 `capability-map`/`capability-scan` 二选一。

---

## 五、我做了什么 / 没做什么

- **做了**：读 package.json/README/ROADMAP/plans、git 史（1 commit）、`git status`/`diff --stat`（量化 53 改动 / +1264 行 / 35 脚本偏离发布版 / 整个 lib 未跟踪）、self-test 覆盖与夹具清单、install-harness 的分发集合。
- **没做（边界诚实）**：**未运行 `npm test`/`self-test.js`/任何脚本**（只读约束沿用上一任务），因此「self-test 今天是否跑绿」**未知**——这恰恰是本审计认定「无验证证据」的那个洞，我没有替它补上，只指出它空着。未对 `capability-map/scan` 做完整 diff。未读未提交改动的逐行内容（只量化规模）。
- **若你要**：我可以（在你放行运行的前提下）跑一次源码包的 `npm test` 看它今天到底过不过，把「未知」变成「实测」。
