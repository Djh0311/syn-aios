# D0D01 完整 main 源码与当前 WIP 迁移到 5600X WSL

阶段：stage-12 Syn 5600X/WSL C2 长期 SSH 开发通道与 D 源码迁移
目标：把 Mac linked worktree 的 main 完整可达历史，以及冻结后的 tracked/untracked WIP，迁到 synadmin@100.98.94.76:47123:/home/synadmin/workspace/syn；不搬可重建产物、真实环境配置、凭据或活动数据。
干完的标准：两端 bundle SHA-256、HEAD、tree、允许 refs、784 个 main 可达提交、git fsck --full --strict、WIP status/path/mode/size/SHA-256 全部一致；目标 .git 为独立目录；排除项计数为零；只签 SOURCE_BYTES_MATCH 后停止。

允许动：
- docs/harness/authorization.json
- docs/harness/plan.md
- docs/harness/stages/stage-12.md
- docs/harness/leaves/D0D01-syn-full-source-migration-to-5600x-wsl.md
- docs/harness/unfinished/D0D01-syn-full-source-migration-to-5600x-wsl.md
- docs/harness/done/2026-08/D0D01-syn-full-source-migration-to-5600x-wsl.md
- docs/harness/unfinished/D0C04-syn-5600x-wsl-persistent-ssh-development-channel.md
- docs/harness/unfinished/D0C05-syn-5600x-wsl-persistent-ssh-restart-validation.md
- docs/harness/audit/2026-08.jsonl
- docs/harness/usage/.turn
- /Users/yoyi/.ssh/syn_5600x_wsl_ed25519 [新增，只用于认证，不读取或输出]
- /Users/yoyi/.ssh/known_hosts [新增，只读校验 host key]
- /private/tmp/syn-source-migration-9103c3b-20260816T003730/ [新增，第一次本地失败半成品只读保留，不再写入或删除]
- /private/tmp/syn-source-migration-9103c3b-20260816T005527/ [新增，第二次本地失败候选只读保留，不再写入或删除]
- /private/tmp/syn-source-migration-9103c3b-20260816T005925/ [新增，Attempt3 本地失败候选只读保留，不再写入或删除]
- synadmin@100.98.94.76:47123:/home/synadmin/workspace/.syn-source-migration-9103c3b-20260816T005925/ [新增，Attempt3 远端失败候选只读保留，不再写入或删除]
- /private/tmp/syn-source-migration-9103c3b-20260816T012913/ [新增，Attempt4 本地失败候选只读保留，不再写入或删除]
- synadmin@100.98.94.76:47123:/home/synadmin/workspace/.syn-source-migration-9103c3b-20260816T012913/ [Attempt4 远端未创建，路径保留为禁止复用]
- /private/tmp/syn-source-migration-lint-20260816T015545/ [新增，Attempt5 lint-only root；只允许脚本/argv/静态收据，不生成迁移物、不 SSH]
- /private/tmp/syn-source-migration-9103c3b-20260816T015545/ [新增，Attempt5 唯一正式本地 root；仅接收 lint 后冻结 SHA 的脚本并全量重建]
- synadmin@100.98.94.76:47123:/home/synadmin/workspace/.syn-source-migration-9103c3b-20260816T015545/ [新增，Attempt5 唯一远端 staging]
- /private/tmp/syn-source-migration-attempt6-20260816T023204/ [新增，Attempt6 根；只允许全新单调编号本地子候选，不覆盖旧子候选]
- synadmin@100.98.94.76:47123:/home/synadmin/workspace/.syn-source-migration-attempt6-20260816T023204/ [新增，Attempt6 根；只允许全新单调编号远端子候选，不覆盖旧子候选]
- synadmin@100.98.94.76:47123:/home/synadmin/workspace/syn/ [新增]

## 固定边界

- Git 历史只取 refs/heads/main，不取其他 heads、remote-tracking refs、refs/stash、reflog 或 dangling objects。
- 当前已提交的 27 字节合成 .env 测试 fixture 随完整 main 历史保留；任何真实 .env、密钥、凭据、登录态和活动 SQLite 不迁。
- 现有专用 SSH 私钥只作为标准认证输入，禁止读取、输出、复制、创建、修改或删除；known_hosts 只读使用，禁止自动接受新 host key。
- tracked WIP 用 full-index binary patch；untracked WIP 只能来自 git ls-files --others --exclude-standard -z 的独立 NUL 清单和 no-clobber 归档。
- untracked 归档必须以 `COPYFILE_DISABLE=1` 且显式 `--no-mac-metadata --no-xattrs --no-acls --no-fflags` 创建；本地用 Python tarfile 原始成员视图验证恰好等于 NUL 清单、全部为普通文件、拒绝绝对路径、`..`、重复项与非普通文件，`._*` 为零且无 LIBARCHIVE/SCHILY xattr、ACL 或 file-flags pax header；validator 必须从 tar 流内复核 mode/size/SHA-256，不提取。
- Python validator 必须持久生成 `untracked-raw-members.tsv`（ordinal/path/type/mode/size/content-sha256/pax-header-count）与 `untracked-raw-validation.txt`（expected/actual/AppleDouble/xattr/ACL/file-flags/pathset digest/result），两者纳入 `artifacts.sha256` 及其 digest 并传输；远端 GNU tar 的原始路径/类型/mode/size与归档流 SHA 收据必须和这两份收据、NUL 清单三方一致后才允许 clone。
- 不迁 target、node_modules、dist、缓存或其他 ignored 生成物；不执行依赖安装、构建、测试或产品运行。
- 不 Git add/commit/push/merge/rebase/reset/clean/stash；不改产品代码；不执行 D0C04/D0C05；不激活 M5–M10；不实现 Headless Core；不切 Primary/epoch。
- 首次远端写入前，纯本地脚本/argv/引用错误只阻断 lint PASS，不构成远端硬停止，也不需要新的用户批准；只允许在独立 lint-only root 内修正脚本，不覆盖旧迁移候选、不生成 bundle/tar/patch/manifest/payload、不 SSH。静态门必须要求 `bash -n` 通过、shell 中任何 standalone `+` token 为零（`find -exec` 也使用 `\\;`）、关键 argv 逐项白名单审计、artifact 文件表精确 22 项且唯一、payload 成员精确 24 项且唯一，并用实际冻结清单向 `/dev/null` 完成无落盘 tar 演练。脚本/validator SHA 冻结后，正式本地 root 只接收相同字节。
- Attempt6 按用户最新“别停直接做到迁移完成”持续执行：本地工具错误或候选校验失败时保留失败子候选并换全新本地子候选全量重建；传输中断时保留失败远端子候选并换全新远端子候选，不覆盖、不清理、不在同一路径重放。目标目录若出现任何条目仍禁止覆盖，只能只读判明碰撞来源后报告阻断。
- staging 与产物使用 `umask 077`；checkout/extract 先使用 `umask 022`，再只依据冻结且已验真的 manifest 对允许的普通文件逐项恢复 `600/644/755`，最后严格复核 mode/path/type/size/SHA。禁止用宽泛递归 chmod。

## 本地打包 Attempt1 停止回执

- run-id 20260816T003730 在生成 bundle 之前停止；敏感路径断言错误地拦截了已经获批的 27 字节合成 .env fixture。
- 已生成的局部 status/refs/patch/WIP manifest/untracked tar 与未完成 worktree manifest 保留在原 0700 目录；main.bundle 不存在。
- 该停止没有触碰源仓库字节，没有建立远端 staging，也没有修改目标目录。旧目录不再写入、不覆盖、不删除；Attempt2 使用全新 run-id 20260816T005527，并只为精确 synthetic fixture 增加 allowlist。

## 本地打包 Attempt2 停止回执

- run-id 20260816T005527 已生成 main.bundle、完整 worktree manifest、WIP patch/untracked tar，并分别通过 bundle verify、源 fsck 和前置 hash；但生成 artifacts.sha256 时，脚本中的多行续行被错误拼成字面加号参数，shasum 退出 1，因此未签发本轮 PASS。
- 该候选 main.bundle 大小 68452941 bytes，未传输；远端 staging 仍不存在，目标仍为空。
- 旧目录不再写入、不覆盖、不删除；Attempt3 使用全新 run-id 20260816T005925，并以逐文件循环生成汇总 checksum，避免续行转义。

## 打包与传输 Attempt3 停止回执

- run-id 20260816T005925 的 main-only bundle、patch、manifest 和 19 项传输文件 SHA-256 在两端全部通过；bundle 只含 `refs/heads/main`，传输未改写字节。
- WSL GNU tar 在恢复前发现 `untracked.tar` 原始成员为 26 项，而冻结清单为 15 项；多出的 11 项均为 `._*` AppleDouble。独立 Python tarfile 读取同一归档得到同样的 26/11，且 11 个源文件的 `com.apple.provenance` xattr 与 11 个 AppleDouble 路径逐项对应，故原因是 macOS 打包元数据而非传输损坏。
- 按任一清单不一致立即停止：未 clone、未应用 patch、未恢复 untracked；目标保持 0 项。旧本地和远端候选不复用、不覆盖、不删除。
- 用户收到该停止回执后再次明确批准全新 Attempt4。Attempt4 使用 run-id 20260816T012913，重新冻结并重建整套包；本地必须以 Python tarfile 原始成员视图先通过 15 项精确校验，远端仍以 GNU tar 做独立门。

## 本地打包 Attempt4 停止回执

- run-id 20260816T012913 在首次远端写入前停止。build 脚本第 146 行的续行在落盘时变成字面 `+` 参数，实际 argv 形如 `/usr/bin/tar + --format=ustar + ...`；bsdtar 因此 exit 1。第 148、180、262、278、281 行还有同类未执行缺陷。
- 本地 0700 候选共 15 个前置文件；未生成 `untracked.tar`、raw receipts、worktree manifest、`main.bundle`、local restore、artifacts checksum 或 payload。远端同名 staging 未创建，目标未写。
- 源 HEAD/tree/main/origin-main/index/status/patch/fsck 复核一致；旧候选只读保留。用户随后明确回复“继续”，并纠正硬停止范围；Attempt5 使用全新 run-id 20260816T015545。

## 本地打包 Attempt5 停止回执

- run-id `20260816T015545` 的 main-only bundle、tracked patch、AppleDouble-free untracked tar 与本地恢复均已生成；两份 3596 路径 manifest 的 path/type/Git mode/size/SHA-256 零差异。
- 唯一差异是 fs_mode：3561 个 tracked `644→600`、5 个 executable `755→700`、15 个 untracked `644→600`；另 15 个源端原本为 `600` 的 tracked 文件保持 `600`。根因精确对应 build 脚本全局 `umask 077` 对 clone/extract 新文件的影响。
- 该候选严格判 FAIL 并只读保留；未生成最终 payload，未创建 Attempt5 远端 staging，未写目标。用户随后要求不再为此类可修正问题停下，Attempt6 使用 run-id `20260816T023204` 并持续做到迁移完成。

## 步骤

1. 重新冻结源 HEAD/tree/main/origin-main/refs、status、index、WIP 逐文件 hash、submodule/LFS 和敏感/生成物边界。
2. 只读预检目标目录、远端临时路径、磁盘/inode、Git/tar/SHA-256 与 Codex CLI；任一碰撞或缺少必要工具即停止。
3. 在唯一 Mac 临时目录生成 main-only bundle、tracked WIP patch、untracked NUL 清单/归档与固定格式 manifest；本地验证 bundle、patch、归档和全部 SHA-256。
4. 再次确认源未漂移后，通过现有 SSH 通道传输到唯一远端临时目录；每个远端文件先以不存在为前提创建，传输中断即停止。
5. 远端先验 bundle/归档 SHA 和 refs，再把 bundle clone 到仍为空的目标；验证 clean HEAD/tree/fsck 后应用 tracked patch，并在逐项无碰撞后恢复 untracked。
6. 双端比较 HEAD/tree/允许 refs、Git 状态、逐文件 manifest/哈希、排除项和 fsck；全部通过后签 SOURCE_BYTES_MATCH 并停止，不自动进入后续阶段。
