'use strict';
// 钩子的输入输出管道。Claude Code 从 stdin 喂一份 JSON（含 cwd、transcript_path）。
const fs = require('fs');

// stdin 上那份 JSON。没有、读不到、不是 JSON，都当空对象，不许因此崩掉钩子。
function readInput() {
  let raw = '';
  try { raw = fs.readFileSync(0, 'utf8'); } catch { return {}; }
  try { return JSON.parse(raw) || {}; } catch { return {}; }
}

// 项目根：命令行 --target 优先（测试用），否则钩子给的 cwd
function rootOf(input, argv) {
  const i = (argv || []).indexOf('--target');
  if (i !== -1 && argv[i + 1]) return require('path').resolve(argv[i + 1]);
  return (input && input.cwd) || process.cwd();
}

// 测试跑没跑、过没过：从对话记录里最后一条测试命令的输出取。
// 代理编不出这一行 —— 它是命令真跑过才有的（R4 机器那栏）。
const SUMMARY = [
  /(?:^|\s)#\s*(pass \d+.*)/i,            // node --test
  /(\d+ (?:passing|passed)[^\n]*)/i,      // mocha / jest
  /(Tests:\s+[^\n]*)/i,                   // jest 汇总
  /(\d+ passed[^\n]*)/i,
  /(ok \d+ - [^\n]*)/i,
];

function pickSummary(text) {
  for (const re of SUMMARY) {
    const m = String(text).match(re);
    if (m) return m[1].trim().slice(0, 120);
  }
  return null;
}

function testsFromTranscript(file) {
  let lines;
  try { lines = fs.readFileSync(file, 'utf8').split('\n'); } catch { return null; }
  // 从后往前找，最近一次跑的算这一轮的
  for (let i = lines.length - 1; i >= 0; i--) {
    const l = lines[i].trim();
    if (l === '' || !/pass|passing|passed|Tests:|fail/i.test(l)) continue;
    let rec;
    try { rec = JSON.parse(l); } catch { continue; }
    const s = pickSummary(JSON.stringify(rec));
    if (s) return s;
  }
  return null;
}

// stdout 上的字进上下文（SessionStart / UserPromptSubmit）
function say(text, cap) {
  const t = String(text || '').trim();
  if (t === '') return;
  process.stdout.write((cap && t.length > cap ? t.slice(0, cap) : t) + '\n');
}

module.exports = { readInput, rootOf, pickSummary, testsFromTranscript, say };
