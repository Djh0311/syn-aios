#!/usr/bin/env node
'use strict';
// 唯一命令入口。子命令派发，报错就是一行人话 + 非零退出码。
const path = require('path');

const ROOT = path.join(__dirname, '..');

// 带值的就这一个。--write / --json 是开关，
// 不分开的话 `hl report --write "这轮：…"` 会把那句话当成 --write 的值吃掉。
const TAKES_VALUE = new Set(['--target', '--break-glass', '--reason']);

function flag(args, name) {
  const i = args.indexOf(name);
  if (i === -1) return null;
  if (!TAKES_VALUE.has(name)) return true;
  return args[i + 1] === undefined || args[i + 1].startsWith('--') ? true : args[i + 1];
}

// --target 给的项目目录，默认当前目录
const target = (args) => path.resolve(String(flag(args, '--target') || process.cwd()));

function out(args, data, text) {
  console.log(flag(args, '--json') ? JSON.stringify(data, null, 2) : text);
}

// 位置参数：不是 --xxx、也不是带值那种 --xxx 的值
function positional(args) {
  const out = [];
  for (let i = 0; i < args.length; i++) {
    if (args[i].startsWith('--')) {
      if (TAKES_VALUE.has(args[i]) && args[i + 1] !== undefined && !args[i + 1].startsWith('--')) i++;
      continue;
    }
    out.push(args[i]);
  }
  return out;
}

const CMDS = {
  limits() {
    const { check, format } = require('../lib/limits.js');
    const rows = check(ROOT);
    console.log(format(rows));
    return rows.some((r) => r.over) ? 1 : 0;
  },

  // 从树顶到当前叶子那一条链（R6：只读一条链，不读整棵树）
  chain(args) {
    const tree = require('../lib/tree.js');
    const c = tree.readChain(target(args));
    out(args, c, tree.formatChain(c));
  },

  // 共 N、完成 M、在干第 M+1（R2）
  progress(args) {
    const tree = require('../lib/tree.js');
    const p = tree.progress(target(args));
    out(args, p, tree.formatProgress(p));
  },

  // git 只读事实（R11）
  facts(args) {
    const gf = require('../lib/gitfacts.js');
    const f = gf.facts(target(args));
    out(args, f, gf.format(f));
  },

  // 叶子退场：声明当前 leaf 完成并归档。dry-run 默认
  done(args) {
    const name = positional(args)[0];
    if (!name) { console.error('要说哪个叶子：hl done L3 [--write]'); return 1; }
    const r = require('../lib/done.js').done(target(args), name, { write: !!flag(args, '--write') });
    if (!r.ok) { console.error(r.msg); return 1; }
    out(args, r, r.msg + (r.wrote ? '' : '（dry-run，加 --write 才真移）') + '\n' + r.handoff);
  },

  // 最后一个 leaf 完成后，单独归档 stage；默认仍是 dry-run。
  'close-stage'(args) {
    if (positional(args).length) { console.error('用法：hl close-stage [--write]'); return 1; }
    const r = require('../lib/close-stage.js').closeStage(target(args), { write: !!flag(args, '--write') });
    if (!r.ok) { console.error(r.msg); return 1; }
    out(args, r, r.msg + (r.wrote ? '' : '（dry-run，加 --write 才真归档）'));
  },

  // 未完成不冒充完成：放回 unfinished/，原因写进 leaf。
  park(args) {
    const pos = positional(args);
    if (!pos[0] || !pos[1]) { console.error('用法：hl park <叶子> <原因> [--write]'); return 1; }
    const r = require('../lib/lifecycle.js').park(target(args), pos[0], pos.slice(1).join(' '), { write: !!flag(args, '--write') });
    if (!r.ok) { console.error(r.msg); return 1; }
    out(args, r, r.msg + (r.file ? `：${r.file}` : ''));
  },

  // 没有 current 时，从 unfinished/ 恢复一个；需要整阶段授权。
  resume(args) {
    const name = positional(args)[0];
    if (!name) { console.error('用法：hl resume <叶子> [--write]'); return 1; }
    const r = require('../lib/lifecycle.js').resume(target(args), name, { write: !!flag(args, '--write') });
    if (!r.ok) { console.error(r.msg); return 1; }
    out(args, r, r.msg);
  },

  // 所有执行面共用的硬门。普通动作直接过；未获授权的硬门退出码 2。
  gate(args) {
    const pos = positional(args);
    if (pos.length < 3) { console.error('用法：hl gate <类别> <动作> <目标> [--break-glass <grant>] [--reason <原因>] [--write]'); return 1; }
    const q = { category: pos[0], operation: pos[1], target: pos.slice(2).join(' ') };
    const r = require('../lib/gate.js').evaluate(target(args), q, {
      breakGlass: flag(args, '--break-glass'), reason: flag(args, '--reason'), write: !!flag(args, '--write'),
    });
    out(args, r, r.decision === 'allow' ? `通过：${r.reason}` : `停下：${r.reason}`);
    return r.decision === 'allow' ? 0 : 2;
  },

  auth(args) {
    const r = require('../lib/authorization.js').active(target(args));
    out(args, r, r.ok ? `当前授权：${r.record.id}（${r.record.scope.kind} ${r.record.scope.id}）` : `当前无有效授权：${r.why}`);
    return r.ok ? 0 : 2;
  },

  // 代理那栏落盘。机器那栏由 hooks/stop.js 写，这里写不了（R4）
  report(args) {
    const root = target(args);
    const tree = require('../lib/tree.js');
    const report = require('../lib/report.js');
    const r = report.writeHead(root, tree.progress(root), positional(args), { write: !!flag(args, '--write') });
    if (r.warn) console.log(r.warn);
    out(args, r, r.wrote ? `已追加到 ${r.file}\n${r.text}` : `${r.note}（dry-run，加 --write 才真写）\n${r.text}`);
  },

  // 每个工具上次什么时候用、一共几次（R12）
  usage(args) {
    const u = require('../lib/usage.js');
    const rows = u.table(target(args));
    out(args, rows, u.format(rows));
  },

  // 给改动路径 → 该跑哪几个测试 + 一条能直接复制的命令（R11）
  tests(args) {
    const files = positional(args);
    if (!files.length) { console.error('要给改动路径：hl tests src/order/create.ts'); return 1; }
    const tp = require('../lib/tests-pick.js');
    const p = tp.pick(target(args), files);
    out(args, p, tp.format(p));
  },

  // 只有这条显式命令会执行项目登记；task 只按给出的改动路径选择。
  check(args) {
    const [profile, ...files] = positional(args);
    if (!profile) { console.error('用法：hl check <quick|task|full|manual> [改动路径...]'); return 1; }
    const checks = require('../lib/checks.js');
    const r = checks.run(target(args), profile, files);
    out(args, r, checks.format(r));
    return r.ok ? 0 : 1;
  },

  // 有哪些模块、每个几行、导出了什么。写新东西前先看有没有现成的（R11）
  map(args) {
    const m = require('../lib/map.js');
    const built = m.build(target(args), { dirs: positional(args) });
    out(args, built, m.format(built));
  },

  // 一句话追加、一句话搜。没有字段、没有必填项（R11）
  mistake(args) {
    const mk = require('../lib/mistakes.js');
    const root = target(args);
    const pos = positional(args);
    if (pos[0] === 'add') {
      const r = mk.add(root, pos.slice(1).join(' '), { write: !!flag(args, '--write') });
      if (!r.ok) { console.error(r.msg); return 1; }
      out(args, r, r.msg + (r.wrote ? '' : '（dry-run，加 --write 才真写）'));
      return 0;
    }
    const word = pos.join(' ');
    const hits = mk.search(root, word);
    out(args, hits, mk.format(hits, word));
  },
};

function main(argv) {
  const [cmd, ...args] = argv;
  if (!cmd || cmd === '--help' || cmd === '-h') {
    console.log(`用法：hl <${Object.keys(CMDS).join('|')}> [--target <项目目录>] [--json]`);
    return cmd ? 0 : 1;
  }
  if (!CMDS[cmd]) {
    console.error(`没有这个子命令：${cmd}。有的是：${Object.keys(CMDS).join('、')}`);
    return 1;
  }
  // 这一轮跑过哪些工具，钩子写记录时要（R12）。记不上不挡主流程
  if (cmd !== 'limits') require('../lib/usage.js').noteTool(target(args), cmd);
  // 指的目录和人在的仓库不是一个 → 先说一句，绝不拦（R7）。--json 那条路要保持纯 JSON
  if (cmd !== 'limits' && !flag(args, '--json')) {
    const note = require('../lib/gitfacts.js').splitCheck(target(args), process.cwd());
    if (note) console.log(note);
  }
  return CMDS[cmd](args) || 0;
}

if (require.main === module) {
  try {
    process.exit(main(process.argv.slice(2)));
  } catch (e) {
    console.error(e.message);
    process.exit(1);
  }
}

module.exports = { main, flag };
