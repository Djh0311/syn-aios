#!/usr/bin/env node

const path = require('path');
const { loadSkillIndex, recommendSkills } = require('./lib/skill-index');

function parseArgs(argv) {
  const args = {
    skills: path.resolve(__dirname, '..', '..', 'skills'),
    text: '',
    limit: 8,
    json: false
  };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--skills') args.skills = argv[++i];
    else if (arg === '--text') args.text = argv[++i];
    else if (arg === '--limit') args.limit = Number(argv[++i]);
    else if (arg === '--json') args.json = true;
    else if (!arg.startsWith('--')) args.text = `${args.text} ${arg}`.trim();
    else throw new Error(`Unknown argument: ${arg}`);
  }

  args.skills = path.resolve(args.skills);
  if (!Number.isFinite(args.limit) || args.limit < 1) throw new Error('--limit must be a positive number');
  return args;
}

function buildReport(args) {
  const skills = loadSkillIndex(args.skills);
  const recommendations = recommendSkills(skills, args.text, { limit: args.limit });
  return {
    skillsRoot: args.skills,
    query: args.text,
    pass: recommendations.length > 0 ? [`Recommended ${recommendations.length} skill(s)`] : [],
    warn: args.text.trim() ? [] : ['No --text provided; only baseline recommendations may appear'],
    fail: skills.length > 0 ? [] : [`No SKILL.md files found under ${args.skills}`],
    details: {
      indexedSkills: skills.length,
      recommendations: recommendations.map((skill) => ({
        name: skill.name,
        score: skill.score,
        matches: skill.matches,
        description: skill.description,
        file: skill.relativeFile
      }))
    }
  };
}

function printReport(report) {
  console.log('Harness skill recommendations');
  console.log(`Skills: ${report.skillsRoot}`);
  console.log(`Query: ${report.query || '(none)'}`);
  console.log('');
  for (const skill of report.details.recommendations) {
    console.log(`- ${skill.name} (score ${skill.score}; matches: ${skill.matches.join(', ') || 'none'})`);
    if (skill.description) console.log(`  ${skill.description}`);
  }
}

try {
  const args = parseArgs(process.argv.slice(2));
  const report = buildReport(args);
  if (args.json) console.log(JSON.stringify(report, null, 2));
  else printReport(report);
  if (report.fail.length > 0) process.exit(1);
} catch (error) {
  console.error(`ERROR: ${error.message}`);
  process.exit(1);
}
