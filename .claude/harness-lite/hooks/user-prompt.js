#!/usr/bin/env node
'use strict';
// UserPromptSubmit：用户说话之前提醒在第几步、能动哪几个目录（R6）。
// 一行。没有当前叶子就什么都不输出、退出码 0 —— 不许因为没叶子就拦住用户说话。
const hookio = require('../lib/hookio.js');
const tree = require('../lib/tree.js');

// 第几步：叶子里的步骤是给人看的，机器数不出干到第几步，
// 所以只说"共几步"，不编一个步号（编出来的号会误导，比没有更糟）。
function build(root) {
  const c = tree.readChain(root);
  if (!c.installed || !c.leaf) return '';
  const bits = [`当前 ${c.leaf.title}`];
  if (c.leaf.steps.length) bits.push(`共 ${c.leaf.steps.length} 步`);
  if (c.allowed.length) {
    bits.push('允许动 ' + c.allowed.map((a) => a.path + (a.isNew ? '[新增]' : '')).join('、'));
  }
  if (c.stage && c.stage.forbidden.length) bits.push('不许动 ' + c.stage.forbidden.join('、'));
  return bits.join('；');
}

function main(argv) {
  const input = hookio.readInput();
  hookio.say(build(hookio.rootOf(input, argv)));
  return 0;
}

if (require.main === module) {
  try { process.exit(main(process.argv.slice(2))); } catch { process.exit(0); }
}

module.exports = { build };
