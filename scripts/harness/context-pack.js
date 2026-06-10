#!/usr/bin/env node

const path = require('path');
const { buildReport } = require('./lib/context-pack');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    taskId: null,
    slug: null,
    aliases: [],
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--task-id') args.taskId = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--alias') args.aliases.push(argv[++i]);
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else process.stdout.write(report.markdown);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
