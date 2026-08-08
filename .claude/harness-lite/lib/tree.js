'use strict';
// 读一条链（树顶到当前叶子）和数进度。全只读。
// leaves/ 里只能有一个 current；unfinished/ 放未开始、暂停或受阻。
const fs = require('fs');
const path = require('path');
const leaf = require('./leaf.js');

const hdir = (root) => path.join(root, 'docs', 'harness');

function mdFiles(dir) {
  try {
    return fs.readdirSync(dir).filter((f) => f.endsWith('.md')).sort()
      .map((f) => path.join(dir, f));
  } catch { return []; }
}

// done/YYYY-MM/*.md，往下一层
function doneFiles(dir) {
  let out = [];
  let ents;
  try { ents = fs.readdirSync(dir, { withFileTypes: true }); } catch { return out; }
  for (const e of ents.sort((a, b) => a.name.localeCompare(b.name))) {
    const p = path.join(dir, e.name);
    if (e.isDirectory()) out = out.concat(mdFiles(p));
    else if (e.name.endsWith('.md')) out.push(p);
  }
  return out;
}

function parseStage(file) {
  const text = leaf.read(file);
  if (text == null) return null;
  const id = path.basename(file, '.md');
  return {
    file,
    id,
    title: leaf.title(text, id),
    plan: leaf.field(text, '总计划'),
    goal: leaf.field(text, '目标'),
    allowed: leaf.bullets(text, '允许动'),
    forbidden: leaf.bullets(text, '不许动'),
    // 阶段文件里的勾选框只给人看，机器不拿它算数（判据只有 done/ 里有没有文件）
    leafList: leaf.after(text, /^##\s*叶子/)
      .map((l) => l.trim().match(/^-\s*\[( |x|X)\]\s*(.+)$/))
      .filter(Boolean)
      .map((m) => ({ checked: m[1] !== ' ', label: m[2].trim() })),
  };
}

function readChain(root) {
  const hd = hdir(root);
  const planText = leaf.read(path.join(hd, 'plan.md'));
  const stages = mdFiles(path.join(hd, 'stages'));
  const stage = stages.length ? parseStage(stages[0]) : null;
  const leaves = mdFiles(path.join(hd, 'leaves'));
  const unfinished = mdFiles(path.join(hd, 'unfinished'));
  const cur = leaves.length === 1 ? leaf.parse(leaves[0]) : null;
  const lifecycle = { ok: leaves.length <= 1 && (!cur || !stage || cur.stageId === stage.id),
    currentCount: leaves.length, unfinishedCount: unfinished.length };
  return {
    root,
    installed: fs.existsSync(hd),
    plan: planText == null ? null : { title: leaf.title(planText, '总计划'), goal: leaf.field(planText, '目标') },
    stage,
    leaf: lifecycle.ok ? cur : null,
    lifecycle,
    allowed: lifecycle.ok && cur ? leaf.checkAllowed(cur, stage && stage.allowed) : [],
    extraStages: stages.slice(1).map((f) => path.basename(f)),
    note: leaves.length > 1 ? `当前叶子有 ${leaves.length} 个，先收回到只剩一个`
      : cur && stage && cur.stageId !== stage.id ? '当前叶子不属于当前阶段，先修正入口'
      : unfinished.length ? '当前没有 active leaf，未完成区还有工作' : '这个阶段的叶子都完了',
  };
}

// 共 N、完成 M、在干第 M+1。属于本阶段的判断：叶子文件里 `阶段：` 那行
function progress(root, chain) {
  const c = chain || readChain(root);
  const hd = hdir(root);
  const stageId = c.stage ? c.stage.id : null;
  const current = mdFiles(path.join(hd, 'leaves')).map((f) => leaf.parse(f)).filter(Boolean)
    .filter((l) => !stageId || l.stageId === stageId);
  const unfinished = mdFiles(path.join(hd, 'unfinished')).map((f) => leaf.parse(f)).filter(Boolean)
    .filter((l) => !stageId || l.stageId === stageId);
  const done = doneFiles(path.join(hd, 'done')).map((f) => leaf.parse(f)).filter(Boolean)
    .filter((l) => l.stageId || !/^stage-/i.test(l.name))
    .filter((l) => !stageId || l.stageId === stageId);
  return {
    stage: c.stage,
    total: current.length + unfinished.length + done.length,
    done: done.length,
    current: current.length === 1 ? done.length + 1 : null,
    currentLeafId: current.length === 1 ? current[0].name.split('-')[0] : null,
    doneTitles: done.map((l) => l.title),
    remaining: current.concat(unfinished).map((l) => l.title),
    remainingIds: current.concat(unfinished).map((l) => l.name.split('-')[0]),
    unfinishedIds: unfinished.map((l) => l.name.split('-')[0]),
    allDone: current.length === 0 && unfinished.length === 0,
  };
}

function formatChain(c) {
  if (!c.installed) return `${c.root} 里没有 docs/harness/，先跑 install.js`;
  const out = [];
  if (c.plan) out.push(`总计划：${c.plan.title}`);
  if (c.stage) {
    out.push(`阶段：${c.stage.title}`);
    if (c.stage.goal) out.push(`阶段目标：${c.stage.goal}`);
  }
  if (c.leaf) {
    out.push(`当前叶子：${c.leaf.title}（${path.basename(c.leaf.file)}）`);
    if (c.leaf.goal) out.push(`目标：${c.leaf.goal}`);
    if (c.leaf.doneWhen) out.push(`干完的标准：${c.leaf.doneWhen}`);
    if (c.leaf.steps.length) out.push(`步骤：${c.leaf.steps.length} 步，第 1 步 ${c.leaf.steps[0]}`);
    out.push('允许动：' + c.allowed.map((a) => a.path + (a.isNew ? '  [新增，指不回阶段文件]' : '')).join('、'));
  } else {
    out.push(c.note);
  }
  if (c.stage && c.stage.forbidden.length) out.push('不许动：' + c.stage.forbidden.join('、'));
  if (c.extraStages.length) out.push(`stages/ 里还有 ${c.extraStages.join('、')}，干完的请挪到 done/`);
  return out.join('\n');
}

function formatProgress(p) {
  const head = p.stage ? `${p.stage.title}：` : '';
  const line = p.allDone
    ? `${head}${p.total} 个叶子，全完了`
    : p.current == null ? `${head}${p.total} 个叶子，完成 ${p.done}，当前已退出，未完成 ${p.remaining.length}`
      : `${head}${p.total} 个叶子，完成 ${p.done}，在干第 ${p.current} 个`;
  return [line, `剩下：${p.remaining.length ? p.remaining.join('、') : '无'}`].join('\n');
}

module.exports = { hdir, mdFiles, doneFiles, parseStage, readChain, progress, formatChain, formatProgress };
