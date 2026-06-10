#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const {
  normalizeTaskPackage,
  renderTaskPackageMarkdown,
  taskPackageFiles,
  validateTaskPackage
} = require('./lib/task-package-schema');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    id: null,
    mission: null,
    path: null,
    readScope: [],
    writeScope: [],
    forbiddenScope: [],
    acceptance: [],
    verification: [],
    riskTags: [],
    inputs: [],
    relatedMistakes: [],
    write: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--id') args.id = argv[++i];
    else if (arg === '--mission') args.mission = argv[++i];
    else if (arg === '--path') args.path = argv[++i];
    else if (arg === '--read-scope') args.readScope.push(argv[++i]);
    else if (arg === '--write-scope') args.writeScope.push(argv[++i]);
    else if (arg === '--forbidden-scope') args.forbiddenScope.push(argv[++i]);
    else if (arg === '--acceptance') args.acceptance.push(argv[++i]);
    else if (arg === '--verification') args.verification.push(argv[++i]);
    else if (arg === '--risk-tag') args.riskTags.push(argv[++i]);
    else if (arg === '--input') args.inputs.push(argv[++i]);
    else if (arg === '--related-mistake') args.relatedMistakes.push(argv[++i]);
    else if (arg === '--write') args.write = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function rel(root, filePath) {
  return path.relative(root, filePath).split(path.sep).join('/') || '.';
}

function buildPlan(args) {
  if (!fs.existsSync(args.target) || !fs.statSync(args.target).isDirectory()) {
    throw new Error(`Target must be an existing directory: ${args.target}`);
  }

  const data = normalizeTaskPackage(args);
  const validation = validateTaskPackage(data, { source: data.id });
  if (!validation.valid) throw new Error(validation.errors.join('; '));

  const files = taskPackageFiles(args.target, data.id);
  const markdown = renderTaskPackageMarkdown(data);
  return {
    mode: args.write ? 'write' : 'dry-run',
    target: args.target,
    taskPackage: data,
    files: {
      json: files.json,
      markdown: files.markdown
    },
    relativeFiles: {
      json: rel(args.target, files.json),
      markdown: rel(args.target, files.markdown)
    },
    markdown
  };
}

function writePlan(plan) {
  for (const filePath of [plan.files.json, plan.files.markdown]) {
    if (fs.existsSync(filePath)) throw new Error(`Task package file already exists; refusing to overwrite: ${filePath}`);
  }

  fs.mkdirSync(path.dirname(plan.files.json), { recursive: true });
  fs.writeFileSync(plan.files.json, `${JSON.stringify(plan.taskPackage, null, 2)}\n`, { encoding: 'utf8', flag: 'wx' });
  fs.writeFileSync(plan.files.markdown, plan.markdown, { encoding: 'utf8', flag: 'wx' });
}

function printText(plan) {
  console.log('Harness task package scaffold');
  console.log(`Mode: ${plan.mode}`);
  console.log(`Target: ${plan.target}`);
  console.log(`JSON: ${plan.relativeFiles.json}`);
  console.log(`Markdown: ${plan.relativeFiles.markdown}`);
  console.log('\nMarkdown preview:');
  console.log(plan.markdown);
  if (plan.mode === 'dry-run') console.log('Dry run only. Re-run with --write to create the files.');
  else console.log('Created task package JSON and Markdown render.');
}

try {
  const args = parseArgs(process.argv.slice(2));
  const plan = buildPlan(args);
  if (args.write) writePlan(plan);
  if (args.json) {
    console.log(JSON.stringify({
      mode: plan.mode,
      target: plan.target,
      id: plan.taskPackage.id,
      files: plan.relativeFiles,
      taskPackage: plan.taskPackage,
      markdown: plan.markdown
    }, null, 2));
  } else {
    printText(plan);
  }
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
