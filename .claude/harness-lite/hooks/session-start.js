#!/usr/bin/env node
'use strict';
// SessionStart：开对话、压缩之后（source=compact）读回从树顶到当前叶子那一条链（R6）。
// stdout 直接进上下文。里面不放当前时间、commit SHA 这类会过期的东西 ——
// 钩子塞的字会存进对话记录，--resume 回放存下来的那份、不重跑钩子。
const path = require('path');
const hookio = require('../lib/hookio.js');
const tree = require('../lib/tree.js');

const CAP = 2000;

// 超了只留标题和当前叶子
function short(c) {
  const out = [];
  if (c.plan) out.push(`总计划：${c.plan.title}`);
  if (c.stage) out.push(`阶段：${c.stage.title}`);
  if (c.leaf) {
    out.push(`当前叶子：${c.leaf.title}（${path.basename(c.leaf.file)}）`);
    out.push('允许动：' + c.allowed.map((a) => a.path).join('、'));
  }
  return out.join('\n');
}

function build(root) {
  const c = tree.readChain(root);
  if (!c.installed) return '';
  const p = tree.progress(root, c);
  const body = [tree.formatChain(c), tree.formatProgress(p)].join('\n');
  const text = ['接着上次的活，当前这条链：', body].join('\n');
  if (text.length <= CAP) return text;
  return ['接着上次的活，当前这条链：', short(c), tree.formatProgress(p)].join('\n');
}

function main(argv) {
  const input = hookio.readInput();
  hookio.say(build(hookio.rootOf(input, argv)));
  return 0;
}

if (require.main === module) {
  try { process.exit(main(process.argv.slice(2))); } catch { process.exit(0); }
}

module.exports = { build, CAP };
