#!/usr/bin/env node
'use strict';

// Adaptive Harness v0.5 — 七类旧文档能力的只读查询壳（AH-050-10）
//
// 这是 `lib/legacy-docs.js` 的薄入口：它只读取一次显式 JSON 输入，调用纯函数，
// 再把结果打印出来。它没有 --write，也不枚举 history；实际 history 恢复继续走
// 既有 `nodes.js history --id` 的显式入口。

const fs = require('node:fs');
const path = require('node:path');

const legacy = require('./lib/legacy-docs');

const CAPABILITY_COMMANDS = Object.freeze([
  'queue',
  'questions',
  'stage-contract',
  'checkpoint',
  'handoff',
  'evidence-index',
  'prevention',
]);
const COMMANDS = Object.freeze(['capabilities', ...CAPABILITY_COMMANDS]);

function usage() {
  return [
    `用法：knowledge.js <${COMMANDS.join('|')}>`,
    '[--input <JSON>] [--json]',
    '所有命令只读；不支持 --write。history 请使用 nodes.js history --id。',
  ].join(' ');
}

function parseArguments(argv) {
  const options = { command: null, input: null, json: false, write: false, cwd: process.cwd() };
  const list = Array.isArray(argv) ? argv.slice() : [];
  if (list.length && !String(list[0]).startsWith('--')) options.command = list.shift();
  while (list.length) {
    const token = list.shift();
    if (token === '--input') {
      options.input = list.shift() || null;
      if (!options.input) return { ok: false, code: 'ARGUMENT_ERROR', error: '--input 需要 JSON 文件路径', options };
      continue;
    }
    if (token === '--json') { options.json = true; continue; }
    if (token === '--write') { options.write = true; continue; }
    return { ok: false, code: 'ARGUMENT_ERROR', error: `未知参数 ${token}`, options };
  }
  if (!options.command || !COMMANDS.includes(options.command)) {
    return { ok: false, code: 'ARGUMENT_ERROR', error: usage(), options };
  }
  return { ok: true, options };
}

function inputFailure(code, error) {
  return { ok: false, code, error, written: false };
}

function readInput(inputPath, cwd, runtime) {
  if (typeof inputPath !== 'string' || inputPath.trim() === '') {
    return inputFailure('KNOWLEDGE_INPUT_REQUIRED', '该知识查询需要 --input <JSON>；不扫描目录或默认文件');
  }
  const absolute = path.resolve(cwd || process.cwd(), inputPath);
  const reader = runtime && typeof runtime.readFileSync === 'function' ? runtime.readFileSync : fs.readFileSync;
  let raw;
  try {
    raw = reader(absolute, 'utf8');
  } catch (error) {
    return inputFailure('KNOWLEDGE_INPUT_READ_FAILED', `读不到 ${absolute}：${error.message}`);
  }
  try {
    const value = JSON.parse(raw);
    if (!value || typeof value !== 'object' || Array.isArray(value)) {
      return inputFailure('KNOWLEDGE_INPUT_INVALID', `${absolute} 顶层必须是 JSON object`);
    }
    return { ok: true, value, absolute };
  } catch (error) {
    return inputFailure('KNOWLEDGE_INPUT_INVALID', `${absolute} 不是合法 JSON：${error.message}`);
  }
}

function capabilityResult(registry) {
  const coverage = legacy.capabilityCoverageReport(registry);
  return {
    ...coverage,
    action: 'CAPABILITY_COVERAGE',
    route: 'READ_ONLY',
    written: false,
  };
}

function commandResult(command, data) {
  if (command === 'queue') return legacy.deriveQueue(data.records);
  if (command === 'questions') {
    return legacy.projectCurrentBlockers(data.questions, {
      nearestOwners: data.nearestOwners,
      nearestOwner: data.nearestOwner,
    });
  }
  if (command === 'stage-contract') return legacy.projectStageContract(data.record);
  if (command === 'checkpoint') return legacy.renderCheckpoint(data.checkpoint);
  if (command === 'handoff') return legacy.renderHandoff(data.handoff);
  if (command === 'evidence-index') return legacy.renderEvidenceIndex(
    data.manifests === undefined ? [] : data.manifests,
  );
  if (command === 'prevention') return legacy.renderPreventionCard(data.card);
  return inputFailure('KNOWLEDGE_COMMAND_UNKNOWN', `未知知识命令 ${command}`);
}

function publicResult(outcome, command) {
  const result = outcome && typeof outcome === 'object'
    ? { ...outcome }
    : inputFailure('KNOWLEDGE_OPERATION_FAILED', '知识查询没有返回结构化结果');
  return {
    ...result,
    command: command || result.command || null,
    written: false,
  };
}

function run(argv, runtime) {
  const parsed = parseArguments(argv);
  if (!parsed.ok) return publicResult(parsed, parsed.options && parsed.options.command);
  const options = parsed.options;
  if (options.write) {
    return publicResult(inputFailure('KNOWLEDGE_READ_ONLY', 'knowledge 是只读/纯输出入口，不支持 --write'), options.command);
  }

  try {
    if (options.command === 'capabilities' && !options.input) {
      return publicResult(capabilityResult(), options.command);
    }
    const input = readInput(options.input, options.cwd, runtime);
    if (!input.ok) return publicResult(input, options.command);
    if (options.command === 'capabilities') {
      return publicResult(capabilityResult(input.value.registry), options.command);
    }
    return publicResult(commandResult(options.command, input.value), options.command);
  } catch (error) {
    return publicResult(inputFailure(
      error && error.code ? error.code : 'KNOWLEDGE_OPERATION_FAILED',
      error && error.message ? error.message : String(error),
    ), options.command);
  }
}

if (require.main === module) {
  const outcome = run(process.argv.slice(2));
  process.stdout.write(`${JSON.stringify(outcome, null, 2)}\n`);
  process.exitCode = outcome.ok ? 0 : 1;
}

module.exports = {
  COMMANDS,
  CAPABILITY_COMMANDS,
  LEGACY_CAPABILITY_REGISTRY: legacy.LEGACY_CAPABILITY_REGISTRY,
  parseArguments,
  readInput,
  run,
};
