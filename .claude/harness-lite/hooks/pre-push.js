#!/usr/bin/env node
'use strict';
// 文件名为兼容旧安装保留；现在是所有 PreToolUse 共用的轻量硬门适配器。
const hookio = require('../lib/hookio.js');
const gate = require('../lib/gate.js');

// 带引号的内容先抽掉，不然 git commit -m "push 之前跑一遍" 会被误当成 push
const unquote = (s) => String(s).replace(/'[^']*'/g, "''").replace(/"[^"]*"/g, '""');

// 按 shell 分隔符切段：`npm test && git push` 里那个 push 也得看见
const parts = (s) => unquote(s).split(/&&|\|\||[;&|\n]+/);

function isPush(cmd) {
  return parts(cmd || '').some((p) => /(^|\s)git(\s|$)/.test(p) && /\bpush\b/.test(p));
}

function decide(input, root, surface, opts) {
  const q = gate.classify(input, gate.policy(root || process.cwd()));
  if (q.category === 'ordinary') return null;
  const r = gate.evaluate(root || process.cwd(), q, q.operation === 'close-stage' ? { ...opts, write: false } : opts);
  // Codex 当前会解析但不执行 ask，所以未授权必须 deny；Claude 可用 ask 停下来确认。
  const decision = r.decision === 'allow' ? 'allow' : surface === 'codex' ? 'deny' : 'ask';
  return {
    hookSpecificOutput: {
      hookEventName: 'PreToolUse',
      permissionDecision: decision,
      permissionDecisionReason: `${q.category}/${q.operation}：${r.reason}`,
    },
  };
}

function main(argv) {
  const input = hookio.readInput();
  const args = argv || [];
  const i = args.indexOf('--surface');
  const surface = i !== -1 && args[i + 1] ? args[i + 1] : 'claude';
  const d = decide(input, hookio.rootOf(input, args), surface, { write: true });
  if (d) process.stdout.write(JSON.stringify(d) + '\n');
  return 0;
}

if (require.main === module) {
  try { process.exit(main(process.argv.slice(2))); } catch (e) {
    process.stdout.write(JSON.stringify({ hookSpecificOutput: { hookEventName: 'PreToolUse',
      permissionDecision: 'deny', permissionDecisionReason: `Harness gate 出错，先停：${e.message}` } }) + '\n');
    process.exit(0);
  }
}

module.exports = { isPush, decide };
