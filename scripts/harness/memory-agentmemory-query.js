#!/usr/bin/env node

const path = require('path');
const { loadHarnessConfig } = require('./lib/config-loader');
const { agentmemoryHealth, agentmemorySmartSearch, memoryConfig } = require('./lib/agentmemory-client');
const {
  classifyMemoryTrust,
  normalizeMemoryCandidate
} = require('./lib/memory-governance');

function parseArgs(argv) {
  const args = {
    target: process.cwd(),
    query: '',
    taskId: '',
    slug: '',
    limit: null,
    includeCandidates: false,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--target') args.target = argv[++i];
    else if (arg === '--query') args.query = argv[++i];
    else if (arg === '--task-id') args.taskId = argv[++i];
    else if (arg === '--slug') args.slug = argv[++i];
    else if (arg === '--limit') args.limit = Number.parseInt(argv[++i], 10);
    else if (arg === '--include-candidates') args.includeCandidates = true;
    else if (arg === '--json') args.json = true;
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.target = path.resolve(args.target);
  return args;
}

function add(report, kind, message) {
  report[kind].push(message);
}

function resultText(item) {
  if (typeof item === 'string') return item;
  if (!item || typeof item !== 'object') return '';
  return item.text || item.content || item.memory || item.claim || item.summary || JSON.stringify(item);
}

function resultSource(item, index) {
  if (!item || typeof item !== 'object') return `agentmemory:${index}`;
  return item.id || item.source || item.session_id || item.sessionId || `agentmemory:${index}`;
}

function resultList(data) {
  if (Array.isArray(data)) return data;
  if (data && Array.isArray(data.results)) return data.results;
  if (data && Array.isArray(data.memories)) return data.memories;
  if (data && Array.isArray(data.items)) return data.items;
  return [];
}

async function buildReport(args) {
  const report = {
    target: args.target,
    pass: [],
    warn: [],
    fail: [],
    details: {
      configPath: null,
      memoryEnabled: false,
      query: args.query || [args.taskId, args.slug].filter(Boolean).join(' '),
      candidates: [],
      governed: []
    }
  };

  const loaded = loadHarnessConfig(args.target);
  report.details.configPath = loaded.path;
  if (loaded.error) {
    add(report, 'fail', `Harness config could not be loaded: ${loaded.error}`);
    return report;
  }
  const config = loaded.data || {};
  const resolved = memoryConfig(config);
  report.details.memoryEnabled = resolved.enabled;
  if (!resolved.enabled) {
    add(report, 'warn', 'memoryIntegration.enabled is false; skipped agentmemory query');
    return report;
  }
  if (!report.details.query) {
    add(report, 'fail', '--query, --task-id, or --slug is required');
    return report;
  }

  const health = await agentmemoryHealth(config);
  if (!health.ok) {
    add(report, 'warn', `agentmemory unavailable: ${health.error || `HTTP ${health.statusCode}`}`);
    return report;
  }
  add(report, 'pass', 'agentmemory health check passed');

  const response = await agentmemorySmartSearch(config, report.details.query, { limit: args.limit || resolved.readPolicy.maxMemoriesPerTask || 5 });
  if (!response.ok) {
    add(report, 'warn', `agentmemory smart-search failed: ${response.error || `HTTP ${response.statusCode}`}`);
    return report;
  }
  const items = resultList(response.data).slice(0, args.limit || resolved.readPolicy.maxMemoriesPerTask || 5);
  const maxChars = resolved.readPolicy.maxMemoryChars || 1200;
  items.forEach((item, index) => {
    const claim = resultText(item).slice(0, maxChars);
    const candidate = normalizeMemoryCandidate({
      id: `MEM-AGENTMEMORY-${index + 1}`,
      project: path.basename(args.target),
      scope: 'project',
      sourceType: 'model-summary',
      source: resultSource(item, index + 1),
      claim,
      confidence: 'medium',
      authority: 'candidate',
      status: 'candidate',
      riskTags: ['agentmemory']
    });
    const classified = classifyMemoryTrust(candidate, { targetRoot: args.target });
    const entry = {
      candidate,
      recommendedStatus: classified.recommendedStatus,
      trustedForContext: classified.trustedForContext,
      reasons: classified.reasons
    };
    report.details.candidates.push(entry);
    if (classified.trustedForContext || args.includeCandidates) report.details.governed.push(entry);
  });
  add(report, 'pass', `agentmemory returned ${items.length} item(s); governed output ${report.details.governed.length}`);
  return report;
}

function printSection(title, items) {
  console.log(`\n${title} (${items.length})`);
  if (items.length === 0) console.log('  None');
  else for (const item of items) console.log(`  - ${item}`);
}

function printReport(report) {
  console.log(`Harness agentmemory query: ${report.target}`);
  printSection('PASS', report.pass);
  printSection('WARN', report.warn);
  printSection('FAIL', report.fail);
  if (report.details.governed.length > 0) {
    console.log('\nGOVERNED MEMORY');
    for (const entry of report.details.governed) console.log(`- ${entry.candidate.id}: ${entry.candidate.claim}`);
  }
}

(async () => {
  try {
    const args = parseArgs(process.argv.slice(2));
    const report = await buildReport(args);
    if (args.json) console.log(JSON.stringify(report, null, 2));
    else printReport(report);
    if (report.fail.length > 0) process.exit(1);
  } catch (error) {
    console.error(`ERROR: ${error.message}`);
    process.exit(1);
  }
})();
