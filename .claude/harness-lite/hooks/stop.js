#!/usr/bin/env node
'use strict';
// Stop：代理这轮说完。取 git 事实拼报告下半截、追加一行使用记录、数阶段还剩几个叶子（R6、R12）。
// 只说下一步是什么，绝对不拦。这个文件里一次都不许出现 decision 那个拦人的值。
const hookio = require('../lib/hookio.js');
const tree = require('../lib/tree.js');
const gitfacts = require('../lib/gitfacts.js');
const report = require('../lib/report.js');
const usage = require('../lib/usage.js');

// 阶段那次交代的九件（R9）。只说"该交代了"是指向空气 —— 哪几件得写出来。
// 第 9 件 2026-08-06 补：那次报告写在一个工作树、提交在另一个，写成功不报错、
// 另一边 git status 也不提，于是记录没进任何提交。所以提交完得回头核对一遍。
const NINE = ['入口切没切走', '实际结果和验收', '遗留问题和谁接手', '改动在哪',
  '有没有并主线', '分支和工作树怎么处理', '测试材料去向', '下一个入口',
  '你写的记录在不在这个提交里（把这个提交实际包含的文件列一遍）'];

// 还剩的叶子 → 推着接着干（R1：数得出剩几个，才不用停下来问）
function nextLine(p) {
  if (p.allDone) {
    return '这个阶段的叶子全完了，停下来答这九件、等用户验收：'
      + NINE.map((s, i) => `${i + 1} ${s}`).join('、');
  }
  const left = p.remaining;
  return `这个阶段还剩 ${left.length} 个叶子：${left.join('、')}。接着干下一个。`;
}

function run(root, opts) {
  const o = opts || {};
  const c = tree.readChain(root);
  if (!c.installed) return { installed: false, context: '' };

  const p = tree.progress(root, c);
  const facts = gitfacts.facts(root);
  const allowed = c.allowed.map((a) => a.path);
  const tests = o.tests !== undefined ? o.tests : hookio.testsFromTranscript(o.transcript);

  // 机器那栏：全部来自 git 和命令输出，代理写不进去（R4）
  const wroteHead = report.hasPendingHead(root, o.date);
  const m = report.writeMachine(root, { facts, allowed, tests }, { write: o.write, date: o.date });

  // 一行一轮，只追加（R12）。工具那栏是这一轮实际跑过的 hl 子命令。
  // 文件数跟报告那行数的是同一批 —— harness 自己的账不算代理动的文件，
  // 两处数字对不上，用户就得先想"哪个才对"，那这行就废了
  const own = facts.repo ? facts.paths.filter((p) => !gitfacts.isOwn(p)) : [];
  const rec = {
    at: o.at,
    stage: c.stage ? c.stage.id : null,
    leaf: c.leaf ? c.leaf.name.split('-')[0] : null,
    report: wroteHead,
    files: own.length,
    outOfScope: m.outOfScope,
    tests: !!tests,
    tools: usage.takeTools(root, { write: o.write }),
  };
  const u = usage.append(root, rec, { write: o.write });

  return {
    installed: true,
    machine: m.lines,
    files: own,
    outOfScope: m.outOfScope,
    usage: u.text,
    progress: p,
    context: nextLine(p),
  };
}

function main(argv) {
  const input = hookio.readInput();
  const root = hookio.rootOf(input, argv);
  const r = run(root, { write: true, transcript: input.transcript_path });
  // Stop 只能用 hookSpecificOutput.additionalContext，stdout 进不去
  process.stdout.write(JSON.stringify({
    hookSpecificOutput: { hookEventName: 'Stop', additionalContext: r.context },
  }) + '\n');
  return 0;
}

if (require.main === module) {
  try { process.exit(main(process.argv.slice(2))); } catch { process.exit(0); }
}

module.exports = { run, nextLine };
