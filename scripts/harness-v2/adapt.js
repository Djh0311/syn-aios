#!/usr/bin/env node
'use strict';

const { buildAdaptReport, formatAdaptReport } = require('./lib/adapt-inspect');
const { sanitizeOutputText } = require('./lib/output-safety');

function usage() {
  return [
    'Usage: node scripts/harness-v2/adapt.js inspect --target <path> [--json]',
    '',
    'Static, bounded, read-only inspection. It does not execute project scripts,',
    'use the network, start services, write files, change permissions, or read',
    'global instructions.',
  ].join('\n');
}

function takeValue(argv, index, flag) {
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) throw new Error(`${flag} requires a value`);
  return value;
}

function parseArgs(argv) {
  const args = {
    command: null,
    target: null,
    json: false,
    help: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (!args.command && !argument.startsWith('--')) {
      args.command = argument;
    } else if (argument === '--target') {
      args.target = takeValue(argv, index, argument);
      index += 1;
    } else if (argument === '--json') {
      args.json = true;
    } else if (argument === '--help' || argument === '-h') {
      args.help = true;
    } else {
      throw new Error(`Unsupported argument: ${argument}`);
    }
  }
  if (args.help) return args;
  if (args.command !== 'inspect') throw new Error('Expected the adapt inspect command');
  if (!args.target) throw new Error('--target is required');
  return args;
}

function runCli(argv, io = process) {
  try {
    const args = parseArgs(argv);
    if (args.help) {
      io.stdout.write(`${usage()}\n`);
      return 0;
    }
    const report = buildAdaptReport(args.target);
    io.stdout.write(formatAdaptReport(report, { json: args.json }));
    return 0;
  } catch (error) {
    const message = sanitizeOutputText(error && error.message, 240) || 'adapt inspect failed';
    io.stderr.write(`adapt inspect: ${message}\n`);
    return 1;
  }
}

if (require.main === module) {
  const exitCode = runCli(process.argv.slice(2));
  if (exitCode !== 0) process.exitCode = exitCode;
}

module.exports = {
  parseArgs,
  runCli,
  usage,
};
