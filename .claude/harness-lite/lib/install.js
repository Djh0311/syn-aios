'use strict';
const fs = require('fs');
const os = require('os');
const path = require('path');
const { spawnSync } = require('child_process');
const io = require('./io.js');
const authorization = require('./authorization.js');
const GIT_ENV = Object.freeze({ PATH: '/usr/bin:/bin', LANG: 'C', LC_ALL: 'C' });
const gitRun = (args, opts = {}) => spawnSync('/usr/bin/git', args, { ...opts, env: GIT_ENV });

const VERSION = '0.8.0';
const SOURCE_COMMIT = 'working-copy:90b5f8c14a71a415d87374821fff4af35ad6372e';
const RUNTIME = '.claude/harness-lite';
const MANIFEST = `${RUNTIME}/ownership.json`;
const EVENTS = ['SessionStart', 'UserPromptSubmit', 'Stop', 'PreToolUse'];
const OWNED05_REQUIRED = `bin/hl.js hooks/README.md hooks/pre-push.js hooks/session-start.js hooks/stop.js hooks/user-prompt.js lib/authorization.js lib/checks.js lib/close-stage.js lib/done.js lib/extensions.js lib/gate.js lib/git-pre-push.js lib/gitfacts.js lib/hookio.js lib/leaf.js lib/lifecycle.js lib/limits.js lib/map.js lib/mistakes.js lib/ownership.js lib/report.js lib/tests-pick.js lib/tree.js lib/usage.js skills/root-cause.md skills/split-work.md skills/test-first.md settings-snippet.json`.split(' ').map((x) => `${RUNTIME}/${x}`).concat('.codex/harness-lite/hooks-snippet.json');
const LEGACY_CONTROL = ['.codex/harness-lite/hooks-snippet.json', 'docs/harness/policy.json', 'docs/harness/authorization.json', 'docs/harness/authorization.example.json', 'docs/harness/templates/leaf.md', 'docs/harness/templates/report.md'];
const PREVIOUS_06_INSTRUCTION = `<!-- HARNESS-LITE:BEGIN -->
Harness Lite 内部规则：用户用清晰自然语言给目标和边界，直接生效；用户日常命令为 0。读取 docs/harness/ 的 plan、唯一 current leaf 及其 stage；清晰新目标且没有 current leaf 时，由模型直接创建最小 stage/leaf 并引用本轮 Hook 来源收据，不让用户操作内部状态。非 push 偏差只报告；push 使用精确一次性确认。文件位置是生命周期事实；退场只原子移动一个文件。报告必须分 Harness、产品、证据、载体，不把 working copy 或离线 fixture 冒充发布或真实运行。
<!-- HARNESS-LITE:END -->`;
const PREVIOUS_07_INSTRUCTION = `<!-- HARNESS-LITE:BEGIN -->
Harness Lite 内部规则：用户用清晰自然语言给目标和边界，直接生效；用户日常命令为 0。读取 docs/harness/ 的 plan、唯一 current leaf 及其 stage；leaf 是模型维护的工作投影，不是授权票或范围最高权威。发现仍服务同一目标的必要相邻路径，模型更新 leaf 并继续；按当前目标影响、后果、可信可能和直接证据做最小处理，低概率低影响理论问题只报告。普通 leaf 范围外只报告；明确“不许动”的路径和 push 按既有停点处理。文件位置是生命周期事实；退场只原子移动一个文件。报告必须分 Harness、产品、证据、载体，不把 working copy 或离线 fixture 冒充发布或真实运行。
<!-- HARNESS-LITE:END -->`;
const OFFICIAL_06 = Object.freeze({
  runtimeVersion: '0.6.0', sourceCommit: 'working-copy:640ad4d0d81828cd80bfee9ff6be6c87d20a7aa6',
  packageDigest: 'sha256:2f644436a6d69af6ed51ca007f250df1daa0a1a39c7bb43726d69226bc338b92',
  instructionBlockDigest: 'sha256:ff7591b87500d31e2fc1400052db4c1e10efb396db881e52642a6ca73f900b4c',
  managedPaths: Object.freeze(['bin/hl.js', 'hooks/README.md', 'hooks/dispatcher.js', 'lib/hook.js', 'lib/install.js', 'lib/io.js', 'lib/limits.js', 'lib/tree.js', 'lib/work.js', 'skills/root-cause.md', 'skills/split-work.md', 'skills/test-first.md']),
});
const OFFICIAL_07 = Object.freeze({
  runtimeVersion: '0.7.0', sourceCommit: 'working-copy:640ad4d0d81828cd80bfee9ff6be6c87d20a7aa6',
  packageDigest: 'sha256:99323df550792f5dc34a33ae7e09df17fae727f5c8d20195f736e746057bafb3',
  hookDefinitionDigest: 'sha256:9dcc3c8b3f4c028956aa42ff34ca21e807903dff9d407d947837ac39186e5054',
  instructionBlockDigest: 'sha256:4a87ff5ab56331b588057614ab5f1c824589017088b38404ea4923bbc3fe9320',
  managedPaths: Object.freeze(['bin/hl.js', 'hooks/README.md', 'hooks/dispatcher.js', 'lib/hook.js', 'lib/install.js', 'lib/io.js', 'lib/limits.js', 'lib/tree.js', 'lib/work.js', 'skills/root-cause.md', 'skills/split-work.md', 'skills/test-first.md']),
});
// 0.4 是一次受冻结迁移输入，不是可泛化的 legacy 认领。安装器只接受
// 19be06b 产生的 46-key ownership、28 个 0644 runtime 文件和四份精确旧 Hook
// 配置；它不会经过 0.5 bridge，也不会把旧 manifest 改标签。
const OFFICIAL_04 = Object.freeze({
  runtimeVersion: '0.4.0', sourceCommit: '19be06ba91e5c283bbaea1ce1ba2b2af263260bd',
  runtimeDigest: 'sha256:e5be6ca3d7bb443e8d1ef1b29ab7178f9bfc494daf19bad91c5763dc882f0cd2', mode: '0644',
  runtimePaths: Object.freeze(['bin/hl.js', 'hooks/README.md', 'hooks/pre-push.js', 'hooks/session-start.js', 'hooks/stop.js', 'hooks/user-prompt.js',
    'lib/authorization.js', 'lib/checks.js', 'lib/close-stage.js', 'lib/done.js', 'lib/extensions.js', 'lib/gate.js', 'lib/git-pre-push.js',
    'lib/gitfacts.js', 'lib/hookio.js', 'lib/leaf.js', 'lib/lifecycle.js', 'lib/limits.js', 'lib/map.js', 'lib/mistakes.js', 'lib/ownership.js',
    'lib/report.js', 'lib/tests-pick.js', 'lib/tree.js', 'lib/usage.js', 'skills/root-cause.md', 'skills/split-work.md', 'skills/test-first.md']),
  files: Object.freeze({
    'AGENTS.md': 'c86445f67d5d7af1d33f18716cad7871f2b34d32968c9fad3129befc5c28ffba',
    'CLAUDE.md': 'b7ca0575486c601cc1fc90460c7946e3068733f4e9e0cd2eed47b8f494564162',
    'docs/harness/plan.md': 'cbddf80e6cfcd60853df119536c0cca7e58edfb40514347b1caabe415e9e4499',
    'docs/harness/stages/stage-01.md': '3c848f85bd0df34a3c0030a97fadb5616925029e695b9096c47fba2572e1d16a',
    'docs/harness/leaves/.keep': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'docs/harness/unfinished/.keep': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'docs/harness/done/.keep': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'docs/harness/audit/.keep': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'docs/harness/reports/.keep': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'docs/harness/usage/.keep': 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855',
    'docs/harness/MISTAKES.md': 'c70ef7f231f29eee5699d23c6aa3d8e0d5bf6b67fdf7d17a34c99de47001bb8e',
    'docs/harness/policy.json': '946cbfac39b2d25b78a6436fe7f8f7dcf481538ec4ff628713b15307275fc003',
    'docs/harness/checks.json': 'fc260050ffc0a260892e6d1b131d8fb736ed076209fa0197b9b3a0da9ed0b433',
    'docs/harness/authorization.example.json': '86ad22df379c259f112c94fea650c2e8ace045ef4aa03e1e19676d1e1e03f6ed',
    'docs/harness/templates/leaf.md': 'c5c4aa24a2dd67fe6345fb756fcd2af74066c8668147ee09145f7954e0d2f89e',
    'docs/harness/templates/report.md': '991c089fde05188a725a7ebcc74dc9da81f5ffb6ea0405c701b01caef1411aff',
    '.claude/harness-lite/skills/root-cause.md': '84c08273bf2c355c83db5f41929b05c23eb7c1e3a1904f130c2bb32e51e1b818',
    '.claude/harness-lite/skills/test-first.md': 'd4d4d12cb8c822b48b4b571f092a1e731a450b54c6677333932fae52eeb54f76',
    '.claude/harness-lite/skills/split-work.md': 'bfa763a4d256daafbe6dd3ea74ebb72db2173753aa1931a5c04daeea11943500',
    '.claude/harness-lite/lib/authorization.js': 'f2aef67a9785bbcd26f15c596c68b3933795100b61f6181e6858e047a3aa38e9',
    '.claude/harness-lite/lib/checks.js': 'a6eb8168036acbbb9ca9fcbd906db638c90e41354a646f7e428d9e6ba32142da',
    '.claude/harness-lite/lib/close-stage.js': 'b4ba1d2f75caa885eb311de40d26ab3101199cbfb16b954b0f82d316dc96118e',
    '.claude/harness-lite/lib/done.js': 'ee8e2cc49d6565c006e35e546348f5324933ebafd982b6c86af167e031be3eb7',
    '.claude/harness-lite/lib/extensions.js': '15360fa4e353e238bd427369684f237807b1d0d466a575b441839d061d033110',
    '.claude/harness-lite/lib/gate.js': 'e01a27ac6866ea9c517534e0e8d67304687924cf2889aae3b167de4b11ecb3cf',
    '.claude/harness-lite/lib/git-pre-push.js': 'e43125255b387fa2679de703f34a84bdf3dd8f768f02a38fcd206268ef135146',
    '.claude/harness-lite/lib/gitfacts.js': 'e1d1e17d04ef51217b7c229f101884f4c86339a562285cd8217e77ee41dbfb20',
    '.claude/harness-lite/lib/hookio.js': '6f88ae429343172526e60f7ed7efeb29a1b1519ddf5f9a2c75c31cb7ef26f8b0',
    '.claude/harness-lite/lib/leaf.js': 'bb697a08a15e680598d2d228b6c25682d6e6bbc77aa49693586aef2d074d171e',
    '.claude/harness-lite/lib/lifecycle.js': 'bcfc2a525464b64b554e0d30e8e00c3af729314f091875220630b1914c8213ef',
    '.claude/harness-lite/lib/limits.js': 'eb604eb182a74d8e863c4c7c8510a26418711d8f0d805cef423f35d9b7858487',
    '.claude/harness-lite/lib/map.js': '2e8398f1e5dcfd0ee705f00cd54d5228035fa8f48938a0a08a67e6db68fb904d',
    '.claude/harness-lite/lib/mistakes.js': 'daac9b485f46be158267ed1f5414b12b5cfcf32e9c6519d31a9abe81d0b82854',
    '.claude/harness-lite/lib/ownership.js': '9fa134dd588ae646854c54acbebbc26cc1b024641009a6ea610c1c68b15fea66',
    '.claude/harness-lite/lib/report.js': '7f9f68e9724e96804a60b9b88efe749fc0107c07337946f1254dc78d0ec325cd',
    '.claude/harness-lite/lib/tests-pick.js': 'fcc015deb579ef930b6a6c332ac83a98c122d268cf3dcd4ba913e1d061f3882d',
    '.claude/harness-lite/lib/tree.js': '0b279bab31fe26c5f9b6959938eae34b45e90a9accfb6d49f2d4e08a46e198da',
    '.claude/harness-lite/lib/usage.js': 'cae589eaee28ff030d26fb001a04b723835c8f7c77ab3e99f956a06ae7a7c95b',
    '.claude/harness-lite/hooks/session-start.js': 'c4c42efebc3b1540e07402f9ae4e5f26b6aea1' + '631abff3637f2c24ad3ac396e2',
    '.claude/harness-lite/hooks/user-prompt.js': '3d6271e8c5bc285e751fba76a4f25b3caea00333f929f7524dfc5d656218f993',
    '.claude/harness-lite/hooks/stop.js': '2961ad5811961e52057daf716d0498a35d4350ed31d23cd30f673b71378ec0e6',
    '.claude/harness-lite/hooks/pre-push.js': 'cacb713d342e4f407495fdd8df4811b85f0cbed2baa1dec8c3841f06812c51cc',
    '.claude/harness-lite/hooks/README.md': '523846b14814260676337f52e59e4d8799eb9fdcad85424933ff6bce15c29740',
    '.claude/harness-lite/bin/hl.js': '4bb31f3ba2c90b18703e1c35750f1befd0ecbb509bb4a8ed4e48c60b407cf7cf',
  }),
  snippets: Object.freeze(['.claude/harness-lite/settings-snippet.json', '.codex/harness-lite/hooks-snippet.json']),
  contentDrift: Object.freeze(['docs/harness/plan.md', 'docs/harness/MISTAKES.md', 'docs/harness/checks.json']),
  stageMissing: 'docs/harness/stages/stage-01.md',
});
const INSTRUCTION = `<!-- HARNESS-LITE:BEGIN -->
Harness Lite 内部规则：用户用清晰自然语言给目标和边界，直接生效；plan、stage、leaf、route、来源 receipt 和 Hook 文本都不能扩大用户原话。读取 docs/harness/ 的 plan、唯一 current leaf 及其 stage；只规划/只建任务进入 unfinished/，只有当前用户明确开始或继续才建立 current leaf。leaf 是模型维护的工作投影，不是授权票或范围最高权威。Stop 只有在短期 authorization.json 与当前 project/session/turn/leaf/stage 精确绑定且本轮有新产品进展时才能内部续跑；内部续跑不是新用户授权，不得扩大范围。发现仍服务同一目标的必要相邻路径，模型更新 leaf 并继续；普通 leaf 范围外只报告，明确“不许动”的路径和 push 按既有停点处理。文件位置是生命周期事实；退场只原子移动一个文件。报告必须分 Harness、产品、证据、载体，不把 working copy、离线 fixture 或协议推断冒充发布或真实运行。
<!-- HARNESS-LITE:END -->`;

const sourceRoot = () => path.join(__dirname, '..');
function runtimePaths(source = sourceRoot()) {
  const files = ['bin', 'hooks', 'lib', 'skills'].flatMap((dir) => io.list(path.join(source, dir), true));
  if (fs.existsSync(path.join(source, 'LICENSE'))) files.push(path.join(source, 'LICENSE'));
  return files.filter((file) => fs.lstatSync(file).isFile()).map((file) => path.relative(source, file).replaceAll('\\', '/')).sort();
}
const desiredMode = (rel) => rel === 'bin/hl.js' || rel === 'hooks/dispatcher.js' ? 0o755 : 0o644;
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map((key) => [key, canonical(value[key])]));
  return value;
}
function samePath(a, b) { try { return fs.realpathSync(a) === fs.realpathSync(b); } catch { return path.resolve(a) === path.resolve(b); } }
function unsafeParent(root, file) {
  let cursor = path.resolve(root); for (const part of path.relative(root, path.dirname(file)).split(path.sep).filter(Boolean)) {
    cursor = path.join(cursor, part); try { const stat = fs.lstatSync(cursor); if (stat.isSymbolicLink() || !stat.isDirectory()) return cursor; } catch (error) { if (error.code !== 'ENOENT') throw error; }
  } return null;
}
function hookConfigRoot(root) {
  const result = gitRun(['worktree', 'list', '--porcelain'], { cwd: root, encoding: 'utf8' });
  if (result.status !== 0) return path.resolve(root);
  const first = result.stdout.split('\n').find((line) => line.startsWith('worktree '));
  return first ? path.resolve(first.slice(9).trim()) : path.resolve(root);
}
function projectDefinition(root, version, frozen = false) {
  const command = frozen ? `node "$(git rev-parse --show-toplevel)/${RUNTIME}/hooks/dispatcher.js"`
    : `${JSON.stringify(path.resolve(process.execPath))} "$(/usr/bin/git rev-parse --show-toplevel)/${RUNTIME}/hooks/dispatcher.js"`;
  const handler = (timeout, canAddContext = true) => ({ type: 'command', command, timeout,
    ...(canAddContext ? { additionalContextLimit: 1800 } : {}) });
  const config = { description: `Harness Lite ${version} lifecycle hooks`, hooks: {
    SessionStart: [{ matcher: 'startup|resume|clear|compact', hooks: [handler(15)] }],
    UserPromptSubmit: [{ hooks: [handler(10)] }],
    Stop: [{ hooks: [handler(30, false)] }],
    PreToolUse: [{ matcher: '^Bash$', hooks: [handler(10)] }],
  } };
  return { config, digest: `sha256:${io.sha(JSON.stringify(canonical(config.hooks)))}`, command };
}
const definition = (root) => projectDefinition(root, VERSION);
const official07Definition = (root) => projectDefinition(root, OFFICIAL_07.runtimeVersion, true);
function manifest(root, staged, source = sourceRoot(), profile = 'project', hookPreimage = { fileExisted: true, addedEvents: [] }) {
  const managedPaths = runtimePaths(source), def = definition(root);
  return { runtimeVersion: VERSION, sourceCommit: SOURCE_COMMIT, profile, managedPaths,
    packageDigest: io.digest(staged, managedPaths), hookDefinitionDigest: def.digest,
    instructionBlockDigest: `sha256:${io.sha(INSTRUCTION)}`, hookPreimage, extensions: {} };
}
function validHookPreimage(value) {
  return !!value && typeof value === 'object' && typeof value.fileExisted === 'boolean' && Array.isArray(value.addedEvents)
    && new Set(value.addedEvents).size === value.addedEvents.length && value.addedEvents.every((event) => EVENTS.includes(event));
}
function runtimeProbe(root, runtime, operation) {
  const script = operation === 'definition'
    ? "const api=require(process.argv[1]);process.stdout.write(JSON.stringify({digest:api.definition(process.argv[2]).digest}))"
    : "const api=require(process.argv[1]);const value=api.verify06(process.argv[2]);process.stdout.write(JSON.stringify(value));process.exit(value.ok?0:1)";
  const run = spawnSync(process.execPath, ['-e', script, path.join(runtime, 'lib', 'install.js'), root], { encoding: 'utf8' });
  let value = null; try { value = JSON.parse(run.stdout || 'null'); } catch { /* malformed probe is a failure */ }
  return { ok: run.status === 0 && !!value, value, reason: String(run.stderr || run.stdout || `runtime ${operation} probe failed`).trim() };
}
function actualRuntimeFiles(runtime) {
  return io.list(runtime, true).map((file) => path.relative(runtime, file).replaceAll('\\', '/'))
    .filter((rel) => rel !== 'ownership.json' && !rel.startsWith('extensions/')).sort();
}
function verify06(root) {
  const runtime = path.join(root, RUNTIME), value = io.json(path.join(root, MANIFEST));
  if (!value || value.runtimeVersion !== VERSION || !Array.isArray(value.managedPaths)) return { ok: false, kind: 'not-current', differences: ['ownership.json'] };
  const actual = actualRuntimeFiles(runtime), expected = [...value.managedPaths].sort(), differences = [];
  const def = definition(root), allowedHookDefinitions = new Set([def.digest, previous06Digest(def)]);
  if (!['project', 'managed'].includes(value.profile)) differences.push('profile');
  if (!allowedHookDefinitions.has(value.hookDefinitionDigest)) differences.push('hookDefinitionDigest');
  if ('hookPreimage' in value && !validHookPreimage(value.hookPreimage)) differences.push('hookPreimage');
  for (const rel of expected) { const type = io.inspect(runtime, rel).type; if (type !== 'file') differences.push(`${type}:${rel}`); }
  for (const rel of actual.filter((x) => !expected.includes(x))) differences.push(`extra:${rel}`);
  let digest = null; try { digest = io.digest(runtime, expected); } catch (error) { differences.push(`identity:${error.message}`); }
  if (digest !== value.packageDigest) differences.push('packageDigest');
  const extensions = value.extensions;
  if (!extensions || typeof extensions !== 'object' || Array.isArray(extensions)) differences.push('extensions-ledger');
  else {
    const claimed = new Set();
    for (const [id, spec] of Object.entries(extensions)) {
      if (!/^[a-z0-9][a-z0-9-]*$/.test(id) || !spec || typeof spec !== 'object' || !spec.packageVersion
        || !Array.isArray(spec.managedPaths) || !/^sha256:[0-9a-f]{64}$/.test(spec.packageDigest || '')) { differences.push(`extension-schema:${id}`); continue; }
      const paths = [...spec.managedPaths].sort(), prefix = `extensions/${id}/`;
      if (!paths.length || paths.some((rel) => typeof rel !== 'string' || !rel.startsWith(prefix) || claimed.has(rel))) { differences.push(`extension-paths:${id}`); continue; }
      for (const rel of paths) { claimed.add(rel); const type = io.inspect(runtime, rel).type; if (type !== 'file') differences.push(`extension-${type}:${rel}`); }
      let extensionDigest = null; try { extensionDigest = io.digest(runtime, paths); } catch (error) { differences.push(`extension-identity:${id}:${error.message}`); }
      if (extensionDigest !== spec.packageDigest) differences.push(`extension-packageDigest:${id}`);
    }
    const physical = io.list(path.join(runtime, 'extensions'), true).map((file) => path.relative(runtime, file).replaceAll('\\', '/')).sort();
    for (const rel of physical.filter((item) => !claimed.has(item))) differences.push(`extension-extra:${rel}`);
  }
  return { ok: differences.length === 0, kind: differences.length ? 'drift' : 'current', differences, manifest: value, digest };
}
function verifyOfficial07(root) {
  const runtime = path.join(root, RUNTIME), value = io.json(path.join(root, MANIFEST));
  if (!value || value.runtimeVersion !== OFFICIAL_07.runtimeVersion || !Array.isArray(value.managedPaths)) return { ok: false, kind: 'drift', differences: ['ownership.json'] };
  const expected = [...OFFICIAL_07.managedPaths], actual = actualRuntimeFiles(runtime), differences = [];
  if (value.sourceCommit !== OFFICIAL_07.sourceCommit) differences.push('sourceCommit');
  if (JSON.stringify([...value.managedPaths].sort()) !== JSON.stringify(expected)) differences.push('managedPaths');
  if (value.packageDigest !== OFFICIAL_07.packageDigest) differences.push('packageDigest');
  if (value.hookDefinitionDigest !== OFFICIAL_07.hookDefinitionDigest) differences.push('hookDefinitionDigest');
  if (value.instructionBlockDigest !== OFFICIAL_07.instructionBlockDigest) differences.push('instructionBlockDigest');
  if ('profile' in value) differences.push('unexpected-profile');
  if (!value.extensions || typeof value.extensions !== 'object' || Array.isArray(value.extensions) || Object.keys(value.extensions).length) differences.push('extensions');
  for (const rel of expected) {
    const item = io.inspect(runtime, rel), wantedMode = (rel === 'bin/hl.js' || rel === 'hooks/dispatcher.js') ? '0755' : '0644';
    if (item.type !== 'file') differences.push(`${item.type}:${rel}`); else if (item.mode !== wantedMode) differences.push(`mode:${rel}`);
  }
  for (const rel of actual.filter((x) => !expected.includes(x))) differences.push(`extra:${rel}`);
  let digest = null; try { digest = io.digest(runtime, expected); } catch (error) { differences.push(`identity:${error.message}`); }
  if (digest !== OFFICIAL_07.packageDigest) differences.push('official-packageDigest');
  return { ok: differences.length === 0, kind: differences.length ? 'drift' : 'official-07', profile: 'project', differences, manifest: value, digest };
}
function verifyOfficial06(root) {
  const runtime = path.join(root, RUNTIME), value = io.json(path.join(root, MANIFEST));
  if (!value || value.runtimeVersion !== OFFICIAL_06.runtimeVersion || !Array.isArray(value.managedPaths)) {
    return { ok: false, kind: 'drift', differences: ['ownership.json'] };
  }
  const expected = [...OFFICIAL_06.managedPaths], actual = actualRuntimeFiles(runtime), differences = [];
  if (value.sourceCommit !== OFFICIAL_06.sourceCommit) differences.push('sourceCommit');
  if (JSON.stringify([...value.managedPaths].sort()) !== JSON.stringify(expected)) differences.push('managedPaths');
  if (value.packageDigest !== OFFICIAL_06.packageDigest) differences.push('packageDigest');
  if (value.instructionBlockDigest !== OFFICIAL_06.instructionBlockDigest) differences.push('instructionBlockDigest');
  if (value.hookDefinitionDigest !== definition(root).digest) differences.push('hookDefinitionDigest');
  if (!value.extensions || typeof value.extensions !== 'object' || Array.isArray(value.extensions) || Object.keys(value.extensions).length) differences.push('extensions');
  for (const rel of expected) { const type = io.inspect(runtime, rel).type; if (type !== 'file') differences.push(`${type}:${rel}`); }
  for (const rel of actual.filter((x) => !expected.includes(x))) differences.push(`extra:${rel}`);
  let digest = null; try { digest = io.digest(runtime, expected); } catch (error) { differences.push(`identity:${error.message}`); }
  if (digest !== OFFICIAL_06.packageDigest) differences.push('official-packageDigest');
  return { ok: differences.length === 0, kind: differences.length ? 'drift' : 'official-06', differences, manifest: value, digest };
}
function verifyOwned05(root, value) {
  const pkg = value?.version === 1 && value.managedBy === 'harness-lite' && value.packages?.core;
  if (!pkg || pkg.packageVersion !== '0.5.0' || !pkg.files || typeof pkg.files !== 'object') return { ok: false, kind: 'hybrid', differences: ['legacy-manifest-shape'] };
  const differences = [];
  for (const rel of OWNED05_REQUIRED) if (!(rel in pkg.files)) differences.push(`unowned:${rel}`);
  for (const [rel, expected] of Object.entries(pkg.files)) {
    const item = io.inspect(root, rel);
    if (item.type !== 'file') differences.push(`${item.type}:${rel}`);
    else if (item.digest !== expected) differences.push(`content:${rel}`);
  }
  const recorded = Object.keys(pkg.files).filter((rel) => rel.startsWith(`${RUNTIME}/`) && rel !== MANIFEST).map((rel) => rel.slice(RUNTIME.length + 1));
  const actual = actualRuntimeFiles(path.join(root, RUNTIME));
  for (const rel of actual.filter((x) => !recorded.includes(x) && x !== 'settings-snippet.json')) differences.push(`extra:${rel}`);
  return { ok: differences.length === 0, kind: differences.length ? 'hybrid' : 'owned-05', differences, manifest: value };
}
const exactKeys = (value, keys) => !!value && typeof value === 'object' && !Array.isArray(value)
  && Object.keys(value).sort().join(',') === [...keys].sort().join(',');
const legacy04LedgerPaths = () => [...Object.keys(OFFICIAL_04.files), ...OFFICIAL_04.snippets].sort();
const legacy04RuntimePath = (rel) => `${RUNTIME}/${rel}`;
const legacy04Control = Object.freeze(['.claude/harness-lite/settings-snippet.json', '.codex/harness-lite/hooks-snippet.json',
  'docs/harness/policy.json', 'docs/harness/authorization.json', 'docs/harness/authorization.example.json',
  'docs/harness/templates/leaf.md', 'docs/harness/templates/report.md']);
function physicalNodes(dir, prefix = '') {
  const nodes = [];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const abs = path.join(dir, entry.name), rel = path.join(prefix, entry.name).replaceAll('\\', '/'); let stat;
    try { stat = fs.lstatSync(abs); } catch { nodes.push({ rel, type: 'missing' }); continue; }
    const type = stat.isSymbolicLink() ? 'symlink' : stat.isFile() ? 'file' : stat.isDirectory() ? 'directory' : 'other';
    nodes.push({ rel, type }); if (type === 'directory') nodes.push(...physicalNodes(abs, rel));
  }
  return nodes;
}
function expectedNodes(files) {
  const nodes = new Map();
  for (const file of files) {
    nodes.set(file, 'file'); let cursor = path.posix.dirname(file);
    while (cursor !== '.') { nodes.set(cursor, 'directory'); cursor = path.posix.dirname(cursor); }
  }
  return nodes;
}

function legacy04Definition(legacyRoot, surface) {
  if (!['claude', 'codex'].includes(surface) || typeof legacyRoot !== 'string' || !path.posix.isAbsolute(legacyRoot)
    || path.posix.normalize(legacyRoot) !== legacyRoot || legacyRoot.includes('"') || legacyRoot.includes('\n')) return null;
  const command = (file, suffix = '') => `node "${path.posix.join(legacyRoot, '.claude/harness-lite/hooks', file)}"${suffix}`;
  const one = (file, timeout, suffix) => [{ hooks: [{ type: 'command', command: command(file, suffix), timeout }] }];
  return { hooks: {
    SessionStart: one('session-start.js', 15),
    UserPromptSubmit: one('user-prompt.js', 10),
    Stop: one('stop.js', 30),
    PreToolUse: one('pre-push.js', 10, ` --surface ${surface}`),
  } };
}
function strictJsonImage(file) {
  const image = fileImage(file);
  if (image.type !== 'file' || image.parent.type !== 'directory') return { ok: false, reason: '不是安全普通 JSON 文件', image };
  try { return { ok: true, image, text: image.body.toString('utf8'), value: parseJsonDocument(image.body.toString('utf8')).value }; }
  catch (error) { return { ok: false, reason: `JSON 不合规：${error.message}`, image }; }
}
function legacy04GroupRoot(event, group, surface) {
  const spec = { SessionStart: ['session-start.js', 15], UserPromptSubmit: ['user-prompt.js', 10], Stop: ['stop.js', 30], PreToolUse: ['pre-push.js', 10] }[event];
  if (!spec || !group || typeof group !== 'object' || Array.isArray(group) || !exactKeys(group, ['hooks']) || !Array.isArray(group.hooks) || group.hooks.length !== 1) return null;
  const handler = group.hooks[0]; if (!handler || !exactKeys(handler, ['type', 'command', 'timeout']) || handler.type !== 'command' || handler.timeout !== spec[1]) return null;
  const suffix = `/.claude/harness-lite/hooks/${spec[0]}"${event === 'PreToolUse' ? ` --surface ${surface}` : ''}`;
  if (typeof handler.command !== 'string' || !handler.command.startsWith('node "') || !handler.command.endsWith(suffix)) return null;
  const legacyRoot = handler.command.slice('node "'.length, handler.command.length - suffix.length);
  const expected = legacy04Definition(legacyRoot, surface);
  return expected && hookGroupKey(group) === hookGroupKey(expected.hooks[event][0]) ? legacyRoot : null;
}
function legacy04HookDocument(file, surface, opts = {}) {
  const parsed = strictJsonImage(file); if (!parsed.ok) return parsed;
  const data = parsed.value;
  if (!data || typeof data !== 'object' || Array.isArray(data) || !data.hooks || typeof data.hooks !== 'object' || Array.isArray(data.hooks)) {
    return { ok: false, reason: 'Hook 根不是 object/hooks object', image: parsed.image };
  }
  if (opts.snippet && !exactKeys(data, ['hooks'])) return { ok: false, reason: 'legacy snippet 顶层不精确', image: parsed.image };
  if (opts.snippet && JSON.stringify(Object.keys(data.hooks).sort()) !== JSON.stringify([...EVENTS].sort())) {
    return { ok: false, reason: 'legacy snippet 事件集合不精确', image: parsed.image };
  }
  const roots = [], unknown = [];
  for (const [event, groups] of Object.entries(data.hooks)) {
    if (!Array.isArray(groups) || groups.some((group) => !group || typeof group !== 'object' || Array.isArray(group) || !Array.isArray(group.hooks))) {
      return { ok: false, reason: `Hook event 不合规：${event}`, image: parsed.image };
    }
    for (const group of groups) {
      const root = EVENTS.includes(event) ? legacy04GroupRoot(event, group, surface) : null;
      if (root) roots.push({ event, root, group });
      else if (group.hooks.some(harnessishCommand)) unknown.push(event);
    }
  }
  if (unknown.length) return { ok: false, reason: `unknown Harness hook: ${[...new Set(unknown)].join(',')}`, image: parsed.image };
  for (const event of EVENTS) {
    const groups = data.hooks[event]; const matched = roots.filter((item) => item.event === event);
    if (!Array.isArray(groups) || matched.length !== 1) return { ok: false, reason: `legacy ${surface} ${event} 不是唯一精确 group`, image: parsed.image };
    if (opts.snippet && groups.length !== 1) return { ok: false, reason: `legacy snippet ${event} 有额外 group`, image: parsed.image };
  }
  const root = roots[0]?.root;
  if (!root || roots.some((item) => item.root !== root)) return { ok: false, reason: `legacy ${surface} root 不一致`, image: parsed.image };
  const expected = legacy04Definition(root, surface);
  if (!expected) return { ok: false, reason: `legacy ${surface} root 不安全`, image: parsed.image };
  if (opts.snippet && parsed.text !== `${JSON.stringify(expected, null, 2)}\n`) return { ok: false, reason: 'legacy snippet 字节不精确', image: parsed.image };
  return { ok: true, image: parsed.image, text: parsed.text, value: data, root, definition: { config: expected } };
}
function legacy04Authorization(root) {
  const parsed = strictJsonImage(path.join(root, authorization.RELATIVE_PATH)); if (!parsed.ok) return parsed;
  const value = parsed.value;
  if (parsed.image.mode !== 0o644 || unsafeParent(root, parsed.image.file)
    || !exactKeys(value, ['version', 'id', 'issuedBy', 'scope', 'summary', 'grants', 'exclusions']) || value.version !== 1
    || typeof value.id !== 'string' || !value.id || typeof value.issuedBy !== 'string' || !value.issuedBy || typeof value.summary !== 'string'
    || !Array.isArray(value.grants) || !Array.isArray(value.exclusions) || !exactKeys(value.scope, ['kind', 'id'])
    || value.scope.kind !== 'leaf' || value.scope.id !== 'D0D01') return { ok: false, reason: 'legacy 0.4 authorization 不是精确七字段 leaf scope', image: parsed.image };
  return { ok: true, image: parsed.image, value };
}
function legacy04Lifecycle(root) {
  const stage = io.inspect(root, 'docs/harness/stages/stage-12.md');
  if (stage.type !== 'file' || stage.mode !== OFFICIAL_04.mode || unsafeParent(root, stage.abs)) return { ok: false, reason: 'stage-12 不安全或不存在' };
  const leafDir = path.join(root, 'docs/harness/leaves'), doneDir = path.join(root, 'docs/harness/done/2026-08'), unfinished = path.join(root, 'docs/harness/unfinished');
  for (const dir of [leafDir, doneDir, unfinished]) {
    try { const stat = fs.lstatSync(dir); if (!stat.isDirectory() || stat.isSymbolicLink()) return { ok: false, reason: `lifecycle 目录不安全：${path.relative(root, dir)}` }; }
    catch { return { ok: false, reason: `lifecycle 目录缺失：${path.relative(root, dir)}` }; }
  }
  let statePreimage;
  try {
    const filesToFreeze = ['docs/harness/plan.md', 'docs/harness/MISTAKES.md', 'docs/harness/checks.json', 'docs/harness/stages/stage-01.md', 'docs/harness/stages/stage-12.md'];
    statePreimage = {
      files: filesToFreeze.map((rel) => ({ rel, image: fileImage(path.join(root, rel)) })),
      dirs: [leafDir, doneDir, unfinished].map((dir) => ({ dir, image: runtimeImage(dir) })),
    };
    if (statePreimage.files.some((item) => (item.rel.endsWith('stage-01.md') ? item.image.type !== 'missing' : item.image.type !== 'file') || item.image.parent.type !== 'directory')) return { ok: false, reason: 'lifecycle 状态 preimage 不安全' };
  } catch (error) { return { ok: false, reason: `lifecycle 状态 preimage 不可读：${error.message}` }; }
  const files = (dir) => fs.readdirSync(dir, { withFileTypes: true }).map((entry) => ({ entry, file: path.join(dir, entry.name) }));
  const d0d = (entry) => /^D0D01(?:[-_][A-Za-z0-9][A-Za-z0-9._-]*)?\.md$/.test(entry.entry.name);
  const leaves = files(leafDir), done = files(doneDir), current = leaves.filter(d0d), settled = done.filter(d0d);
  const unsafe = [...leaves, ...done, ...files(unfinished)].find((item) => item.entry.isSymbolicLink() || (!item.entry.isFile() && !item.entry.isDirectory()));
  if (unsafe) return { ok: false, reason: `lifecycle 非普通文件：${path.relative(root, unsafe.file)}` };
  const pending = current.length === 1 && settled.length === 0 && leaves.filter((item) => item.entry.isFile() && item.entry.name !== '.keep').length === 1;
  const ready = current.length === 0 && settled.length === 1 && leaves.filter((item) => item.entry.isFile() && item.entry.name !== '.keep').length === 0;
  const d0cRows = files(unfinished).filter((item) => item.entry.isFile() && /^D0C0[45](?:[-_].*)?\.md$/.test(item.entry.name));
  const d0c = d0cRows.map((item) => item.entry.name);
  if (d0c.filter((name) => /^D0C04(?:[-_].*)?\.md$/.test(name)).length !== 1 || d0c.filter((name) => /^D0C05(?:[-_].*)?\.md$/.test(name)).length !== 1
    || d0cRows.some((item) => (fs.lstatSync(item.file).mode & 0o777) !== 0o644)) {
    return { ok: false, reason: 'D0C04/D0C05 不在唯一 unfinished 位置' };
  }
  if (!pending && !ready) return { ok: false, reason: 'D0D lifecycle 不是 current 或 R1 状态' };
  const item = pending ? current[0] : settled[0], stat = fs.lstatSync(item.file), leafImage = fileImage(item.file);
  if (!stat.isFile() || stat.isSymbolicLink() || (stat.mode & 0o777) !== 0o644) return { ok: false, reason: 'D0D leaf 不安全' };
  const text = fs.readFileSync(item.file, 'utf8');
  if (!/^#\s*D0D01\b/m.test(text) || !/^阶段：\s*stage-12(?:\s|$)/m.test(text)) return { ok: false, reason: 'D0D leaf 不绑定 stage-12' };
  const dest = path.join(doneDir, item.entry.name), destImage = fileImage(dest);
  if (leafImage.type !== 'file' || leafImage.mode !== 0o644 || leafImage.parent.type !== 'directory') return { ok: false, reason: 'D0D leaf preimage 不安全' };
  if (pending && destImage.type !== 'missing') return { ok: false, reason: 'D0D done 目标已存在' };
  if (!sameLegacy04State(statePreimage)) return { ok: false, reason: 'lifecycle 状态在验证中发生变化' };
  return { ok: true, state: pending ? 'pending-d0d' : 'r1', source: pending ? item.file : null,
    dest, leaf: item.file, leafImage, destImage, statePreimage };
}
function verifyOfficial04(root) {
  const manifestFile = path.join(root, MANIFEST), parsed = strictJsonImage(manifestFile), differences = [];
  if (!parsed.ok) return { ok: false, kind: 'drift', differences: ['ownership.json'] };
  const value = parsed.value, required = legacy04LedgerPaths();
  if (!exactKeys(value, ['version', 'managedBy', 'packages']) || value.version !== 1 || value.managedBy !== 'harness-lite'
    || !exactKeys(value.packages, ['core']) || !exactKeys(value.packages.core, ['packageVersion', 'files'])
    || value.packages.core.packageVersion !== OFFICIAL_04.runtimeVersion || !value.packages.core.files || typeof value.packages.core.files !== 'object'
    || Array.isArray(value.packages.core.files) || JSON.stringify(Object.keys(value.packages.core.files).sort()) !== JSON.stringify(required)) {
    return { ok: false, kind: 'drift', differences: ['legacy-04-manifest-shape'] };
  }
  if (parsed.image.mode.toString(8).padStart(4, '0') !== OFFICIAL_04.mode || unsafeParent(root, manifestFile)) differences.push('ownership.json');
  const runtime = path.join(root, RUNTIME), runtimeStat = (() => { try { return fs.lstatSync(runtime); } catch { return null; } })(); let runtimePreimage = null;
  if (!runtimeStat || !runtimeStat.isDirectory() || runtimeStat.isSymbolicLink()) differences.push(`type:${RUNTIME}`);
  else {
    try { runtimePreimage = runtimeImage(runtime); } catch (error) { differences.push(`runtime-preimage:${error.message}`); }
    const expected = expectedNodes([...OFFICIAL_04.runtimePaths, 'settings-snippet.json', 'ownership.json']);
    let physical = [];
    try { physical = physicalNodes(runtime); } catch (error) { differences.push(`runtime-enumeration:${error.message}`); }
    for (const item of physical) {
      const wanted = expected.get(item.rel);
      if (!wanted) differences.push(`extra:${item.type}:${item.rel}`);
      else if (item.type !== wanted) differences.push(`runtime:${item.type}:${item.rel}`);
    }
    for (const [rel] of expected) if (!physical.some((item) => item.rel === rel)) differences.push(`missing:${rel}`);
  }
  const codexLegacy = path.join(root, '.codex/harness-lite');
  try {
    const stat = fs.lstatSync(codexLegacy);
    if (!stat.isDirectory() || stat.isSymbolicLink()) differences.push('type:.codex/harness-lite');
    else for (const item of physicalNodes(codexLegacy)) {
      if (item.rel !== 'hooks-snippet.json' || item.type !== 'file') differences.push(`extra:.codex/harness-lite/${item.type}:${item.rel}`);
    }
  } catch { differences.push('missing:.codex/harness-lite'); }
  for (const rel of required) {
    const item = io.inspect(root, rel), ledger = value.packages.core.files[rel], frozen = OFFICIAL_04.files[rel];
    if (typeof ledger !== 'string' || !/^[0-9a-f]{64}$/.test(ledger)) { differences.push(`ledger:${rel}`); continue; }
    if (frozen && ledger !== frozen) differences.push(`ledger:${rel}`);
    if (rel === OFFICIAL_04.stageMissing) { if (item.type !== 'missing') differences.push(`stage-01:${item.type}`); continue; }
    if (item.type !== 'file' || item.mode !== OFFICIAL_04.mode || unsafeParent(root, item.abs)) { differences.push(`${item.type}:${rel}`); continue; }
    if (!OFFICIAL_04.contentDrift.includes(rel) && !OFFICIAL_04.snippets.includes(rel) && item.digest !== ledger) differences.push(`content:${rel}`);
  }
  for (const rel of OFFICIAL_04.runtimePaths) {
    const item = io.inspect(runtime, rel);
    if (item.type !== 'file' || item.mode !== OFFICIAL_04.mode || unsafeParent(runtime, item.abs)) differences.push(`runtime:${item.type}:${rel}`);
  }
  try { if (io.digest(runtime, OFFICIAL_04.runtimePaths, () => 0o644) !== OFFICIAL_04.runtimeDigest) differences.push('official-runtime-digest'); }
  catch (error) { differences.push(`official-runtime:${error.message}`); }
  const snippets = [legacy04HookDocument(path.join(root, OFFICIAL_04.snippets[0]), 'claude', { snippet: true }),
    legacy04HookDocument(path.join(root, OFFICIAL_04.snippets[1]), 'codex', { snippet: true }),
    legacy04HookDocument(path.join(root, '.claude/settings.json'), 'claude'),
    legacy04HookDocument(path.join(hookConfigRoot(root), '.codex/hooks.json'), 'codex')];
  for (const result of snippets) if (!result.ok) differences.push(`legacy-hook:${result.reason}`);
  if (!samePath(hookConfigRoot(root), root)) differences.push('legacy-hook-config-root');
  const roots = snippets.filter((result) => result.ok).map((result) => result.root);
  if (roots.length !== 4 || roots.some((item) => item !== roots[0])) differences.push('legacy-hook-root');
  for (let index = 0; index < 2; index++) {
    const rel = OFFICIAL_04.snippets[index], item = io.inspect(root, rel);
    if (snippets[index]?.ok && item.digest !== value.packages.core.files[rel]) differences.push(`content:${rel}`);
  }
  const auth = legacy04Authorization(root); if (!auth.ok) differences.push(`authorization:${auth.reason}`);
  const lifecycle = legacy04Lifecycle(root); if (!lifecycle.ok) differences.push(`lifecycle:${lifecycle.reason}`);
  // A syntactically equivalent ownership rewrite is still a concurrent change:
  // this branch accepts a frozen byte-level 0.4 identity, not merely its JSON
  // meaning.  Keep the image captured before every verifier read stable until
  // the result becomes the write transaction's preimage.
  if (!sameImage(parsed.image)) differences.push('ownership-preimage-concurrent-change');
  if (runtimePreimage && !sameRuntimeImage(runtime, runtimePreimage)) differences.push('runtime-preimage-concurrent-change');
  if (differences.length) return { ok: false, kind: 'drift', differences, manifest: value };
  return { ok: true, kind: lifecycle.state === 'r1' ? 'official-04' : 'official-04-pending-d0d', profile: 'project',
    differences: [], manifest: value, legacyRoot: roots[0], lifecycle, authorization: auth, runtimePreimage };
}
function settleOfficial04(root, opts = {}) {
  root = path.resolve(root); const before = verifyOfficial04(root);
  if (!before.ok || before.kind !== 'official-04-pending-d0d') return { status: 'HOLD', kind: before.kind, wrote: false,
    reason: before.differences?.join(',') || 'D0D 不是可结算的 official 0.4 前态' };
  const source = before.lifecycle.source, dest = before.lifecycle.dest, sourceImage = before.lifecycle.leafImage, destImage = before.lifecycle.destImage, unsafe = unsafeParent(root, dest);
  if (unsafe || !sameImage(sourceImage) || !sameImage(destImage)) return { status: 'HOLD', kind: before.kind, wrote: false, reason: unsafe ? 'D0D done parent 不安全' : 'D0D preimage 已变化' };
  if (!opts.write) return { status: 'READY', kind: before.kind, wrote: false, dryRun: true, source, dest };
  try {
    opts.fault?.('d0d:before'); const again = verifyOfficial04(root);
    if (!again.ok || again.kind !== 'official-04-pending-d0d' || again.lifecycle.source !== source || again.lifecycle.dest !== dest
      || !sameImage(sourceImage) || !sameImage(destImage)) throw new Error('D0D preimage 在写前发生变化');
    opts.fault?.('move:before'); if (!sameImage(sourceImage) || !sameImage(destImage)) throw new Error('D0D preimage 在移动前发生变化');
    io.moveNoClobber(source, dest); opts.fault?.('d0d:moved');
    if (fileImage(source).type !== 'missing' || !sameFileImage(fileImage(dest), sourceImage)) throw new Error('D0D move postimage 不精确');
    const after = verifyOfficial04(root);
    if (!after.ok || after.kind !== 'official-04') throw new Error(`D0D R1 验证失败：${after.differences?.join(',') || after.kind}`);
    opts.fault?.('d0d:verified'); return { status: 'SETTLED', kind: 'official-04', wrote: true, source, dest };
  } catch (error) {
    try {
      const nowSource = fileImage(source), nowDest = fileImage(dest);
      if (nowSource.type === 'missing' && sameFileImage(nowDest, sourceImage)) io.moveNoClobber(dest, source);
      else if (nowSource.type === 'missing' || nowDest.type !== 'missing') return { status: 'PARTIAL', kind: before.kind, wrote: false, reason: `${error.message}; rollback:concurrent-change` };
    } catch (restore) { return { status: 'PARTIAL', kind: before.kind, wrote: false, reason: `${error.message}; rollback:${restore.message}` }; }
    return { status: 'HOLD', kind: before.kind, wrote: false, reason: error.message };
  }
}
function legacyUnowned(root) {
  const runtime = path.join(root, RUNTIME), actual = actualRuntimeFiles(runtime);
  const items = actual.map((rel) => io.inspect(runtime, rel));
  const anomalies = new Set(items.filter((item) => item.type !== 'file').map((item) => `${item.type}:${item.path}`));
  const rows = items.slice(0, 50).map((item) => `${item.type}:${item.mode || '-'}:${item.path}`);
  if (items.length > 50) rows.push(`…等${items.length}项`);
  let rootDigest = null;
  try { rootDigest = io.digest(runtime, items.filter((item) => item.type === 'file').map((item) => item.path)); } catch { /* 只读诊断允许缺失 */ }
  return { ok: false, kind: 'legacy-unowned', differences: rows, evidence: { files: items.length, anomalies: [...anomalies], rootDigest } };
}
function inspect(root, source = sourceRoot()) {
  const runtime = path.join(root, RUNTIME), stat = (() => { try { return fs.lstatSync(runtime); } catch { return null; } })();
  if (!stat) return { ok: true, kind: 'fresh', differences: [] };
  if (!stat.isDirectory() || stat.isSymbolicLink()) return { ok: false, kind: 'drift', differences: [`type:${RUNTIME}`] };
  const value = io.json(path.join(root, MANIFEST));
  if (!value) return legacyUnowned(root);
  if (value.runtimeVersion === VERSION) return verify06(root);
  if (value.runtimeVersion === OFFICIAL_07.runtimeVersion) return verifyOfficial07(root);
  if (value.runtimeVersion === OFFICIAL_06.runtimeVersion) return verifyOfficial06(root);
  if (value?.packages?.core?.packageVersion === OFFICIAL_04.runtimeVersion) return verifyOfficial04(root);
  if (value.runtimeVersion) return { ok: false, kind: 'drift', differences: ['runtimeVersion'] };
  return verifyOwned05(root, value);
}
function surfacePreflight(root) {
  const configRoot = hookConfigRoot(root), dirs = ['.claude', 'docs', 'docs/harness', 'docs/harness/archive', 'docs/harness/archive/legacy-0.5', ...['stages', 'leaves', 'unfinished', 'done', 'reports', 'usage'].map((x) => `docs/harness/${x}`), '.codex'];
  const files = ['AGENTS.md', 'CLAUDE.md', '.claude/settings.json', 'docs/harness/plan.md', 'docs/harness/MISTAKES.md'];
  if (path.resolve(configRoot) === path.resolve(root)) files.push('.codex/hooks.json'); else {
    const item = io.inspect(configRoot, '.codex'); if (item.type !== 'missing' && item.type !== 'directory') return [`${item.type}:${item.path}`];
    const hookFile = io.inspect(configRoot, '.codex/hooks.json'); if (hookFile.type !== 'missing' && hookFile.type !== 'file') return [`${hookFile.type}:${hookFile.path}`];
  }
  const differences = [];
  for (const rel of dirs) { const item = io.inspect(root, rel); if (item.type !== 'missing' && item.type !== 'directory') differences.push(`${item.type}:${rel}`); }
  for (const rel of files) { const item = io.inspect(root, rel); if (item.type !== 'missing' && item.type !== 'file') differences.push(`${item.type}:${rel}`); }
  return differences;
}
function stage(root, source, fault = () => {}, profile = 'project', hookPreimage = { fileExisted: true, addedEvents: [] }) {
  const parent = path.join(root, '.claude'); fs.mkdirSync(parent, { recursive: true });
  const dir = fs.mkdtempSync(path.join(parent, '.harness-lite.stage-')); fs.chmodSync(dir, 0o700);
  let preimage = null, fence = null;
  try {
    fence = directoryFence(root, dir); preimage = runtimeImage(dir);
    for (const rel of runtimePaths(source)) {
      const dest = path.join(dir, rel); fs.mkdirSync(path.dirname(dest), { recursive: true });
      fs.copyFileSync(path.join(source, rel), dest); fs.chmodSync(dest, desiredMode(rel)); preimage = runtimeImage(dir); fault(`stage:${rel}`);
    }
    const value = manifest(root, dir, source, profile, hookPreimage); fs.writeFileSync(path.join(dir, 'ownership.json'), `${JSON.stringify(value, null, 2)}\n`, { mode: 0o600 });
    preimage = runtimeImage(dir); fault('stage:manifest');
    const check = { ...value, packageDigest: io.digest(dir, value.managedPaths) };
    if (check.packageDigest !== value.packageDigest) throw new Error('stage digest mismatch');
    const ownDefinition = runtimeProbe(root, dir, 'definition');
    if (!ownDefinition.ok || ownDefinition.value.digest !== value.hookDefinitionDigest) throw new Error(`staged Hook definition mismatch${ownDefinition.reason ? `: ${ownDefinition.reason}` : ''}`);
    return { dir, manifest: value, preimage, fence };
  } catch (error) {
    const cleanup = preimage && fence ? removeRuntimeIfExact(root, dir, preimage, fence) : 'stage-preimage-unavailable';
    if (cleanup) error.message = `${error.message};${cleanup}`;
    throw error;
  }
}
function runtimeImage(dir, prefix = '') {
  const root = fs.lstatSync(dir);
  if (!root.isDirectory() || root.isSymbolicLink()) throw new Error(`runtime root 不安全：${dir}`);
  const rows = [[prefix, 'directory', root.mode & 0o777, `${root.dev}:${root.ino}`]];
  for (const entry of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) => a.name.localeCompare(b.name))) {
    const file = path.join(dir, entry.name), rel = path.join(prefix, entry.name).replaceAll('\\', '/'), stat = fs.lstatSync(file);
    const type = stat.isSymbolicLink() ? 'symlink' : stat.isFile() ? 'file' : stat.isDirectory() ? 'directory' : 'other';
    rows.push([rel, type, stat.mode & 0o777, type === 'file' ? io.sha(fs.readFileSync(file)) : type === 'symlink' ? fs.readlinkSync(file) : type === 'directory' ? `${stat.dev}:${stat.ino}` : null]);
    if (type === 'directory') rows.push(...runtimeImage(file, rel).slice(1));
  }
  return rows;
}
function sameRuntimeImage(dir, before) {
  try { return JSON.stringify(runtimeImage(dir)) === JSON.stringify(before); } catch { return false; }
}
function directoryFence(root, dir) {
  const stat = fs.lstatSync(dir);
  if (!stat.isDirectory() || stat.isSymbolicLink() || unsafeParent(root, dir)) throw new Error(`runtime directory 不安全：${dir}`);
  const fence = fileImage(path.join(dir, '.harness-lite-directory-fence'));
  if (fence.type !== 'missing' || fence.parent.type !== 'directory') throw new Error(`runtime directory fence 不安全：${dir}`);
  return fence;
}
function removeRuntimeIfExact(root, dir, image, fence) {
  try {
    if (!sameRuntimeImage(dir, image) || !sameImage(fence) || unsafeParent(root, dir)) return `${path.basename(dir)}:concurrent-change`;
    fs.rmSync(dir, { recursive: true, force: false });
    return fs.existsSync(dir) ? `${path.basename(dir)}:cleanup-incomplete` : null;
  } catch (error) { return `${path.basename(dir)}:${error.message}`; }
}
function reserveRuntimeCarrier(root, kind) {
  const parent = path.join(root, '.claude'), stat = fs.lstatSync(parent);
  if (!stat.isDirectory() || stat.isSymbolicLink() || unsafeParent(root, parent)) throw new Error(`runtime carrier parent 不安全：${parent}`);
  const container = fs.mkdtempSync(path.join(parent, `.harness-lite.${kind}-`)); fs.chmodSync(container, 0o700);
  return { container, runtime: path.join(container, 'runtime'), fence: directoryFence(root, container) };
}
function removeRuntimeCarrier(root, carrier) {
  if (!carrier) return null;
  try {
    if (!sameImage(carrier.fence) || unsafeParent(root, carrier.container)) return `${path.basename(carrier.container)}:concurrent-change`;
    if (fs.readdirSync(carrier.container).length) return `${path.basename(carrier.container)}:not-empty`;
    fs.rmdirSync(carrier.container); return fs.existsSync(carrier.container) ? `${path.basename(carrier.container)}:cleanup-incomplete` : null;
  } catch (error) { return `${path.basename(carrier.container)}:${error.message}`; }
}
function moveDirectoryNoClobber(source, dest) {
  const src = fs.lstatSync(source), parent = fs.lstatSync(path.dirname(dest));
  if (!src.isDirectory() || src.isSymbolicLink()) throw new Error('只移动安全目录');
  if (!parent.isDirectory() || parent.isSymbolicLink()) throw new Error('目录移动目标 parent 不安全');
  if (fs.existsSync(dest)) throw new Error(`目录移动目标已存在：${dest}`);
  if (src.dev !== parent.dev) throw new Error('目录移动必须位于同一文件系统');
  const args = process.platform === 'linux' ? ['-T', '-n', '--', source, dest] : ['-n', '--', source, dest];
  const moved = spawnSync('/bin/mv', args, { encoding: 'utf8' });
  if (moved.status !== 0 || fs.existsSync(source)) throw new Error(String(moved.stderr || 'directory no-clobber move 未完成').trim());
  const target = fs.lstatSync(dest); if (!target.isDirectory() || target.isSymbolicLink()) throw new Error('directory no-clobber move postimage 不安全');
}
const legacy04RuntimeAfterArchive = (image) => image.filter((row) => row[0] !== 'settings-snippet.json');
function sameLegacy04State(state) {
  return !!state && state.files.every((item) => sameImage(item.image)) && state.dirs.every((item) => sameRuntimeImage(item.dir, item.image));
}
function swap(root, built, current, fault = () => {}, expectedOldRuntime = null) {
  const stable = path.join(root, RUNTIME), strictNoClobber = !!expectedOldRuntime;
  let rollbackCarrier = null, rollback = null, oldMoved = false, newMoved = false, rollbackPreimage = null;
  const newPreimage = built.preimage || runtimeImage(built.dir), newFence = built.fence || directoryFence(root, built.dir);
  try {
    if (!sameRuntimeImage(built.dir, newPreimage) || !sameImage(newFence)) throw new Error('staged runtime 在 swap 前发生变化');
    if (current) {
      rollbackPreimage = runtimeImage(stable);
      if (expectedOldRuntime && JSON.stringify(rollbackPreimage) !== JSON.stringify(expectedOldRuntime)) throw new Error('old runtime 在 swap 前发生变化');
      rollbackCarrier = strictNoClobber ? reserveRuntimeCarrier(root, 'rollback') : null;
      rollback = strictNoClobber ? rollbackCarrier.runtime : path.join(root, '.claude', `.harness-lite.rollback-${Date.now()}`);
      if (strictNoClobber) moveDirectoryNoClobber(stable, rollback); else fs.renameSync(stable, rollback);
      oldMoved = true; fault('swap:old');
    }
    if (strictNoClobber) moveDirectoryNoClobber(built.dir, stable); else fs.renameSync(built.dir, stable);
    newMoved = true; fault('swap:new');
    const verified = verify06(root); if (!verified.ok) throw new Error(`post-swap ${verified.differences.join(',')}`);
    const self = runtimeProbe(root, stable, 'verify'); if (!self.ok) throw new Error(`installed runtime self-verify failed: ${self.reason}`);
    fault('swap:verified'); return { rollback: oldMoved ? (strictNoClobber ? rollbackCarrier.container : rollback) : null, rollbackRuntime: rollback,
      rollbackCarrier, rollbackPreimage, newPreimage, newFence, strictNoClobber, verified };
  } catch (error) {
    let failedCarrier = null, failed = path.join(root, '.claude', `.harness-lite.failed-${Date.now()}`); const recovery = [];
    try {
      if (newMoved && fs.existsSync(stable)) {
        if (!sameRuntimeImage(stable, newPreimage)) recovery.push('new-runtime:concurrent-change');
        else {
          if (strictNoClobber) { failedCarrier = reserveRuntimeCarrier(root, 'failed'); failed = failedCarrier.runtime; moveDirectoryNoClobber(stable, failed); }
          else fs.renameSync(stable, failed);
        }
      }
    } catch (restore) { recovery.push(`new-runtime:${restore.message}`); }
    try {
      if (oldMoved && fs.existsSync(rollback) && !fs.existsSync(stable)) {
        if (!sameRuntimeImage(rollback, rollbackPreimage) || (rollbackCarrier && !sameImage(rollbackCarrier.fence))) recovery.push('old-runtime:concurrent-change');
        else if (strictNoClobber) moveDirectoryNoClobber(rollback, stable); else fs.renameSync(rollback, stable);
      }
    } catch (restore) { recovery.push(`old-runtime:${restore.message}`); }
    try {
      if (newMoved && fs.existsSync(failed) && (!oldMoved || fs.existsSync(stable))) {
        if (!sameRuntimeImage(failed, newPreimage)) recovery.push('failed-runtime:concurrent-change');
        else if (strictNoClobber) {
          const cleanup = removeRuntimeIfExact(root, failed, newPreimage, failedCarrier?.fence);
          if (cleanup) recovery.push(`failed-runtime:${cleanup}`);
          else { const carrierCleanup = removeRuntimeCarrier(root, failedCarrier); if (carrierCleanup) recovery.push(`failed-runtime:${carrierCleanup}`); }
        } else fs.rmSync(failed, { recursive: true, force: false });
      }
    } catch (restore) { recovery.push(`failed-runtime:${restore.message}`); }
    if (strictNoClobber && rollbackCarrier && !fs.existsSync(rollback) && fs.existsSync(stable)) {
      const cleanup = removeRuntimeCarrier(root, rollbackCarrier); if (cleanup) recovery.push(`rollback:${cleanup}`);
    }
    if (fs.existsSync(built.dir)) { const cleanup = removeRuntimeIfExact(root, built.dir, newPreimage, newFence); if (cleanup) recovery.push(`stage:${cleanup}`); }
    error.recovery = { rollback: strictNoClobber ? rollbackCarrier?.container : rollback, failed: strictNoClobber ? failedCarrier?.container : failed, recovery };
    if (recovery.length) error.message = `${error.message};${recovery.join('|')}`; throw error;
  }
}
const harnessishCommand = (handler) => typeof handler?.command === 'string' && /(?:\.claude[\\/]harness-lite|harness[-_ ]?lite)[^\r\n]*(?:hooks?[\\/]|dispatcher)/i.test(handler.command);
const LEGACY_HOOK = { SessionStart: ['session-start.js', 15], UserPromptSubmit: ['user-prompt.js', 10], Stop: ['stop.js', 30], PreToolUse: ['pre-push.js', 10] };
const hookGroupKey = (group) => JSON.stringify(canonical(group));
function previous06Stop(expected) {
  const group = JSON.parse(JSON.stringify(expected.config.hooks.Stop[0]));
  group.hooks[0].additionalContextLimit = 1800;
  return group;
}
function previous06Digest(expected) {
  const hooks = JSON.parse(JSON.stringify(expected.config.hooks)); hooks.Stop[0] = previous06Stop(expected);
  return `sha256:${io.sha(JSON.stringify(canonical(hooks)))}`;
}
function isExactPrevious06(data, expected) {
  return EVENTS.every((event) => {
    const groups = (Array.isArray(data.hooks?.[event]) ? data.hooks[event] : [])
      .filter((group) => Array.isArray(group?.hooks) && group.hooks.some(harnessishCommand));
    const wanted = event === 'Stop' ? previous06Stop(expected) : expected.config.hooks[event][0];
    return groups.length === 1 && hookGroupKey(groups[0]) === hookGroupKey(wanted);
  });
}
function hookGroupKind(event, group, expected, allowPrevious06 = false, definitionExpected = null, legacyRoot = null, frozenDefinition = null) {
  const key = hookGroupKey(group);
  if (key === hookGroupKey(expected)) return 'current';
  if (allowPrevious06 && event === 'Stop' && key === hookGroupKey(previous06Stop(definitionExpected))) return 'previous';
  if (frozenDefinition && key === hookGroupKey(frozenDefinition.config.hooks[event]?.[0])) return 'legacy';
  const handlers = group?.hooks || [], legacy = LEGACY_HOOK[event], handler = handlers[0], keys = handler ? Object.keys(handler).sort().join(',') : '';
  const legacyCommand = legacyRoot && legacy ? `node ${JSON.stringify(path.join(legacyRoot, RUNTIME, 'hooks', legacy[0]))}${event === 'PreToolUse' ? ' --surface codex' : ''}` : null;
  if (legacyCommand && handlers.length === 1 && !group.matcher && Object.keys(group).sort().join(',') === 'hooks' && keys === 'command,timeout,type'
    && handler.type === 'command' && handler.timeout === legacy[1] && handler.command === legacyCommand) return 'legacy';
  return handlers.some(harnessishCommand) ? 'unknown' : 'external';
}
function parseJsonDocument(text) {
  let at = 0;
  const whitespace = () => { while (/\s/.test(text[at] || '')) at++; };
  function stringNode() {
    const start = at++; let escaped = false;
    while (at < text.length) {
      const char = text[at++];
      if (escaped) escaped = false; else if (char === '\\') escaped = true; else if (char === '"') break;
    }
    const raw = text.slice(start, at); return { type: 'string', start, end: at, value: JSON.parse(raw) };
  }
  function valueNode() {
    whitespace(); const start = at, char = text[at];
    if (char === '"') return stringNode();
    if (char === '[') {
      at++; whitespace(); const items = [];
      if (text[at] !== ']') for (;;) { items.push(valueNode()); whitespace(); if (text[at] === ']') break; if (text[at++] !== ',') throw new Error('invalid JSON array'); }
      if (text[at++] !== ']') throw new Error('invalid JSON array end');
      return { type: 'array', start, end: at, items, value: items.map((item) => item.value) };
    }
    if (char === '{') {
      at++; whitespace(); const properties = [], keys = new Set(), value = {};
      if (text[at] !== '}') for (;;) {
        whitespace(); if (text[at] !== '"') throw new Error('invalid JSON key'); const keyNode = stringNode();
        if (keys.has(keyNode.value)) throw new Error(`duplicate JSON key: ${keyNode.value}`); keys.add(keyNode.value);
        whitespace(); if (text[at++] !== ':') throw new Error('invalid JSON colon'); const child = valueNode();
        properties.push({ key: keyNode.value, keyNode, valueNode: child }); value[keyNode.value] = child.value;
        whitespace(); if (text[at] === '}') break; if (text[at++] !== ',') throw new Error('invalid JSON object');
      }
      if (text[at++] !== '}') throw new Error('invalid JSON object end');
      return { type: 'object', start, end: at, properties, value };
    }
    while (at < text.length && !/[\s,\]}]/.test(text[at])) at++;
    const raw = text.slice(start, at); return { type: 'primitive', start, end: at, value: JSON.parse(raw) };
  }
  const root = valueNode(); whitespace(); if (at !== text.length) throw new Error('trailing JSON content'); return root;
}
function transformHookDocument(text, expected, options = {}) {
  const root = parseJsonDocument(text); if (root.type !== 'object') throw new Error('Hook config root 不是 object');
  const hookProperty = root.properties.find((item) => item.key === 'hooks'), hooks = hookProperty?.valueNode;
  if (!hooks || hooks.type !== 'object') throw new Error('Hook config hooks 不是 object');
  const replacements = [], missing = [], pruned = new Set();
  for (const event of EVENTS) {
    const property = hooks.properties.find((item) => item.key === event), wanted = expected.config.hooks[event][0];
    if (!property) { if (options.append) missing.push(`${JSON.stringify(event)}:[${JSON.stringify(wanted)}]`); continue; }
    const array = property.valueNode; if (array.type !== 'array') throw new Error(`Hook event 不是 array：${event}`);
    const kept = array.items.filter((item) => !options.removeKinds.has(hookGroupKind(event, item.value, wanted,
      options.previous, expected, options.legacyRoot, options.frozenDefinition))).map((item) => text.slice(item.start, item.end));
    if (!options.append && kept.length === 0 && options.pruneEvents?.has(event)) { pruned.add(property); continue; }
    if (options.append) kept.push(JSON.stringify(wanted));
    const original = text.slice(array.start, array.end);
    const prefix = array.items.length ? text.slice(array.start, array.items[0].start) : text.slice(array.start, array.end - 1);
    const suffix = array.items.length ? text.slice(array.items.at(-1).end, array.end) : ']';
    const replacement = `${prefix}${kept.join(',')}${suffix}`;
    if (replacement !== original) replacements.push({ start: array.start, end: array.end, text: replacement });
  }
  if (pruned.size) {
    const properties = hooks.properties;
    for (let index = properties.length - 1; index >= 0;) {
      if (!pruned.has(properties[index])) { index--; continue; }
      const endIndex = index; while (index > 0 && pruned.has(properties[index - 1])) index--;
      const startIndex = index; let start, end;
      if (startIndex > 0) {
        start = properties[startIndex].keyNode.start;
        while (start > properties[startIndex - 1].valueNode.end && /\s/.test(text[start - 1])) start--;
        if (text[start - 1] === ',') start--;
        end = properties[endIndex].valueNode.end;
      } else if (endIndex < properties.length - 1) {
        start = properties[startIndex].keyNode.start; end = properties[endIndex + 1].keyNode.start;
      } else { start = hooks.start + 1; end = hooks.end - 1; }
      replacements.push({ start, end, text: '' }); index--;
    }
  }
  if (missing.length) replacements.push({ start: hooks.end - 1, end: hooks.end - 1,
    text: `${hooks.properties.length ? ',' : ''}${missing.join(',')}` });
  let output = text;
  for (const replacement of replacements.sort((a, b) => b.start - a.start)) output = `${output.slice(0, replacement.start)}${replacement.text}${output.slice(replacement.end)}`;
  return output;
}
const freshHookDocument = (expected) => `${JSON.stringify({ description: 'Project hooks', hooks: Object.fromEntries(EVENTS.map((event) => [event, [expected.config.hooks[event][0]]])) }, null, 2)}\n`;
function fileImage(file) {
  let cursor = path.dirname(file), parent;
  for (;;) {
    try { const stat = fs.lstatSync(cursor); parent = { path: cursor, type: stat.isDirectory() && !stat.isSymbolicLink() ? 'directory' : 'unsafe', dev: stat.dev, ino: stat.ino, real: fs.realpathSync(cursor) }; break; }
    catch (error) { if (error.code !== 'ENOENT' || cursor === path.dirname(cursor)) throw error; cursor = path.dirname(cursor); }
  }
  try {
    const stat = fs.lstatSync(file); if (!stat.isFile() || stat.isSymbolicLink()) return { file, type: 'unsafe', parent };
    return { file, type: 'file', mode: stat.mode & 0o777, body: fs.readFileSync(file), parent };
  } catch (error) { if (error.code === 'ENOENT') return { file, type: 'missing', parent }; throw error; }
}
function sameImage(expected) {
  const actual = fileImage(expected.file), parent = actual.parent, prior = expected.parent;
  return actual.type === expected.type && sameImageParent(parent, prior)
    && (actual.type === 'missing' || (actual.type === 'file' && actual.mode === expected.mode && actual.body.equals(expected.body)));
}
function sameImageParent(actual, expected) {
  return actual.type === expected.type && actual.dev === expected.dev && actual.ino === expected.ino && actual.real === expected.real;
}
function sameFileImage(actual, expected) {
  return actual.type === expected.type && (actual.type === 'missing' || (actual.type === 'file' && actual.mode === expected.mode && actual.body.equals(expected.body)));
}
function instructionPostimage(row) {
  return !!row.preimage && sameImage({ ...row.preimage, type: 'file', mode: row.mode, body: Buffer.from(row.text) });
}
function restoreImage(image) {
  if (image.type === 'missing') { fs.rmSync(image.file, { force: true }); return; }
  if (image.type !== 'file') throw new Error(`不能恢复非普通文件：${image.file}`);
  io.atomic(image.file, image.body, image.mode);
}
function writeHookPlan(plan, fault = () => {}) {
  const { file, before, text, mode, digest, preimage } = plan;
  if (!sameImage(preimage)) return { ok: false, status: 'HOLD', wrote: false, reason: 'Hook 配置或父目录在 preflight 后发生变化', file };
  if (text === before) return { ok: true, wrote: false, file, digest };
  try { fault('hooks:before'); } catch (error) { return { ok: false, status: 'HOLD', wrote: false, reason: error.message, file }; }
  if (!sameImage(preimage)) return { ok: false, status: 'HOLD', wrote: false, reason: 'Hook 配置或父目录在写入前发生变化', file };
  try {
    if (text === null) fs.unlinkSync(file); else io.atomic(file, text, mode); fault('hooks:written');
    if (text === null ? fs.existsSync(file) : fs.readFileSync(file, 'utf8') !== text) throw new Error('hooks verify failed'); fault('hooks:verified');
    return { ok: true, wrote: true, file, digest };
  } catch (error) {
    let recovery = null;
    try { if (!hookPlanPostimage(plan)) recovery = 'Hook postimage 已并发变化'; else restoreImage(preimage); } catch (restore) { recovery = restore.message; }
    return { ok: false, status: 'HOLD', wrote: false, reason: recovery ? `${error.message}；Hook preimage 恢复失败：${recovery}` : error.message, file };
  }
}
function prepareLegacy04Hooks(root, identity) {
  const configRoot = hookConfigRoot(root);
  if (!samePath(configRoot, root)) return { ok: false, status: 'HOLD', reason: 'official 0.4 只接受独立 project Hook root' };
  const legacyRoot = identity?.legacyRoot;
  const legacyCodex = legacy04HookDocument(path.join(configRoot, '.codex/hooks.json'), 'codex');
  const legacyClaude = legacy04HookDocument(path.join(root, '.claude/settings.json'), 'claude');
  if (!legacyCodex.ok || !legacyClaude.ok || legacyCodex.root !== legacyRoot || legacyClaude.root !== legacyRoot) {
    return { ok: false, status: 'HOLD', reason: 'legacy 0.4 live Hook 在 preflight 后不再精确' };
  }
  const expected = definition(root), frozenCodex = { config: legacy04Definition(legacyRoot, 'codex') }, frozenClaude = { config: legacy04Definition(legacyRoot, 'claude') };
  let codexText, claudeText;
  try {
    codexText = transformHookDocument(legacyCodex.text, expected, { append: true, previous: false, legacyRoot: null, frozenDefinition: frozenCodex,
      removeKinds: new Set(['legacy']) });
    claudeText = transformHookDocument(legacyClaude.text, expected, { append: false, previous: false, legacyRoot: null, frozenDefinition: frozenClaude,
      removeKinds: new Set(['legacy']), pruneEvents: new Set(EVENTS) });
  } catch (error) { return { ok: false, status: 'HOLD', reason: error.message }; }
  const plans = [
    { name: 'codex', file: legacyCodex.image.file, before: legacyCodex.text, text: codexText, mode: legacyCodex.image.mode,
      digest: expected.digest, preimage: legacyCodex.image },
    { name: 'claude', file: legacyClaude.image.file, before: legacyClaude.text, text: claudeText, mode: legacyClaude.image.mode,
      digest: null, preimage: legacyClaude.image },
  ];
  return { ok: true, wrote: false, plans, hookPreimage: { fileExisted: true, addedEvents: [] } };
}
function restoreLegacy04HookPlans(plans) {
  const recovery = [];
  for (const plan of [...plans].reverse()) {
    try { if (hookPlanPostimage(plan)) restoreImage(plan.preimage); else recovery.push(`${plan.name}:concurrent-change`); }
    catch (error) { recovery.push(`${plan.name}:${error.message}`); }
  }
  return recovery;
}
function writeLegacy04HookPlans(prepared, fault = () => {}) {
  const plans = prepared.plans || [];
  if (plans.some((plan) => !sameImage(plan.preimage))) return { ok: false, status: 'HOLD', wrote: false, reason: 'Hook 配置或父目录在 preflight 后发生变化' };
  const written = [];
  try {
    for (const plan of plans) {
      if (!sameImage(plan.preimage)) throw new Error(`${plan.name}:Hook 配置或父目录在写入前发生变化`);
      fault(`hooks:${plan.name}:before`); fault('hooks:before');
      if (!sameImage(plan.preimage)) throw new Error(`${plan.name}:Hook 配置或父目录在写入前发生变化`);
      io.atomic(plan.file, plan.text, plan.mode); written.push(plan); fault(`hooks:${plan.name}:written`); fault('hooks:written');
      if (fs.readFileSync(plan.file, 'utf8') !== plan.text) throw new Error(`${plan.name}:hooks verify failed`);
      fault(`hooks:${plan.name}:verified`); fault('hooks:verified');
      if (!hookPlanPostimage(plan)) throw new Error(`${plan.name}:Hook postimage 在写入后发生变化`);
    }
    return { ok: true, wrote: written.length > 0, plans };
  } catch (error) {
    const recovery = restoreLegacy04HookPlans(written);
    return { ok: false, status: 'HOLD', wrote: false, reason: recovery.length ? `${error.message}；Hook preimage 恢复失败：${recovery.join('|')}` : error.message, plans };
  }
}
function mergeHooks(root, opts = {}) {
  const identity = inspect(root);
  if (!['fresh', 'current', 'official-07', 'official-06', 'owned-05'].includes(identity.kind)) {
    return { ok: false, status: 'HOLD', reason: 'runtime identity 不允许 Hook 配置写入', file: path.join(hookConfigRoot(root), '.codex', 'hooks.json') };
  }
  const configRoot = hookConfigRoot(root), file = path.join(configRoot, '.codex', 'hooks.json'), expected = definition(root);
  const present = fs.existsSync(file), before = present ? fs.readFileSync(file, 'utf8') : null;
  let data = present ? io.json(file) : { description: 'Project hooks', hooks: {} };
  if (!data || !data.hooks || Array.isArray(data.hooks)) return { ok: false, status: 'HOLD', reason: 'invalid .codex/hooks.json', file };
  const previous = identity.kind === 'current' && isExactPrevious06(data, expected)
    && [expected.digest, previous06Digest(expected)].includes(identity.manifest?.hookDefinitionDigest);
  const legacyRoot = identity.kind === 'owned-05' ? root : null;
  const frozenDefinition = identity.kind === 'official-07' ? official07Definition(root) : null;
  const malformed = Object.entries(data.hooks).filter(([, groups]) => !Array.isArray(groups) || groups.some((group) => !group || !Array.isArray(group.hooks)));
  if (malformed.length) return { ok: false, status: 'HOLD', reason: `invalid hook groups: ${malformed.map(([event]) => event).join(',')}`, file };
  const unknown = Object.entries(data.hooks).flatMap(([event, groups]) => groups
    .filter((group) => hookGroupKind(event, group, expected.config.hooks[event]?.[0], previous, expected, legacyRoot, frozenDefinition) === 'unknown').map((group) => `${event}:${group.hooks.map((x) => x.command).join('|')}`));
  if (unknown.length) return { ok: false, status: 'HOLD', reason: `unknown Harness hook: ${unknown.join(',')}`, file };
  const hookPreimage = { fileExisted: present, addedEvents: EVENTS.filter((event) => !Object.hasOwn(data.hooks, event)) };
  let text;
  try {
    text = present ? transformHookDocument(before, expected, { append: true, previous, legacyRoot, frozenDefinition, removeKinds: new Set(['current', 'previous', 'legacy']) })
      : freshHookDocument(expected);
  } catch (error) { return { ok: false, status: 'HOLD', reason: error.message, file }; }
  const preimage = fileImage(file); if (!['file', 'missing'].includes(preimage.type) || preimage.parent.type !== 'directory') return { ok: false, status: 'HOLD', reason: 'Hook 配置路径不是安全普通文件', file };
  const plan = { file, before, text, mode: present ? fs.statSync(file).mode & 0o777 : 0o600, digest: expected.digest, preimage };
  if (opts.write) return { ...writeHookPlan(plan, opts.fault), hookPreimage };
  return { ok: true, wrote: false, file, digest: expected.digest, hookPreimage, plan };
}
const OLD_INSTRUCTIONS = {
  'AGENTS.md': new Set(['4417f9dd90075b53b668c8cab24610d112ba40e8a17741904435f22ecea7cfb4', '80d945a4ecaf76dde35d1a89e1aa624ea5504b703cda485ed7b227697c054128']),
  'CLAUDE.md': new Set(['b7ca0575486c601cc1fc90460c7946e3068733f4e9e0cd2eed47b8f494564162']),
};
function instructionPlan(root) {
  const rows = [], re = /<!-- HARNESS-LITE:BEGIN -->[\s\S]*?<!-- HARNESS-LITE:END -->/;
  for (const name of ['AGENTS.md', 'CLAUDE.md']) {
    const file = path.join(root, name), before = io.read(file, ''), block = before.match(re)?.[0], digest = io.sha(before);
    if (block && ![INSTRUCTION, PREVIOUS_07_INSTRUCTION, PREVIOUS_06_INSTRUCTION].includes(block)) return { ok: false, reason: `unknown Harness instruction edit: ${name}`, rows };
    if (!block && /Harness Lite|HARNESS-AUTHORIZATION|hl (?:done|park|auth|chain)/i.test(before) && !OLD_INSTRUCTIONS[name].has(digest)) {
      return { ok: false, reason: `unknown legacy Harness instruction: ${name}`, rows };
    }
    const text = block ? before.replace(block, INSTRUCTION) : OLD_INSTRUCTIONS[name].has(digest) ? `${INSTRUCTION}\n`
      : `${before}${before && !before.endsWith('\n') ? '\n' : ''}${before ? '\n' : ''}${INSTRUCTION}\n`;
    rows.push({ file, before, text, changed: text !== before, existed: fs.existsSync(file), mode: fs.existsSync(file) ? fs.statSync(file).mode & 0o777 : 0o644 });
  }
  return { ok: true, rows };
}
function instructionPlan04(root) {
  const rows = [];
  for (const name of ['AGENTS.md', 'CLAUDE.md']) {
    const file = path.join(root, name), image = fileImage(file), expected = OFFICIAL_04.files[name];
    if (image.type !== 'file' || image.parent.type !== 'directory' || image.mode !== 0o644
      || io.sha(image.body) !== expected || unsafeParent(root, file)) return { ok: false, reason: `official 0.4 instruction 不精确：${name}`, rows };
    rows.push({ file, before: image.body.toString('utf8'), text: `${INSTRUCTION}\n`, changed: true, existed: true, mode: image.mode, preimage: image });
  }
  return { ok: true, rows };
}
function mergeInstructions(root, opts = {}) {
  const plan = opts.plan || instructionPlan(root); if (!plan.ok) return plan;
  const changed = [];
  try { if (opts.write) for (const row of plan.rows) if (row.changed) {
    if (row.preimage && !sameImage(row.preimage)) throw new Error(`instruction:${path.basename(row.file)}:concurrent-change`);
    opts.fault?.(`instruction:${path.basename(row.file)}:before`);
    if (row.preimage && !sameImage(row.preimage)) throw new Error(`instruction:${path.basename(row.file)}:concurrent-change`);
    io.atomic(row.file, row.text, row.mode); changed.push(row); opts.fault?.(`instruction:${path.basename(row.file)}:written`);
    if (row.preimage && !instructionPostimage(row)) throw new Error(`instruction:${path.basename(row.file)}:postimage-concurrent-change`);
  } } catch (error) {
    const recovery = [];
    for (const row of changed.reverse()) try {
      if (row.preimage) { if (!instructionPostimage(row)) { recovery.push(`${path.basename(row.file)}:concurrent-change`); continue; } restoreImage(row.preimage); }
      else if (row.existed) io.atomic(row.file, row.before, row.mode); else fs.rmSync(row.file, { force: true });
    } catch (restore) { recovery.push(`${path.basename(row.file)}:${restore.message}`); }
    return { ok: false, reason: recovery.length ? `${error.message};${recovery.join('|')}` : error.message, rows: plan.rows };
  }
  return plan;
}
function skeleton(root, opts = {}) {
  const hd = path.join(root, 'docs', 'harness');
  const dirs = ['stages', 'leaves', 'unfinished', 'done', 'reports', 'usage'];
  if (opts.write) for (const dir of dirs) fs.mkdirSync(path.join(hd, dir), { recursive: true });
  const files = { 'plan.md': '# 总计划\n\n目标：由用户清晰自然语言建立。\n', 'MISTAKES.md': '# 错题本\n' };
  if (opts.write) for (const [name, text] of Object.entries(files)) if (!fs.existsSync(path.join(hd, name))) fs.writeFileSync(path.join(hd, name), text);
}
function archive05(root, checked, opts = {}) {
  const owned = checked?.manifest?.packages?.core?.files || {}, rows = LEGACY_CONTROL.filter((rel) => (owned[rel] || rel === 'docs/harness/authorization.json') && fs.existsSync(path.join(root, rel)))
    .map((rel) => ({ source: path.join(root, rel), dest: path.join(root, 'docs', 'harness', 'archive', 'legacy-0.5', rel) }));
  const unsafe = rows.map((row) => unsafeParent(root, row.dest)).find(Boolean); if (unsafe) return { ok: false, reason: `legacy archive parent 不安全：${path.relative(root, unsafe)}`, rows };
  const conflict = rows.find((row) => fs.existsSync(row.dest)); if (conflict) return { ok: false, reason: `legacy archive 已存在：${path.relative(root, conflict.dest)}`, rows };
  const moved = [];
  try { if (opts.write) for (const row of rows) { try { io.moveNoClobber(row.source, row.dest, opts.fault); moved.push(row); }
    catch (error) { try { const a = fs.statSync(row.source), b = fs.statSync(row.dest); if (a.dev === b.dev && a.ino === b.ino) fs.unlinkSync(row.dest); } catch { /* foreign dest preserved */ } throw error; } } }
  catch (error) { for (const row of moved.reverse()) try { io.moveNoClobber(row.dest, row.source); } catch { /* source retained by no-clobber */ } return { ok: false, reason: error.message, rows }; }
  return { ok: true, wrote: !!opts.write && rows.length > 0, rows };
}
function archive04(root, opts = {}) {
  const archiveRoot = path.join(root, 'docs', 'harness', 'archive', 'legacy-0.4');
  const archiveParent = opts.rows?.[0]?.archiveParent || path.dirname(archiveRoot), archiveParentExisted = opts.rows?.[0]?.archiveParentExisted ?? fs.existsSync(archiveParent);
  const archiveParentImage = opts.rows?.[0]?.archiveParentImage || fileImage(path.join(archiveParent, '.harness-lite-archive-parent-fence'));
  const archiveRootImage = opts.rows?.[0]?.archiveRootImage || fileImage(archiveRoot);
  if (archiveRootImage.type !== 'missing') return { ok: false, reason: 'legacy 0.4 archive root 已存在', rows: [] };
  const rows = opts.rows || legacy04Control.map((rel) => ({ source: path.join(root, rel),
    dest: path.join(archiveRoot, rel), root, archiveRoot, archiveParent, archiveParentExisted, archiveParentImage, archiveRootImage }));
  for (const row of rows) {
    if (!row.before) row.before = fileImage(row.source);
    if (row.before.type !== 'file' || row.before.parent.type !== 'directory' || !sameImage(row.before)) return { ok: false, reason: `legacy 0.4 control preimage 不安全：${path.relative(root, row.source)}`, rows };
    if (!sameImage(row.archiveParentImage) || !sameImage(row.archiveRootImage)) return { ok: false, reason: 'legacy 0.4 archive preimage 在计划中发生变化', rows };
    const unsafe = unsafeParent(root, row.dest); if (unsafe) return { ok: false, reason: `legacy 0.4 archive parent 不安全：${path.relative(root, unsafe)}`, rows };
    if (fileImage(row.dest).type !== 'missing') return { ok: false, reason: `legacy 0.4 archive 已存在：${path.relative(root, row.dest)}`, rows };
  }
  const moved = [];
  try {
    if (opts.write) {
      if (!rows.every((row) => sameImage(row.archiveParentImage) && sameImage(row.archiveRootImage))) throw new Error('legacy 0.4 archive preimage 在创建前发生变化');
      fs.mkdirSync(archiveParent, { recursive: true });
      if (unsafeParent(root, path.join(archiveParent, '.harness-lite-archive-parent-fence'))) throw new Error('legacy 0.4 archive parent 不安全');
      fs.mkdirSync(archiveRoot, { mode: 0o700 });
      const archiveRootFence = directoryFence(root, archiveRoot);
      for (const row of rows) row.archiveRootFence = archiveRootFence, row.archiveRootCreated = true;
    }
    if (opts.write) for (const row of rows) {
      if (!sameImage(row.before) || !sameImage(row.archiveRootFence)) throw new Error(`legacy 0.4 control 在归档前发生变化：${path.relative(root, row.source)}`);
      opts.fault?.(`archive04:${path.relative(root, row.source)}:before`);
      if (!sameImage(row.before) || !sameImage(row.archiveRootFence) || fileImage(row.dest).type !== 'missing' || unsafeParent(root, row.dest)) throw new Error(`legacy 0.4 control 在归档前发生变化：${path.relative(root, row.source)}`);
      opts.fault?.('move:before');
      if (!sameImage(row.before) || !sameImage(row.archiveRootFence) || fileImage(row.dest).type !== 'missing' || unsafeParent(root, row.dest)) throw new Error(`legacy 0.4 control 在移动前发生变化：${path.relative(root, row.source)}`);
      io.moveNoClobber(row.source, row.dest); moved.push(row);
      const dest = fileImage(row.dest); row.destParent = dest.parent;
      if (fileImage(row.source).type !== 'missing' || !sameFileImage(dest, row.before) || !sameImageParent(dest.parent, row.destParent) || !sameImage(row.archiveRootFence) || unsafeParent(root, row.dest)) {
        throw new Error(`legacy 0.4 control move postimage 不精确：${path.relative(root, row.source)}`);
      }
    }
    if (moved.some((row) => !archive04DestExact(row))) throw new Error('legacy 0.4 archive postimage 在提交前发生变化');
  } catch (error) {
    const recovery = reverseArchive04(rows);
    return { ok: false, reason: recovery.length ? `${error.message};${recovery.join('|')}` : error.message, rows };
  }
  return { ok: true, wrote: !!opts.write, rows };
}
function archive04DestExact(row) {
  const dest = fileImage(row.dest);
  return !!row.archiveRootFence && !!row.destParent && !unsafeParent(row.root, row.dest) && sameImage(row.archiveRootFence)
    && dest.type === 'file' && sameFileImage(dest, row.before) && sameImageParent(dest.parent, row.destParent);
}
function restoreInstructionRows(rows) {
  const failures = [];
  for (const row of [...(rows || [])].reverse().filter((item) => item.changed)) {
    try {
      if (row.preimage) { if (!instructionPostimage(row)) { failures.push(`${path.basename(row.file)}:concurrent-change`); continue; } restoreImage(row.preimage); }
      else {
        const actual = fileImage(row.file);
        if (actual.type !== 'file' || actual.mode !== row.mode || !actual.body.equals(Buffer.from(row.text))) { failures.push(`${path.basename(row.file)}:concurrent-change`); continue; }
        if (row.existed) io.atomic(row.file, row.before, row.mode); else fs.rmSync(row.file, { force: true });
      }
    } catch (error) { failures.push(`${path.basename(row.file)}:${error.message}`); }
  }
  return failures;
}
function reverseArchive(rows) {
  const failures = [];
  for (const row of [...(rows || [])].reverse()) {
    try {
      if (!fs.existsSync(row.dest)) continue;
      if (fs.existsSync(row.source)) { failures.push(`${path.relative(path.dirname(row.source), row.source)}:source-exists`); continue; }
      io.moveNoClobber(row.dest, row.source);
    } catch (error) { failures.push(`${path.basename(row.source)}:${error.message}`); }
  }
  return failures;
}
function reverseArchive04(rows) {
  const failures = [];
  for (const row of [...(rows || [])].reverse()) {
    try {
      const source = fileImage(row.source), dest = fileImage(row.dest);
      if (dest.type === 'missing') continue;
      if (!archive04DestExact(row)) { failures.push(`${path.basename(row.dest)}:archive-concurrent-change`); continue; }
      if (source.type === 'missing') {
        if (!row.before || !sameImageParent(source.parent, row.before.parent)) { failures.push(`${path.basename(row.dest)}:archive-concurrent-change`); continue; }
        io.moveNoClobber(row.dest, row.source);
        if (!sameImage(row.before) || fileImage(row.dest).type !== 'missing') failures.push(`${path.basename(row.dest)}:archive-postimage-concurrent-change`);
        continue;
      }
      if (source.type === 'file' && row.before?.type === 'file' && sameImageParent(source.parent, row.before.parent) && source.mode === row.before.mode && dest.mode === row.before.mode
        && source.body.equals(row.before.body) && dest.body.equals(row.before.body)) {
        if (!sameImage(row.archiveRootFence) || unsafeParent(row.root, row.dest)) { failures.push(`${path.basename(row.dest)}:archive-concurrent-change`); continue; }
        fs.unlinkSync(row.dest);
        if (fileImage(row.dest).type !== 'missing' || !sameImage(row.before)) failures.push(`${path.basename(row.dest)}:archive-postimage-concurrent-change`);
        continue;
      }
      failures.push(`${path.basename(row.source)}:source-exists`);
    } catch (error) { failures.push(`${path.basename(row.source)}:${error.message}`); }
  }
  const dirs = [...new Set((rows || []).flatMap((row) => {
    const out = []; let dir = path.dirname(row.dest);
    while (row.archiveRoot && (dir === row.archiveRoot || dir.startsWith(`${row.archiveRoot}${path.sep}`))) { out.push(dir); if (dir === row.archiveRoot) break; dir = path.dirname(dir); }
    return out;
  }))].sort((a, b) => b.length - a.length);
  for (const dir of dirs) try {
    if (!fs.existsSync(dir)) continue;
    const fence = rows?.find((row) => row.archiveRoot === dir)?.archiveRootFence;
    if (dir === rows?.[0]?.archiveRoot && (!fence || !sameImage(fence))) { failures.push(`${path.basename(dir)}:archive-dir-concurrent-change`); continue; }
    const stat = fs.lstatSync(dir); if (!stat.isDirectory() || stat.isSymbolicLink()) { failures.push(`${path.basename(dir)}:archive-dir-concurrent-change`); continue; }
    if (fs.readdirSync(dir).length === 0) fs.rmdirSync(dir);
  } catch (error) { failures.push(`${path.basename(dir)}:${error.message}`); }
  const parent = rows?.[0]?.archiveParent;
  if (parent && !rows[0].archiveParentExisted) try {
    if (fs.existsSync(parent) && fs.lstatSync(parent).isDirectory() && fs.readdirSync(parent).length === 0) fs.rmdirSync(parent);
  } catch (error) { failures.push(`${path.basename(parent)}:${error.message}`); }
  return failures;
}
function rollbackSwap(root, swapped) {
  if (!swapped) return [];
  const failures = [], stable = path.join(root, RUNTIME), strictNoClobber = swapped.strictNoClobber === true, rollback = swapped.rollbackRuntime || swapped.rollback;
  let failedCarrier = null, failed = path.join(root, '.claude', `.harness-lite.failed-install-${Date.now()}`), moved = false;
  try {
    if (fs.existsSync(stable)) {
      if (!sameRuntimeImage(stable, swapped.newPreimage)) failures.push('new-runtime:concurrent-change');
      else {
        if (strictNoClobber) { failedCarrier = reserveRuntimeCarrier(root, 'failed-install'); failed = failedCarrier.runtime; moveDirectoryNoClobber(stable, failed); }
        else fs.renameSync(stable, failed);
        moved = true;
      }
    }
  } catch (error) { failures.push(`new-runtime:${error.message}`); }
  try {
    if (rollback && fs.existsSync(rollback) && !fs.existsSync(stable)) {
      if (!sameRuntimeImage(rollback, swapped.rollbackPreimage) || (swapped.rollbackCarrier && !sameImage(swapped.rollbackCarrier.fence))) failures.push('old-runtime:concurrent-change');
      else if (strictNoClobber) moveDirectoryNoClobber(rollback, stable); else fs.renameSync(rollback, stable);
    }
  } catch (error) { failures.push(`old-runtime:${error.message}`); }
  try {
    if (moved && (!rollback || fs.existsSync(stable))) {
      if (!sameRuntimeImage(failed, swapped.newPreimage)) failures.push('failed-runtime:concurrent-change');
      else if (strictNoClobber) {
        const cleanup = removeRuntimeIfExact(root, failed, swapped.newPreimage, failedCarrier?.fence);
        if (cleanup) failures.push(`failed-runtime:${cleanup}`);
        else { const carrierCleanup = removeRuntimeCarrier(root, failedCarrier); if (carrierCleanup) failures.push(`failed-runtime:${carrierCleanup}`); }
      } else fs.rmSync(failed, { recursive: true, force: false });
    }
  } catch (error) { failures.push(`failed-runtime:${error.message}`); }
  if (strictNoClobber && swapped.rollbackCarrier && !fs.existsSync(rollback) && fs.existsSync(stable)) {
    const cleanup = removeRuntimeCarrier(root, swapped.rollbackCarrier); if (cleanup) failures.push(`rollback:${cleanup}`);
  }
  return failures;
}
function hookPlanPostimage(plan) {
  const actual = fileImage(plan.file), parent = actual.parent, prior = plan.preimage.parent;
  return (plan.text === null ? actual.type === 'missing' : actual.type === 'file' && actual.mode === plan.mode && actual.body.equals(Buffer.from(plan.text)))
    && parent.type === prior.type && parent.dev === prior.dev && parent.ino === prior.ino && parent.real === prior.real;
}
function install(root, opts = {}) {
  root = path.resolve(root); const source = opts.source || sourceRoot(), surfaceDifferences = surfacePreflight(root);
  if (surfaceDifferences.length) return { status: 'HOLD', kind: 'unsafe-surface', written: 0, differences: surfaceDifferences, reason: '安装 surface type/symlink 不安全' };
  const before = inspect(root, source);
  if (before.kind === 'official-04-pending-d0d') return { status: 'HOLD', kind: before.kind, written: 0, differences: ['D0D01-current'],
    reason: 'official 0.4 必须先独立结算 D0D01，安装器只接受 R1' };
  const legacy04 = before.kind === 'official-04';
  const profile = opts.profile || before.manifest?.profile || (before.kind === 'official-07' ? 'project' : 'project');
  if (!['project', 'managed'].includes(profile)) return { status: 'HOLD', kind: before.kind, written: 0, differences: ['profile'], reason: 'profile 必须是 project 或 managed' };
  if (legacy04 && profile !== 'project') return { status: 'HOLD', kind: before.kind, written: 0, differences: ['profile'], reason: 'official 0.4 直迁只允许 project profile' };
  if (legacy04 && !opts.upgrade) return { status: 'HOLD', kind: before.kind, written: 0, differences: ['official-04-upgrade'], reason: '官方 0.4 runtime 需要显式 upgrade' };
  const authorizationPlan = authorization.prepare(root, { operation: 'install', identityKind: before.kind });
  if (!authorizationPlan.ok) return { status: 'HOLD', kind: before.kind, written: 0, differences: [authorizationPlan.reason], reason: authorizationPlan.reason };
  const desiredPaths = runtimePaths(source), desiredDigest = io.digest(source, desiredPaths, desiredMode);
  const sourceChanged = before.kind === 'current' && (JSON.stringify([...before.manifest.managedPaths].sort()) !== JSON.stringify(desiredPaths)
    || before.manifest.packageDigest !== desiredDigest);
  const definitionChanged = before.kind === 'current' && before.manifest.hookDefinitionDigest !== definition(root).digest;
  if (before.kind === 'official-06' && !opts.upgrade) return { status: 'HOLD', kind: before.kind, written: 0, differences: ['official-06-upgrade'], reason: '官方 0.6 runtime 需要显式 upgrade' };
  if (before.kind === 'official-07' && !opts.upgrade) return { status: 'HOLD', kind: before.kind, written: 0, differences: ['official-07-upgrade'], reason: '官方 0.7 runtime 需要显式 upgrade' };
  const allowed = before.kind === 'fresh' || before.kind === 'current' || (['official-07', 'official-06', 'owned-05', 'official-04'].includes(before.kind) && opts.upgrade);
  if (!allowed) return { status: 'HOLD', kind: before.kind, written: 0, differences: before.differences,
    reason: before.kind === 'legacy-unowned' ? '无 ownership legacy：Core 全部写入口零写 HOLD，转独立一次性迁移包' : 'runtime identity 不允许增量混装' };
  if (sourceChanged && !opts.upgrade) return { status: 'HOLD', kind: before.kind, written: 0, differences: ['source-package-digest'], reason: 'source package 变化需要显式 upgrade' };
  const instruction = legacy04 ? instructionPlan04(root) : instructionPlan(root);
  if (!instruction.ok) return { status: 'HOLD', kind: before.kind, written: 0, differences: [instruction.reason], reason: instruction.reason };
  const managedApi = require('./managed.js'), needsGlobal = profile === 'managed' || before.manifest?.profile === 'managed';
  if (needsGlobal && !opts.globalRoot) return { status: 'HOLD', kind: before.kind, written: 0, differences: ['globalRoot'], reason: 'managed profile 迁移需要 globalRoot' };
  const hookPreflight = legacy04 ? prepareLegacy04Hooks(root, before) : profile === 'managed' ? removeHooks(root) : mergeHooks(root);
  if (!hookPreflight.ok) return { status: 'HOLD', kind: before.kind, written: 0, differences: [hookPreflight.reason], reason: hookPreflight.reason };
  const hookPreimage = before.kind === 'current' && validHookPreimage(before.manifest?.hookPreimage)
    ? before.manifest.hookPreimage : hookPreflight.hookPreimage || { fileExisted: true, addedEvents: [] };
  const registryPlan = opts.globalRoot ? managedApi.prepareProject(opts.globalRoot, root, profile === 'managed', desiredDigest) : null;
  if (registryPlan && !registryPlan.ok) return { status: 'HOLD', kind: before.kind, written: 0, differences: [registryPlan.reason], reason: registryPlan.reason };
  const archivePlan = legacy04 ? archive04(root) : before.kind === 'owned-05' ? archive05(root, before) : { ok: true, rows: [] };
  if (!archivePlan.ok) return { status: 'HOLD', kind: before.kind, written: 0, differences: [archivePlan.reason], reason: archivePlan.reason };
  if (!opts.write) return { status: 'READY', kind: before.kind, written: 0, differences: [], dryRun: true };
  const oldRuntimeAfterArchive = legacy04 ? legacy04RuntimeAfterArchive(before.runtimePreimage) : null;
  if (legacy04) {
    const rechecked = verifyOfficial04(root);
    if (!rechecked.ok || rechecked.kind !== 'official-04' || !sameRuntimeImage(path.join(root, RUNTIME), before.runtimePreimage)
      || !sameLegacy04State(before.lifecycle.statePreimage)) {
      return { status: 'HOLD', kind: before.kind, written: 0, differences: ['official-04-preimage'], reason: 'official 0.4 identity 或 lifecycle 在写入前发生变化' };
    }
  }
  let swapped = null, archive = archivePlan, instructionResult = { ok: true, rows: [] }, hooks = { ok: true, wrote: false }, registry = { ok: true, wrote: false }, authorizationResult = { ok: true, wrote: false, written: [] }, committed;
  try {
    if (legacy04) {
      archive = archive04(root, { write: true, fault: opts.fault, rows: archivePlan.rows }); if (!archive.ok) throw new Error(archive.reason);
      if (!sameRuntimeImage(path.join(root, RUNTIME), oldRuntimeAfterArchive)) throw new Error('official 0.4 runtime 在 swap 前发生变化');
    }
    if (before.kind !== 'current' || sourceChanged || definitionChanged || before.manifest?.profile !== profile) {
      swapped = swap(root, stage(root, source, opts.fault, profile, hookPreimage), before.kind !== 'fresh', opts.fault, legacy04 ? oldRuntimeAfterArchive : null);
    }
    if (!legacy04) { archive = before.kind === 'owned-05' ? archive05(root, before, { write: true, fault: opts.fault }) : archivePlan; if (!archive.ok) throw new Error(archive.reason); }
    instructionResult = mergeInstructions(root, { write: true, fault: opts.fault, plan: instruction }); if (!instructionResult.ok) throw new Error(instructionResult.reason);
    if (profile === 'project' && registryPlan) { registry = managedApi.applyProject(registryPlan, opts.fault); if (!registry.ok) throw new Error(registry.reason); }
    hooks = legacy04 ? writeLegacy04HookPlans(hookPreflight, opts.fault) : hookPreflight.plan ? writeHookPlan(hookPreflight.plan, opts.fault) : { ok: true, wrote: false };
    if (!hooks.ok) throw new Error(hooks.reason);
    if (profile === 'managed' && registryPlan) { registry = managedApi.applyProject(registryPlan, opts.fault); if (!registry.ok) throw new Error(registry.reason); }
    authorizationResult = authorization.apply(authorizationPlan, opts.fault); if (!authorizationResult.ok) throw new Error(authorizationResult.reason);
    // official-04 R1 already has these project-owned files; recreating a
    // concurrently removed plan/MISTAKES here would overwrite the frozen R1
    // byte baseline rather than rolling back fail-closed.
    if (!legacy04) skeleton(root, { write: true });
    if (legacy04 && !sameLegacy04State(before.lifecycle.statePreimage)) throw new Error('official 0.4 lifecycle 在提交前发生变化');
    committed = { status: 'INSTALLED', kind: before.kind, written: before.kind === 'current' && !sourceChanged && !definitionChanged ? 0 : 1,
      rollback: swapped?.rollback || null, archive, hooks, registry, authorization: authorizationResult, instructions: instructionResult.rows, manifest: io.json(path.join(root, MANIFEST)) };
  } catch (error) {
    const recovery = [];
    recovery.push(...authorization.rollback(authorizationResult));
    recovery.push(...managedApi.rollbackProject(registry));
    if (hooks.ok && hooks.wrote) {
      if (legacy04) recovery.push(...restoreLegacy04HookPlans(hookPreflight.plans));
      else try { if (hookPlanPostimage(hookPreflight.plan)) restoreImage(hookPreflight.plan.preimage); else recovery.push('hooks:concurrent-change'); } catch (restore) { recovery.push(`hooks:${restore.message}`); }
    }
    recovery.push(...restoreInstructionRows(instructionResult.ok ? instructionResult.rows : []));
    if (legacy04) { recovery.push(...rollbackSwap(root, swapped)); recovery.push(...reverseArchive04(archive.rows)); }
    else { recovery.push(...reverseArchive(archive.rows)); recovery.push(...rollbackSwap(root, swapped)); }
    return { status: 'PARTIAL', kind: before.kind, written: 0, differences: [error.message, ...recovery], reason: error.message,
      rollback: swapped?.rollback || null, archive, hooks, registry, instructions: instructionResult.rows };
  }
  return committed;
}
function configured(root) {
  const file = path.join(hookConfigRoot(root), '.codex', 'hooks.json'), data = io.json(file), expected = definition(root);
  if (!data?.hooks) return false;
  return EVENTS.every((event) => (data.hooks[event] || []).some((group) => JSON.stringify(canonical(group)) === JSON.stringify(canonical(expected.config.hooks[event][0]))));
}
function hostDefinition(root, profile, globalRoot) {
  if (profile !== 'managed') {
    const value = definition(root); return { ...value, source: 'project', sourcePath: path.join(hookConfigRoot(root), '.codex', 'hooks.json') };
  }
  if (!globalRoot) return null;
  const managedApi = require('./managed.js'), target = managedApi.layout(globalRoot), config = managedApi.hooksDefinition(target.gateway, VERSION);
  return { config, command: target.gateway, digest: `sha256:${io.sha(JSON.stringify(canonical(config.hooks)))}`, source: 'system', sourcePath: target.definition };
}
const normalizeHostId = (value) => String(value || '-').replace(/[^A-Za-z0-9._-]/g, '_').slice(0, 120);
function trustedHostSnapshot(snapshot, profile, expected) {
  return snapshot?.definitionDigest === expected.digest && snapshot.profile === profile && !(snapshot.warnings || []).length && !(snapshot.errors || []).length
    && EVENTS.every((event) => {
      const item = snapshot.events?.[event]; return item?.matched === true && item.enabled === true && !!item.currentHash
        && (profile === 'managed' ? item.isManaged === true && item.trustStatus === 'managed' : item.isManaged !== true && item.trustStatus === 'trusted');
    });
}
function trustedHostLedger(host, profile, expected, root) {
  const before = host.before, after = host.after, fingerprint = `sha256:${io.sha(fs.realpathSync(root))}`;
  return !!expected && host.source === 'codex-hooks-list' && typeof host.connectionId === 'string' && !!host.connectionId
    && host.cwdFingerprint === fingerprint && trustedHostSnapshot(before, profile, expected) && trustedHostSnapshot(after, profile, expected) && EVENTS.every((event) => {
      const left = before.events?.[event], right = after.events?.[event];
      return left?.matched === true && right?.matched === true && left.enabled === true && right.enabled === true
        && !!left.currentHash && left.currentHash === right.currentHash
        && (profile === 'managed' ? left.isManaged === true && right.isManaged === true && left.trustStatus === 'managed' && right.trustStatus === 'managed'
          : left.isManaged !== true && right.isManaged !== true && left.trustStatus === 'trusted' && right.trustStatus === 'trusted');
    });
}
function eventJoin(hostEvents, host, event, expected, packageDigest) {
  const currentHash = host.after?.events?.[event]?.currentHash, lower = Date.parse(host.before?.at || ''), upper = Date.parse(host.after?.at || '');
  const records = (hostEvents.records || []).filter((row) => row.event === event && Number.isFinite(lower) && Number.isFinite(upper)
    && Date.parse(row.at) >= lower && Date.parse(row.at) <= upper
    && row.definitionDigest === expected.digest && row.packageDigest === packageDigest && row.handlerHash === currentHash
    && row.connectionId === host.connectionId && typeof row.runId === 'string' && row.runId);
  const completed = records.filter((row) => row.phase === 'completed').sort((a, b) => String(a.at).localeCompare(String(b.at)));
  if (!completed.length) return { observed: false, verdict: 'missing' };
  const runId = completed.at(-1).runId, starts = records.filter((row) => row.runId === runId && row.phase === 'started'), finishes = records.filter((row) => row.runId === runId && row.phase === 'completed');
  if (starts.length > 1 || finishes.length > 1 || finishes.some((row) => row.receiptVerdict === 'ambiguous')) return { observed: false, verdict: 'ambiguous', runId };
  const exact = starts.length === 1 && finishes.length === 1 && finishes[0].status === 'completed' && finishes[0].receiptVerdict === 'exact'
    && starts[0].thread === finishes[0].thread && starts[0].turn === finishes[0].turn;
  return { observed: exact, verdict: exact ? 'exact' : 'missing', runId };
}
function health(root, opts = {}) {
  const checked = verify06(root), value = checked.manifest || {}, def = definition(root);
  const profile = value.profile || 'project', expected = hostDefinition(root, profile, opts.globalRoot), managedApi = profile === 'managed' ? require('./managed.js') : null;
  const host = io.json(path.join(root, 'docs', 'harness', 'usage', 'host-health.json'), {}), hostEvents = io.json(path.join(root, 'docs', 'harness', 'usage', 'host-events.json'), { events: {} });
  const configuredValue = profile === 'project' ? configured(root) : !!expected && checked.ok && !configured(root)
    && managedApi.verifyGlobal(opts.globalRoot).ok && managedApi.projectState(opts.globalRoot, root).active;
  const trusted = checked.ok && !!expected && trustedHostLedger(host, profile, expected, root), joins = Object.fromEntries(EVENTS.map((event) => [event,
    trusted && hostEvents.source === 'codex-app-server-hooks' && hostEvents.connectionId === host.connectionId
      ? eventJoin(hostEvents, host, event, expected, value.packageDigest) : { observed: false, verdict: 'missing' }]));
  const seen = Object.fromEntries(EVENTS.map((event) => [event, joins[event].observed]));
  const native = nativeState(root, value.packageDigest);
  const nativeConfigured = checked.ok && native.configured, nativeProbed = nativeConfigured && native.probed;
  return { profile, installed: checked.ok, configured: configuredValue, trusted, policyTrusted: profile === 'managed' && trusted, observed: seen, evidenceJoin: Object.fromEntries(EVENTS.map((event) => [event, joins[event].verdict])),
    effective: checked.ok && configuredValue && trusted && EVENTS.every((event) => seen[event]) && nativeConfigured && nativeProbed,
    nativePrePush: { configured: nativeConfigured, probed: nativeProbed } };
}
const HOST_EVENT = { SessionStart: 'sessionStart', UserPromptSubmit: 'userPromptSubmit', Stop: 'stop', PreToolUse: 'preToolUse' };
function recordHostList(root, response, opts = {}) {
  const checked = verify06(root), profile = checked.manifest?.profile || 'project';
  if (opts.write && !checked.ok) return { ok: false, status: 'HOLD', reason: 'runtime identity 不允许写宿主健康账' };
  const entry = response?.data?.find((x) => samePath(x.cwd, root)) || { hooks: [], warnings: [], errors: [] };
  const expected = hostDefinition(root, profile, opts.globalRoot), events = {};
  if (!expected) return { ok: false, status: 'HOLD', reason: 'managed host list 需要 globalRoot', events: {} };
  for (const event of EVENTS) {
    const matcher = expected.config.hooks[event][0].matcher || null, matches = (entry.hooks || []).filter((item) => item.eventName === HOST_EVENT[event]
      && item.command === expected.command && (item.matcher || null) === matcher
      && (profile === 'managed' ? item.isManaged === true && ['system', 'managed'].includes(item.source) : item.source === 'project')
      && samePath(item.sourcePath, expected.sourcePath)), found = matches.length === 1 ? matches[0] : null;
    events[event] = found ? { matched: true, ambiguous: false, enabled: found.enabled === true, trustStatus: found.trustStatus,
      isManaged: found.isManaged === true, currentHash: found.currentHash || null, sourcePath: found.sourcePath }
      : { matched: false, ambiguous: matches.length > 1, enabled: false, isManaged: false, trustStatus: null, currentHash: null };
  }
  const file = path.join(root, 'docs', 'harness', 'usage', 'host-health.json'), prior = io.json(file, {}), connectionId = typeof opts.connectionId === 'string' && opts.connectionId ? opts.connectionId : null;
  const snapshot = { profile, at: new Date().toISOString(), definitionDigest: expected.digest, warnings: entry.warnings || [], errors: entry.errors || [], events };
  const before = prior.source === 'codex-hooks-list' && prior.connectionId === connectionId ? prior.after : null;
  const value = { source: 'codex-hooks-list', profile, at: snapshot.at, connectionId, cwdFingerprint: `sha256:${io.sha(fs.realpathSync(root))}`,
    definitionDigest: expected.digest, warnings: snapshot.warnings, errors: snapshot.errors, events, before, after: snapshot };
  if (opts.write) io.atomic(file, `${JSON.stringify(value, null, 2)}\n`);
  return value;
}
function recordHostEvent(root, notification, opts = {}) {
  const run = notification?.run || {}, event = Object.entries(HOST_EVENT).find(([, value]) => value === run.eventName)?.[0];
  const checked = verify06(root), profile = checked.manifest?.profile || 'project', expected = hostDefinition(root, profile, opts.globalRoot), def = expected || definition(root), host = io.json(path.join(root, 'docs', 'harness', 'usage', 'host-health.json'), {});
  if (!checked.ok) return { ok: false, reason: 'runtime identity 不允许写宿主事件账' };
  if (!expected) return { ok: false, reason: 'managed host event 需要 globalRoot' };
  const listed = host.after?.events?.[event], phase = notification?.method === 'hook/started' ? 'started' : notification?.method === 'hook/completed' ? 'completed' : null;
  const valid = !!event && !!phase && typeof run.id === 'string' && !!run.id && run.handlerType === 'command'
    && (profile === 'managed' ? run.isManaged === true && ['system', 'managed'].includes(run.source) : run.source === 'project')
    && samePath(run.sourcePath || '/', expected.sourcePath) && (phase === 'started' || run.status === 'completed')
    && trustedHostSnapshot(host.after, profile, expected) && listed?.currentHash && opts.connectionId === host.connectionId;
  if (!valid) return { ok: false, reason: `${notification?.method || 'unknown'} 与 Harness definition/connection 不匹配` };
  const file = path.join(root, 'docs', 'harness', 'usage', 'host-events.json');
  const value = io.json(file, { source: 'codex-app-server-hooks', connectionId: host.connectionId, records: [] });
  if (value.connectionId !== host.connectionId) { value.records = []; value.connectionId = host.connectionId; }
  value.source = 'codex-app-server-hooks';
  const row = { at: new Date().toISOString(), phase, runId: run.id, event, handlerHash: listed.currentHash, status: phase === 'completed' ? run.status : null,
    thread: notification.threadId || null, turn: notification.turnId || null, connectionId: host.connectionId,
    definitionDigest: def.digest, packageDigest: checked.manifest?.packageDigest || null };
  let receiptRows = [];
  if (profile === 'managed') {
      const global = require('./managed.js'), target = global.layout(opts.globalRoot), manifest = global.verifyGlobal(opts.globalRoot).manifest;
      receiptRows = io.read(target.receipts, '').split('\n').filter(Boolean).flatMap((line) => { try { return [JSON.parse(line)]; } catch { return []; } });
      row.expectedGeneration = manifest?.generation ?? null;
  } else receiptRows = io.read(path.join(root, 'docs', 'harness', 'usage', '.observed.jsonl'), '').split('\n').filter(Boolean).flatMap((line) => { try { return [JSON.parse(line)]; } catch { return []; } });
  if (phase === 'started') row.receiptCursor = receiptRows.length;
  if (phase === 'completed') {
    const starts = (value.records || []).filter((item) => item.phase === 'started' && item.runId === run.id && item.event === event
      && item.connectionId === host.connectionId), start = starts.length === 1 ? starts[0] : null;
    const candidates = start && Number.isInteger(start.receiptCursor) ? receiptRows.slice(start.receiptCursor) : [];
    const receiptMatches = profile === 'managed' ? candidates.filter((item) => item.event === event && item.session === (notification.threadId || '-')
      && (item.turn || null) === (notification.turnId || null) && item.decision === 'executed' && item.generation === start.expectedGeneration
      && item.digestPrefix === checked.manifest.packageDigest.slice(7, 19)) : candidates.filter((item) => item.event === event
        && item.session === normalizeHostId(notification.threadId) && (item.turn || null) === (notification.turnId ? normalizeHostId(notification.turnId) : null)
        && item.decision === 'executed' && item.definitionDigest === def.digest && item.packageDigest === checked.manifest.packageDigest);
    row.receiptVerdict = starts.length > 1 ? 'ambiguous' : receiptMatches.length === 1 ? 'exact' : receiptMatches.length ? 'ambiguous' : 'missing';
    if (receiptMatches.length === 1) row.receipt = { at: receiptMatches[0].at || null, decision: 'executed',
      generation: Number.isInteger(receiptMatches[0].generation) ? receiptMatches[0].generation : null, digestPrefix: receiptMatches[0].digestPrefix || null };
  }
  value.records = [...(Array.isArray(value.records) ? value.records : []), row];
  if (opts.write) io.atomic(file, `${JSON.stringify(value, null, 2)}\n`);
  return { ok: phase === 'started' || row.receiptVerdict === 'exact', event, phase, receiptVerdict: row.receiptVerdict || null, value,
    ...(phase === 'completed' && row.receiptVerdict !== 'exact' ? { reason: `runtime receipt ${row.receiptVerdict}` } : {}) };
}
function removeHooks(root, opts = {}) {
  const identity = inspect(root), file = path.join(hookConfigRoot(root), '.codex', 'hooks.json'), present = fs.existsSync(file), data = io.json(file), expected = definition(root);
  const hookPreimage = validHookPreimage(identity.manifest?.hookPreimage) ? identity.manifest.hookPreimage
    : { fileExisted: present, addedEvents: EVENTS.filter((event) => !Object.hasOwn(data?.hooks || {}, event)) };
  if (!present) return { ok: true, wrote: false, hookPreimage };
  if (!data || !data.hooks || Array.isArray(data.hooks)) return { ok: false, wrote: false, reason: 'invalid .codex/hooks.json' };
  const malformed = Object.entries(data.hooks).filter(([, groups]) => !Array.isArray(groups) || groups.some((group) => !group || !Array.isArray(group.hooks)));
  if (malformed.length) return { ok: false, wrote: false, reason: `invalid hook groups: ${malformed.map(([event]) => event).join(',')}` };
  const previous = identity.kind === 'current' && isExactPrevious06(data, expected)
    && [expected.digest, previous06Digest(expected)].includes(identity.manifest?.hookDefinitionDigest);
  for (const event of EVENTS) {
    const groups = data.hooks[event] || [];
    if (groups.some((group) => ['legacy', 'unknown'].includes(hookGroupKind(event, group, expected.config.hooks[event][0], previous, expected)))) return { ok: false, wrote: false, reason: `unknown Harness hook: ${event}` };
  }
  const before = fs.readFileSync(file, 'utf8');
  let text; try { text = transformHookDocument(before, expected,
    { append: false, previous, legacyRoot: null, pruneEvents: new Set(hookPreimage.addedEvents), removeKinds: new Set(['current', 'previous']) }); }
  catch (error) { return { ok: false, wrote: false, reason: error.message }; }
  if (!hookPreimage.fileExisted && before === freshHookDocument(expected)) text = null;
  const preimage = fileImage(file);
  if (preimage.type !== 'file' || preimage.parent.type !== 'directory') return { ok: false, wrote: false, reason: 'Hook 配置路径不是安全普通文件' };
  const plan = { file, before, text, mode: preimage.mode, digest: expected.digest, preimage };
  if (opts.write) return writeHookPlan(plan, opts.fault);
  return { ok: true, wrote: false, hookPreimage, plan };
}
function uninstall(root, opts = {}) {
  const checked = verify06(root); if (!checked.ok) return { status: 'HOLD', differences: checked.differences, removed: false };
  const authorizationPlan = authorization.prepare(root, { operation: 'uninstall', identityKind: 'current' });
  if (!authorizationPlan.ok) return { status: 'HOLD', differences: [authorizationPlan.reason], reason: authorizationPlan.reason, removed: false };
  if (Object.keys(checked.manifest.extensions || {}).length) return { status: 'HOLD', differences: ['extensions-installed'], removed: false };
  const managedApi = require('./managed.js');
  if (checked.manifest.profile === 'managed' && !opts.globalRoot) return { status: 'HOLD', differences: ['globalRoot'], reason: 'managed uninstall 需要 globalRoot', removed: false };
  const registryPlan = opts.globalRoot ? managedApi.prepareProject(opts.globalRoot, root, false, checked.manifest.packageDigest) : null;
  if (registryPlan && !registryPlan.ok) return { status: 'HOLD', differences: [registryPlan.reason], reason: registryPlan.reason, removed: false };
  const hookPlan = removeHooks(root), nativePlan = removeNative(root);
  if (!hookPlan.ok || !nativePlan.ok) return { status: 'HOLD', differences: [hookPlan.reason || nativePlan.reason], removed: false };
  if (!opts.write) return { status: 'READY', removed: false };
  const stable = path.join(root, RUNTIME), tomb = path.join(root, '.claude', `.harness-lite.uninstall-${Date.now()}`);
  let hooks, native, registry = { ok: true, wrote: false };
  try {
    fs.renameSync(stable, tomb); opts.fault?.('uninstall:tomb');
    if (registryPlan) { registry = managedApi.applyProject(registryPlan, opts.fault); if (!registry.ok) throw new Error(registry.reason); }
    hooks = hookPlan.plan ? writeHookPlan(hookPlan.plan, opts.fault) : { ok: true, wrote: false };
    if (!hooks.ok) throw new Error(hooks.reason);
    native = nativePlan.plan ? writeNativeRemovalPlan(nativePlan.plan, opts.fault) : { ok: true, wrote: false };
    if (!native.ok) throw new Error(native.reason);
    fs.rmSync(tomb, { recursive: true }); return { status: 'UNINSTALLED', removed: true, hooks, native, registry };
  } catch (error) {
    const recovery = [];
    recovery.push(...managedApi.rollbackProject(registry));
    try { if (hooks?.ok && hooks.wrote) restoreImage(hookPlan.plan.preimage); } catch (restore) { recovery.push(`hooks:${restore.message}`); }
    try { if (native?.ok && native.wrote) for (const image of [nativePlan.plan.backup, nativePlan.plan.hook, nativePlan.plan.marker]) restoreImage(image); } catch (restore) { recovery.push(`native:${restore.message}`); }
    try { if (!fs.existsSync(stable) && fs.existsSync(tomb)) fs.renameSync(tomb, stable); } catch (restore) { recovery.push(`runtime:${restore.message}`); }
    return { status: 'PARTIAL', removed: false, differences: [error.message, ...recovery], reason: error.message, hooks, native, registry };
  }
}
function loadExtension(dir) {
  const data = io.json(path.join(dir, 'manifest.json'));
  if (data?.version !== 1 || !/^[a-z0-9][a-z0-9-]*$/.test(data.id || '') || !data.packageVersion || !data.files || Array.isArray(data.files)) throw new Error('扩展 manifest 不合规');
  const base = fs.realpathSync(dir), prefix = `${RUNTIME}/extensions/${data.id}/`, files = [];
  for (const [target, source] of Object.entries(data.files)) {
    const cleanTarget = target.replaceAll('\\', '/'); if (!cleanTarget.startsWith(prefix)) throw new Error(`扩展目标必须位于 ${prefix}`);
    const abs = fs.realpathSync(io.safe(dir, source)), stat = fs.lstatSync(abs);
    if (!abs.startsWith(`${base}${path.sep}`) || !stat.isFile() || stat.isSymbolicLink()) throw new Error(`扩展源文件不合规：${source}`);
    files.push({ target: cleanTarget.slice(RUNTIME.length + 1), source: abs });
  }
  return { ...data, files: files.sort((a, b) => a.target.localeCompare(b.target)) };
}
function verifyExtension(root, id) {
  const core = verify06(root), spec = core.manifest?.extensions?.[id]; if (!core.ok || !spec) return { ok: false, differences: ['extension-manifest'] };
  const runtime = path.join(root, RUNTIME), actual = io.list(path.join(runtime, 'extensions', id), true).map((x) => path.relative(runtime, x).replaceAll('\\', '/')).sort(), differences = [];
  for (const rel of spec.managedPaths) { const type = io.inspect(runtime, rel).type; if (type !== 'file') differences.push(`${type}:${rel}`); }
  for (const rel of actual.filter((x) => !spec.managedPaths.includes(x))) differences.push(`extra:${rel}`);
  let digest; try { digest = io.digest(runtime, spec.managedPaths); } catch (error) { differences.push(`identity:${error.message}`); }
  if (digest !== spec.packageDigest) differences.push('packageDigest'); return { ok: differences.length === 0, differences, spec };
}
function installExtension(root, dir, opts = {}) {
  root = path.resolve(root); const core = verify06(root); if (!core.ok) return { status: 'HOLD', differences: core.differences };
  const ext = loadExtension(path.resolve(dir)), prior = core.manifest.extensions?.[ext.id], priorCheck = prior ? verifyExtension(root, ext.id) : null;
  if (prior && !priorCheck.ok) return { status: 'HOLD', differences: priorCheck.differences };
  const parent = path.join(root, '.claude'), staged = path.join(parent, `.harness-lite.ext-stage-${process.pid}-${Date.now()}`), managedPaths = ext.files.map((x) => x.target);
  for (const item of ext.files) { const dest = path.join(staged, item.target); fs.mkdirSync(path.dirname(dest), { recursive: true }); fs.copyFileSync(item.source, dest); fs.chmodSync(dest, 0o644); }
  const packageDigest = io.digest(staged, managedPaths), spec = { packageVersion: ext.packageVersion, managedPaths, packageDigest };
  if (prior && JSON.stringify(canonical(prior)) === JSON.stringify(canonical(spec))) { fs.rmSync(staged, { recursive: true }); return { status: opts.write ? 'INSTALLED' : 'READY', id: ext.id, written: 0 }; }
  if (prior && !opts.upgrade) { fs.rmSync(staged, { recursive: true }); return { status: 'HOLD', differences: ['extension-upgrade-required'] }; }
  if (!opts.write) { fs.rmSync(staged, { recursive: true }); return { status: 'READY', id: ext.id, written: 0 }; }
  const stable = path.join(root, RUNTIME, 'extensions', ext.id), rollback = `${stable}.rollback-${Date.now()}`;
  const ledger = path.join(root, MANIFEST), ledgerBefore = fs.readFileSync(ledger, 'utf8'); let old = false, fresh = false, ledgerChanged = false;
  try {
    fs.mkdirSync(path.dirname(stable), { recursive: true }); if (fs.existsSync(stable)) { fs.renameSync(stable, rollback); old = true; }
    fs.renameSync(path.join(staged, 'extensions', ext.id), stable); fresh = true; fs.rmSync(staged, { recursive: true });
    const value = io.json(ledger); value.extensions = { ...(value.extensions || {}), [ext.id]: spec };
    io.atomic(ledger, `${JSON.stringify(value, null, 2)}\n`, 0o600); ledgerChanged = true; if (!verifyExtension(root, ext.id).ok) throw new Error('extension verify failed');
    return { status: 'INSTALLED', id: ext.id, written: 1, rollback: old ? rollback : null };
  } catch (error) { if (ledgerChanged) io.atomic(ledger, ledgerBefore, 0o600); if (fresh) fs.rmSync(stable, { recursive: true, force: true }); if (old && fs.existsSync(rollback)) fs.renameSync(rollback, stable); throw error; }
}
function uninstallExtension(root, id, opts = {}) {
  const checked = verifyExtension(root, id); if (!checked.ok) return { status: 'HOLD', differences: checked.differences };
  if (!opts.write) return { status: 'READY', removed: false };
  const stable = path.join(root, RUNTIME, 'extensions', id), tomb = `${stable}.uninstall-${Date.now()}`, ledger = path.join(root, MANIFEST), before = fs.readFileSync(ledger, 'utf8'); fs.renameSync(stable, tomb);
  try { const value = io.json(ledger); delete value.extensions[id]; io.atomic(ledger, `${JSON.stringify(value, null, 2)}\n`, 0o600); fs.rmSync(tomb, { recursive: true }); return { status: 'UNINSTALLED', removed: true }; }
  catch (error) { io.atomic(ledger, before, 0o600); if (!fs.existsSync(stable) && fs.existsSync(tomb)) fs.renameSync(tomb, stable); throw error; }
}
function installNative(root, opts = {}) {
  const checked = verify06(root); if (!checked.ok) return { ok: false, reason: `runtime 未通过 identity：${checked.differences.join(',')}` };
  const common = (gitRun(['rev-parse', '--git-common-dir'], { cwd: root, encoding: 'utf8' }).stdout || '').trim();
  if (!common) return { ok: false, reason: 'not a git repository' };
  const dispatcherDigest = `sha256:${io.sha(fs.readFileSync(path.join(root, RUNTIME, 'hooks', 'dispatcher.js')))}`;
  const dir = path.resolve(root, common, 'hooks'), file = path.join(dir, 'pre-push'), backup = `${file}.harness-lite.foreign`, script = nativeScript(backup, dispatcherDigest);
  const markerFile = path.join(root, '.codex', 'harness-lite', 'native.json'), hookImage = fileImage(file), backupImage = fileImage(backup), markerImage = fileImage(markerFile);
  if (![hookImage, backupImage, markerImage].every((image) => ['file', 'missing'].includes(image.type) && image.parent.type === 'directory')) return { ok: false, reason: 'native 安装 surface 不是安全普通文件', file, backup };
  let marker = null;
  if (markerImage.type === 'file') { try { marker = JSON.parse(markerImage.body.toString('utf8')); } catch { return { ok: false, reason: 'native marker JSON 损坏', file, backup }; } }
  const looksHarness = hookImage.type === 'file' && hookImage.body.includes('dispatcher="$root/.claude/harness-lite/hooks/dispatcher.js"')
    && hookImage.body.includes('exec node "$dispatcher" --native-pre-push');
  const priorOwned = !!marker?.configured && /^sha256:[0-9a-f]{64}$/.test(marker.dispatcherDigest || '') && marker.commonDir === path.resolve(root, common)
    && hookImage.type === 'file' && marker.hookDigest === `sha256:${io.sha(hookImage.body)}`
    && /^sha256:[0-9a-f]{64}$/.test(marker.packageDigest || '') && looksHarness;
  if (marker && !priorOwned) return { ok: false, reason: 'native marker 或 common pre-push identity 漂移', file, backup };
  if (!marker && (backupImage.type === 'file' || looksHarness)) return { ok: false, reason: '发现无有效 marker 的 Harness/backup 现场，保持零写 HOLD', file, backup };
  const current = nativeState(root, checked.manifest.packageDigest);
  if (current.configured) return { ok: true, configured: true, wrote: false, file, backup, dispatcherDigest };
  const markerText = `${JSON.stringify({ configured: true, probed: false, hookDigest: `sha256:${io.sha(script)}`,
    dispatcherDigest, packageDigest: checked.manifest.packageDigest, commonDir: path.resolve(root, common) }, null, 2)}\n`;
  if (!opts.write) return { ok: true, configured: false, wrote: false, file, backup, dispatcherDigest };
  const images = [hookImage, backupImage, markerImage];
  try {
    if (!images.every(sameImage)) throw new Error('native 安装 surface 在 preflight 后发生变化'); opts.fault?.('native-install:before');
    if (!images.every(sameImage)) throw new Error('native 安装 surface 在写入前发生变化');
    fs.mkdirSync(dir, { recursive: true });
    if (!priorOwned && hookImage.type === 'file') io.atomic(backup, hookImage.body, hookImage.mode); opts.fault?.('native-install:backup');
    io.atomic(file, script, 0o755); opts.fault?.('native-install:hook'); io.atomic(markerFile, markerText, 0o600); opts.fault?.('native-install:marker');
    if (!nativeState(root, checked.manifest.packageDigest).configured) throw new Error('native 安装后 identity 校验失败');
    return { ok: true, configured: true, wrote: true, file, backup, dispatcherDigest };
  } catch (error) {
    const recovery = [];
    for (const image of [backupImage, hookImage, markerImage]) try { restoreImage(image); } catch (restore) { recovery.push(restore.message); }
    return { ok: false, configured: false, wrote: false, reason: recovery.length ? `${error.message}；native preimage 恢复失败：${recovery.join('|')}` : error.message, file, backup };
  }
}
function probeNative(root, opts = {}) {
  const checked = verify06(root), native = nativeState(root, checked.manifest?.packageDigest);
  if (!checked.ok) return { ok: false, status: 'HOLD', probed: false, reason: `runtime 未通过 identity：${checked.differences.join(',')}` };
  if (!native.configured) return { ok: false, probed: false, reason: 'native pre-push 未配置或 identity 漂移' };
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'hl-native-probe-')), remote = path.join(dir, 'remote.git');
  try {
    const init = gitRun(['init', '--bare', '-q', remote], { encoding: 'utf8' });
    const run = init.status === 0 ? gitRun(['push', '--dry-run', remote, 'HEAD:refs/heads/harness-lite-probe'], { cwd: root, encoding: 'utf8' }) : init;
    const probed = run.status !== 0 && /Harness pre-push/.test(`${run.stdout || ''}${run.stderr || ''}`);
    if (opts.write) { const value = io.json(path.join(root, '.codex', 'harness-lite', 'native.json'), {}); io.atomic(path.join(root, '.codex', 'harness-lite', 'native.json'), `${JSON.stringify({ ...value, probed,
      probedHookDigest: native.actualDigest, probedAt: new Date().toISOString(), probe: 'disposable-local-bare-remote' }, null, 2)}\n`); }
    return { ok: probed, probed, remote: 'disposable-local-bare-remote', reason: probed ? null : '未观察到 native pre-push' };
  } finally { fs.rmSync(dir, { recursive: true, force: true }); }
}

function nativeScript(backup, dispatcherDigest) {
  const expected = String(dispatcherDigest || '').replace(/^sha256:/, '');
  return `#!/bin/sh\nroot=$(/usr/bin/git rev-parse --show-toplevel 2>/dev/null) || exit 0\ndispatcher="$root/${RUNTIME}/hooks/dispatcher.js"\nmarker="$root/.codex/harness-lite/native.json"\nactual=$(/usr/bin/shasum -a 256 "$dispatcher" 2>/dev/null | /usr/bin/awk '{print $1}')\nif [ "$actual" = ${JSON.stringify(expected)} ]; then\n  if [ ! -f "$marker" ]; then echo "Harness pre-push：native marker 缺失，fail closed" >&2; exit 1; fi\n  exec ${JSON.stringify(path.resolve(process.execPath))} "$dispatcher" --native-pre-push "$@" --foreign ${JSON.stringify(backup)}\nfi\nif [ -f "$marker" ]; then echo "Harness pre-push：runtime identity 损坏，fail closed" >&2; exit 1; fi\nif [ -x ${JSON.stringify(backup)} ]; then exec ${JSON.stringify(backup)} "$@"; fi\nexit 0\n`;
}
function nativeState(root, packageDigest) {
  const value = io.json(path.join(root, '.codex', 'harness-lite', 'native.json'), {}), common = (gitRun(['rev-parse', '--git-common-dir'], { cwd: root, encoding: 'utf8' }).stdout || '').trim();
  const file = common ? path.join(path.resolve(root, common, 'hooks'), 'pre-push') : null, backup = file ? `${file}.harness-lite.foreign` : null;
  const actualDigest = file && fs.existsSync(file) ? `sha256:${io.sha(fs.readFileSync(file))}` : null, expectedDigest = backup ? `sha256:${io.sha(nativeScript(backup, value.dispatcherDigest))}` : null;
  const configured = /^sha256:[0-9a-f]{64}$/.test(packageDigest || '') && value.configured === true && value.commonDir === (common ? path.resolve(root, common) : null) && value.hookDigest === actualDigest
    && actualDigest === expectedDigest && value.packageDigest === packageDigest;
  return { ...value, file, backup, actualDigest, configured, probed: configured && value.probed === true && value.probedHookDigest === actualDigest };
}
function writeNativeRemovalPlan(plan, fault = () => {}) {
  const images = [plan.marker, plan.hook, plan.backup];
  if (!images.every(sameImage)) return { ok: false, status: 'HOLD', wrote: false, reason: 'native pre-push 或 marker 在 preflight 后发生变化' };
  try { fault('native:before'); } catch (error) { return { ok: false, status: 'HOLD', wrote: false, reason: error.message }; }
  if (!images.every(sameImage)) return { ok: false, status: 'HOLD', wrote: false, reason: 'native pre-push 或 marker 在写入前发生变化' };
  try {
    if (plan.backup.type === 'file') io.atomic(plan.hook.file, plan.backup.body, plan.backup.mode); else fs.unlinkSync(plan.hook.file);
    fault('native:hook'); fs.rmSync(plan.marker.file, { force: true }); fault('native:marker');
    if (plan.backup.type === 'file') fs.rmSync(plan.backup.file, { force: true }); fault('native:backup');
    return { ok: true, wrote: true };
  } catch (error) {
    const recovery = [];
    for (const image of [plan.backup, plan.hook, plan.marker]) try { restoreImage(image); } catch (restore) { recovery.push(restore.message); }
    return { ok: false, status: 'HOLD', wrote: false, reason: recovery.length ? `${error.message}；native preimage 恢复失败：${recovery.join('|')}` : error.message };
  }
}
function removeNative(root, opts = {}) {
  const markerFile = path.join(root, '.codex', 'harness-lite', 'native.json'), marker = fileImage(markerFile);
  if (!['missing', 'file'].includes(marker.type) || marker.parent.type !== 'directory') return { ok: false, wrote: false, reason: 'native marker 不是安全普通文件' };
  const common = (gitRun(['rev-parse', '--git-common-dir'], { cwd: root, encoding: 'utf8' }).stdout || '').trim();
  const hookFile = common ? path.join(path.resolve(root, common, 'hooks'), 'pre-push') : null;
  const physical = hookFile ? fileImage(hookFile) : null;
  const harnessWrapper = physical?.type === 'file' && physical.body.includes('--native-pre-push');
  if (marker.type === 'missing') return harnessWrapper
    ? { ok: false, wrote: false, reason: 'native marker 缺失但 common pre-push 仍是 Harness wrapper' }
    : { ok: true, wrote: false, plan: null };
  let value; try { value = JSON.parse(marker.body.toString('utf8')); } catch { return { ok: false, wrote: false, reason: 'native marker JSON 损坏' }; }
  if (!common || !physical || physical.type !== 'file' || physical.parent.type !== 'directory') return { ok: false, wrote: false, reason: 'native pre-push 不是安全普通文件' };
  const state = nativeState(root, value.packageDigest); if (!state.configured) return { ok: false, wrote: false, reason: 'native pre-push identity 漂移' };
  const backup = fileImage(state.backup);
  if (!['missing', 'file'].includes(backup.type) || backup.parent.type !== 'directory') return { ok: false, wrote: false, reason: 'native backup 不是安全普通文件' };
  const plan = { marker, hook: physical, backup };
  if (opts.write) return writeNativeRemovalPlan(plan, opts.fault);
  return { ok: true, wrote: false, plan };
}

module.exports = { VERSION, SOURCE_COMMIT, RUNTIME, MANIFEST, EVENTS, INSTRUCTION, PREVIOUS_07_INSTRUCTION, PREVIOUS_06_INSTRUCTION, OFFICIAL_04, OFFICIAL_06, OFFICIAL_07, runtimePaths, hookConfigRoot,
  definition, official07Definition, legacy04Definition, manifest, verify06, verifyOfficial04, verifyOfficial07, verifyOfficial06, verifyOwned05, legacyUnowned, inspect, settleOfficial04, surfacePreflight, instructionPlan,
  install, configured, health, recordHostList, recordHostEvent, uninstall, loadExtension, verifyExtension,
  installExtension, uninstallExtension, nativeScript, nativeState, installNative, probeNative };
