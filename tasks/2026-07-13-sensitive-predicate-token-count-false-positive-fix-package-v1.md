# 任务包:敏感谓词 token 计数键误报修复(重档·外科手术式)v1

日期:2026-07-13 · 档位:**重档**(高危#3 类:改迁移面安全谓词本身)· 基线 commit `cdd8f57`。
上承:M3 演练首战果(catch-log 07-13)——live 主 store 被 `contains_sensitive_value` 判 `rejected_sensitive`,根因 `"token"` 子串误命中良性计数键 `estimated_tokens`/`max_estimated_tokens`;M5 真翻闸硬前置。

> **派发闸(本包最重要的一行)**:本包文件存在 ≠ 已授权。**执行线动第一行代码前,必须用户对本包明确说"授权"那一下**(AGENTS §二 重档:用户在场+单独一步+明确授权)。
> **✅ 授权已给**:用户 2026-07-13 对本包明确「可以」(总指导会话·出席点①),派发闸已开;本行即授权留痕。总指导核收也只核一件事:**只收窄两个数值计数键,其余拦截逐条仍中——yes/no**。

## 谓词现状(总指导 2026-07-13 亲读)

- 本体 `workbench_sqlite_importer.rs:969-985`:Object→键小写化+`-`→`_` 后对 `SENSITIVE_KEY_PARTS`(:62-73,10 项:prompt_body/secret/**token**/credential/keychain/oauth/provider_credential/full_transcript/transcript_body/rollout_body)**子串**匹配,并递归值;Array→递归;String→对 `SENSITIVE_STRING_MARKERS`(:74-79,4 项,不含 token)子串匹配;**数值/布尔/null→false**。
- 调用点仅两处:`:187`(主 store)与 `:235`(sidecar),均在 dry_run 拒收路径。`production_apply.rs:2169` 只是注释。
- 既有测试:`:1241 sqlite_importer_dry_run_rejects_sensitive_fixture`;preflight `:711` 是**文件名**闸,与本谓词无关、不许碰。
- 误报机理:`estimated_tokens: 123` 的**值**是数值(天然 false),纯被**键名**子串 `token` 击中。

## 修法(拍死方向,执行照此)

**精确键+数值型双保险豁免**——在键匹配处加唯一例外:

> 归一化后的键 **全等于** `estimated_tokens` 或 `max_estimated_tokens`,**且**其值为 JSON 数值(`Value::Number`)→ 跳过键名拒收;其余路径字节级不变。

- **不做**词界/分段匹配(`"tokens"` 段匹配会放过 `api_tokens`/`auth_tokens` 这类真敏感键——那是放宽,禁止);
- **不做**通配 `*_tokens` 豁免(同理);
- 值为字符串的 `estimated_tokens: "xxx"` **仍拒**(豁免只对数值成立);
- `SENSITIVE_STRING_MARKERS`、递归结构、两个调用点、preflight 文件名闸零碰。
- 预计 diff:谓词处 ≤15 行 + 测试。

## 允许写入

- `workbench_sqlite_importer.rs`(谓词+其 `#[cfg(test)]`)
- `workbench_sqlite_production_apply.rs` **仅限** `sqlite_m3_live_snapshot_level_b_rehearsal_and_reconcile`(#[ignore]·env 门)——从"记录 blocker 早退"改为**断言完整往返完成**
- 新证据档 evidence/

## 禁止事项(红线)

1. 除上述两文件两区域外零 diff;**SENSITIVE_KEY_PARTS/SENSITIVE_STRING_MARKERS 清单一项不许删改**。
2. 不碰 preflight 文件名闸/stop_write 禁词标记/任何其它安全谓词。
3. live 根只读(copy-out/hash);不写真实根/生产 DB;不翻闸。
4. 零新 command/sidecar;棘轮不碰。发现豁免不足以放行 live 主 store(还有第二误报源)→**停下报,不擅自扩豁免**。

## 验收(预写死)

- **不放宽矩阵(核心)**:新测试逐项断言 10 个 KEY_PARTS 的典型键仍拒(`auth_token`/`api-token`/`secret_x`/`provider_credential`/`keychain_ref`/`oauth_x`/`prompt_body`/`full_transcript`/`transcript_body`/`rollout_body`/嵌套/数组内对象);`estimated_tokens:"字符串"` 仍拒;`tokens`(裸键,数值)仍拒(非豁免键)。
- **误报闭合**:`estimated_tokens: 123`/`max_estimated_tokens: 456`(含 `-` 变体)通过;**真实 live 快照只读拷贝 dry_run 不再 rejected_sensitive**。
- **M3 缺口闭合(本包报偿)**:`WORKBENCH_M3_LIVE_ROOT` 指向新鲜只读拷贝跑 ignored 测试→**完整 JSON→SQLite→JSON 往返完成并对账通过**(此前只到"记录 blocker")。
- 通用:`cargo test --lib` 基线 **903/0/44 只增不减**;`git diff --check`;fmt 仅历史三漂移;shape gate 零净增;真实根 hash 前后一致。

## 必须回传(10 项)

同模板;第 10 项「被闸拦过的事」无也写"无"。**附:修改前后谓词函数全文**(diff 小,直接贴进回传供总指导逐行读)。

## 总指导回收动作

亲跑全库+矩阵测试;**逐行读谓词 diff**(就一件事:豁免是否恰好两个键+数值型,其余字节不变);live 拷贝 dry_run 亲验;真实根 hash 亲核 → 接受/需改/废弃;commit 问一次。
