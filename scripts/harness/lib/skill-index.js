const fs = require('fs');
const path = require('path');

const BASELINE_KEYWORDS = {
  'using-superpowers': ['start', 'task', 'risk', 'path', 'scope', 'route', 'router'],
  brainstorming: ['brainstorm', 'ambiguous', 'feature', 'product', 'design', 'options', 'clarify'],
  'writing-plans': ['plan', 'multi-step', 'handoff', 'phases', 'implementation plan'],
  'executing-plans': ['execute plan', 'existing plan', 'checkpoint', 'phase'],
  'subagent-driven-development': ['subagent', 'agent', 'dispatch', 'task package'],
  'dispatching-parallel-agents': ['parallel', 'independent investigations', 'multiple agents'],
  'test-driven-development': ['test', 'tdd', 'behavior', 'regression', 'public api', 'data transformation', 'state flow'],
  'systematic-debugging': ['bug', 'failure', 'failing', 'runtime error', 'unexpected', 'root cause', 'production issue'],
  'learning-from-mistakes': ['mistake', 'wrong root cause', 'wrong fix', 'regression', 'scope violation'],
  'ui-browser-verification': ['ui', 'browser', 'frontend', 'visual', 'layout', 'responsive', 'interaction', 'screenshot'],
  'verification-before-completion': ['complete', 'done', 'ready', 'passing', 'verify', 'verification', 'evidence'],
  'evaluator-acceptance-review': ['strict', 'acceptance', 'evaluator', 'high-impact', 'production-facing'],
  'requesting-code-review': ['review', 'code review', 'major implementation', 'complex fix'],
  'receiving-code-review': ['review feedback', 'reviewer feedback', 'requested changes'],
  'using-git-worktrees': ['worktree', 'branch isolation', 'isolation'],
  'finishing-a-development-branch': ['finish branch', 'merge', 'pull request', 'pr', 'cleanup'],
  'writing-skills': ['skill', 'skills', 'skill.md', 'frontmatter']
};

const STOPWORDS = new Set([
  'when',
  'with',
  'where',
  'that',
  'this',
  'from',
  'before',
  'after',
  'work',
  'task',
  'user',
  'requires',
  'required',
  'implementation',
  'completion',
  'status',
  'project',
  'claiming',
  'available'
]);

function walk(dir, files = []) {
  if (!fs.existsSync(dir)) return files;
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full, files);
    else if (entry.isFile() && entry.name === 'SKILL.md') files.push(full);
  }
  return files;
}

function parseFrontmatter(text) {
  const metadata = {};
  const match = String(text || '').match(/^---\r?\n([\s\S]*?)\r?\n---\r?\n?/);
  if (!match) return { metadata, body: String(text || '') };

  for (const line of match[1].split(/\r?\n/)) {
    const pair = line.match(/^([A-Za-z0-9_-]+):\s*(.*)$/);
    if (pair) metadata[pair[1]] = pair[2].trim().replace(/^["']|["']$/g, '');
  }

  return {
    metadata,
    body: String(text || '').slice(match[0].length)
  };
}

function textWords(value) {
  return String(value || '')
    .toLowerCase()
    .replace(/[^a-z0-9._-]+/g, ' ')
    .split(/\s+/)
    .filter(Boolean);
}

function unique(values) {
  return Array.from(new Set(values.filter(Boolean)));
}

function readSkillFile(filePath, root) {
  const text = fs.readFileSync(filePath, 'utf8');
  const parsed = parseFrontmatter(text);
  const fallbackName = path.basename(path.dirname(filePath));
  const name = parsed.metadata.name || fallbackName;
  const heading = parsed.body.match(/^#\s+(.+)$/m);
  const description = parsed.metadata.description || '';
  const baseline = BASELINE_KEYWORDS[name] || [];
  const keywords = unique([
    ...baseline,
    ...textWords(name),
    ...textWords(description).filter((word) => word.length >= 4 && !STOPWORDS.has(word))
  ]);

  return {
    name,
    description,
    title: heading ? heading[1].trim() : name,
    file: filePath,
    relativeFile: root ? path.relative(root, filePath).split(path.sep).join('/') : filePath,
    keywords
  };
}

function loadSkillIndex(skillsRoot) {
  const root = path.resolve(skillsRoot);
  const files = walk(root).sort((a, b) => a.localeCompare(b));
  return files.map((file) => readSkillFile(file, root));
}

function scoreSkill(skill, query) {
  const haystack = String(query || '').toLowerCase();
  const words = new Set(textWords(query));
  const matches = [];
  let score = 0;

  for (const keyword of skill.keywords || []) {
    const normalized = String(keyword || '').toLowerCase().trim();
    if (!normalized) continue;
    const matched = normalized.includes(' ')
      ? haystack.includes(normalized)
      : words.has(normalized) || haystack.includes(normalized);
    if (!matched) continue;
    matches.push(normalized);
    score += normalized.includes(' ') ? 3 : 1;
  }

  return { score, matches: unique(matches).sort() };
}

function recommendSkills(skills, query, options = {}) {
  const limit = Number.isFinite(options.limit) ? options.limit : 8;
  const includeBaseline = options.includeBaseline !== false;
  const scored = [];

  for (const skill of skills) {
    const scoredSkill = scoreSkill(skill, query);
    let score = scoredSkill.score;
    const matches = [...scoredSkill.matches];
    if (includeBaseline && skill.name === 'using-superpowers') {
      score += 1;
      matches.push('baseline');
    }
    if (includeBaseline && skill.name === 'verification-before-completion') {
      score += 1;
      matches.push('baseline');
    }
    if (score > 0) scored.push({ ...skill, score, matches: unique(matches).sort() });
  }

  scored.sort((a, b) => b.score - a.score || a.name.localeCompare(b.name));
  return scored.slice(0, Math.max(1, limit));
}

module.exports = {
  BASELINE_KEYWORDS,
  loadSkillIndex,
  parseFrontmatter,
  recommendSkills,
  scoreSkill
};
