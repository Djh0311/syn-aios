'use strict';

// Adaptive Harness v0.5 — 写面与独占资源的相交判据（AH-050-07）
//
// 需求溯源：
//   GIT-2  排队与否只看写面重不重叠，不看在办的有几件；拒绝的理由必须是内容冲突
//   GIT-1  同一个工作副本同一时刻最多一件正在执行的事（唯一硬约束，不因不相交而放宽）
//   §6.4   并行与串行的判据（本文件就是该节的实现）
//   D3     串行槽已被否决：本文件里没有任何以「在办的有几件」为形式的总闸
//
// 本文件是纯函数、零 IO。判定的**全部输入**只有各任务自己的原始声明；
// 同一批声明喂进来，结果与在册条目的多少无关——把互不相交的声明从 0 条加到
// N 条，第 N+1 个请求的结论逐字不变。
//
// 三条只要漏一条，串行就会以别的名义复活：
//   1. 拒绝必须携带非空、可回查的重叠项（对方编号 + 双方哪条声明 + 具体重叠路径）；
//   2. 声明的默认值不得恒相交（不默认全仓、不默认省略、不默认通配）；
//   3. 控制面路径两侧同时豁免（只豁免一侧会让第二件事永远提交不了）。

// ---------------------------------------------------------------------------
// 控制面豁免：有限、明示、不外溢
// ---------------------------------------------------------------------------

// 每个叶子都必然写到的控制面位置。在办平面挂在 git common dir，根本不是仓内路径；
// 仓内只剩历史平面这一个受管根。清单是有限且明示的，**不得外溢**，
// 也**不授予对产品路径的写权限**——它只是把「大家都要写的那一格」从判据里摘出去。
const CONTROL_PLANE_EXEMPT_PREFIXES = Object.freeze([
  'docs/harness/history',
]);

// 判据与越界检查必须用同一份豁免清单。只在一侧豁免，第二件事永远交不了差。
//
// 豁免只**向下**成立：一条路径必须整个落在豁免前缀**之内**才算控制面。
// 这里绝不能写成「与豁免前缀重叠」——那样 `docs`、`docs/harness` 这些
// 豁免前缀的**祖先目录**也会被认成控制面，于是：
//   * 两个任务都声明 `docs` 时判为不相交（假放行）；
//   * 提交时点名 `docs/harness` 可以绕开写面与禁区检查（豁免外溢成写权限）。
// 豁免是把「大家都要写的那一格」摘出去，不是把它上面的整棵树都摘出去。
function isControlPlanePath(candidate, exemptPrefixes) {
  const list = Array.isArray(exemptPrefixes) ? exemptPrefixes : CONTROL_PLANE_EXEMPT_PREFIXES;
  return list.some((prefix) => pathWithin(candidate, prefix));
}

// ---------------------------------------------------------------------------
// 路径前缀重叠：前缀必须落在目录边界上
// ---------------------------------------------------------------------------

/**
 * 把一条声明写法归一成同一套路径语法，再拿去比。
 *
 * 这里归一的是**语法**（分隔符、`.`、`..`、首尾斜杠），不是**粒度**——
 * 绝不把 `src/lib/a.ts` 上卷成 `src/lib/`。粒度一上卷就再也给不出见证路径了。
 *
 * 语法不归一同样致命：`/src/a`、`src//a`、`src/./a`、`src/x/../a` 指的都是
 * `src/a`，但字符串前缀比对全都判成不相交——一条被污染的声明就能让判据整个失明。
 */
function normalizePath(value) {
  const raw = String(value === null || value === undefined ? '' : value).trim().replace(/\\/g, '/');
  if (raw === '') return '';
  const segments = [];
  let escaped = 0;
  for (const segment of raw.split('/')) {
    if (segment === '' || segment === '.') continue;
    if (segment === '..') {
      if (segments.length > 0) segments.pop();
      else escaped += 1;
      continue;
    }
    segments.push(segment);
  }
  const body = segments.join('/');
  // 爬到仓库根之上的写法原样留着 `..` 前缀：它跟仓内任何路径都不会相等，
  // 也就不会被误判成落在某个前缀之内。
  if (escaped > 0) return `${'../'.repeat(escaped)}${body}`.replace(/\/+$/, '');
  return body;
}

// 「声明为全仓」的各种写法。它们视为与一切相交（§6.4），
// 但正因如此，自动生成时**绝不允许**默认填成这些值。
const WHOLE_REPO_TOKENS = Object.freeze(['', '.', '/', '*', '**', '**/*', './']);

function isWholeRepoToken(value) {
  const text = normalizePath(value);
  return WHOLE_REPO_TOKENS.includes(text) || text === '*' || text === '**';
}

/**
 * 两条声明是否按目录边界重叠。返回一条**同时落在两条声明之内**的具体路径，
 * 不重叠返回 null。
 *
 * `src/a` 与 `src/a/b` 重叠（见证路径 `src/a/b`）；
 * `src/a` 与 `src/ab` 不重叠——前缀必须落在目录边界上，不是字符串前缀。
 *
 * 判定**以两份原始声明为准**：这里既不上卷到公共父目录，也不归一到统一粒度。
 * 上卷之后就给不出这条见证路径了，而给不出见证路径的相交结论一律不成立。
 */
function pathsOverlapWitness(left, right) {
  const a = normalizePath(left);
  const b = normalizePath(right);
  if (a === '' || b === '') return null;
  if (a === b) return a;
  if (b.startsWith(`${a}/`)) return b;
  if (a.startsWith(`${b}/`)) return a;
  return null;
}

/** 一条路径是否落在某个前缀之内（同样按目录边界）。越界检查用它。 */
function pathWithin(candidate, prefix) {
  const target = normalizePath(candidate);
  const base = normalizePath(prefix);
  if (target === '' || base === '') return false;
  return target === base || target.startsWith(`${base}/`);
}

function pathWithinAny(candidate, prefixes) {
  const list = Array.isArray(prefixes) ? prefixes : [];
  return list.some((prefix) => pathWithin(candidate, prefix));
}

// ---------------------------------------------------------------------------
// 声明：默认值不得恒相交
// ---------------------------------------------------------------------------

const DECLARATION_FIELDS = Object.freeze(['write-scope', 'forbidden-scope', 'exclusive-resources']);

function listOf(value) {
  if (Array.isArray(value)) return value.map((item) => String(item).trim()).filter((item) => item !== '');
  return null;
}

/**
 * 把一份任务声明读成判定输入。
 *
 * **不做任何默认填充。** 缺字段就是缺字段，通配就是通配，全仓就是全仓——
 * 三者都会被 declarationDefects 指名。自动生成器生成不出可用范围时，
 * 应当让用户在开工请求里补齐，而不是替他填一个「与一切相交」的值：
 * 输入被污染，判据再对也等于全串行。
 */
function readDeclaration(source) {
  const raw = source && typeof source === 'object' ? source : {};
  return {
    id: typeof raw.id === 'string' && raw.id.trim() !== '' ? raw.id.trim() : null,
    worktree: typeof raw.worktree === 'string' && raw.worktree.trim() !== '' ? raw.worktree.trim() : null,
    'write-scope': listOf(raw['write-scope'] !== undefined ? raw['write-scope'] : raw.writeScope),
    'forbidden-scope': listOf(raw['forbidden-scope'] !== undefined ? raw['forbidden-scope'] : raw.forbiddenScope),
    'exclusive-resources': listOf(raw['exclusive-resources'] !== undefined
      ? raw['exclusive-resources']
      : raw.exclusiveResources),
  };
}

/**
 * 声明本身的毛病。**每一项都会让这份声明与一切相交**，所以必须当场指名，
 * 不能默默放过——放过一次，后面所有判定都是假的。
 */
function declarationDefects(declaration) {
  const defects = [];
  for (const field of DECLARATION_FIELDS) {
    if (declaration[field] === null) {
      defects.push({
        code: 'DECLARATION_FIELD_MISSING',
        field,
        message: `${field} 字段缺失；缺字段按与一切相交处理，请在开工请求里补齐（可显式写成空表）`,
      });
    }
  }
  const writeScope = declaration['write-scope'];
  if (Array.isArray(writeScope)) {
    if (writeScope.length === 0) {
      defects.push({
        code: 'WRITE_SCOPE_EMPTY',
        field: 'write-scope',
        message: 'write-scope 不得为空表；写不出可用范围时请在开工请求里补齐',
      });
    }
    for (const entry of writeScope) {
      if (isWholeRepoToken(entry)) {
        defects.push({
          code: 'WRITE_SCOPE_WHOLE_REPO',
          field: 'write-scope',
          message: `write-scope 条目「${entry}」等于声明全仓，视为与一切相交；请改成具体目录`,
        });
      }
    }
  }
  for (const field of ['forbidden-scope', 'exclusive-resources']) {
    const value = declaration[field];
    if (!Array.isArray(value)) continue;
    for (const entry of value) {
      if (field === 'forbidden-scope' && isWholeRepoToken(entry)) {
        defects.push({
          code: 'FORBIDDEN_SCOPE_WHOLE_REPO',
          field,
          message: `forbidden-scope 条目「${entry}」等于整仓，不是显式列举的有限集合`,
        });
      }
      if (field === 'exclusive-resources' && (entry === '*' || entry === 'all' || entry === 'ALL')) {
        defects.push({
          code: 'EXCLUSIVE_RESOURCE_WILDCARD',
          field,
          message: `exclusive-resources 条目「${entry}」是通配标识，视为与一切相交`,
        });
      }
    }
  }
  return defects;
}

// ---------------------------------------------------------------------------
// 参与判定的声明集合
// ---------------------------------------------------------------------------

// 只有**显式列举**的禁区参与相交判定。以「write-scope 补集」形式表达的禁区不参与，
// 只在越界检查里生效——否则每份禁区都盖住别人全部写面，两两恒相交。
//
// 禁区**双向**参与的含义：一份禁区是「这批路径在我干活期间必须保持不动」的主张，
// 因此 A 的写面碰上 B 的禁区、B 的写面碰上 A 的禁区都算冲突；
// 两份禁区之间不算——双方都只要求别动，本来就不冲突。
function surfacesOf(declaration) {
  return [
    { field: 'write-scope', entries: declaration['write-scope'] || [] },
    { field: 'forbidden-scope', entries: declaration['forbidden-scope'] || [] },
  ];
}

function comparablePair(ownField, otherField) {
  if (ownField === 'forbidden-scope' && otherField === 'forbidden-scope') return false;
  return true;
}

/**
 * 两份**原始声明**之间的全部重叠项，逐项列出，每项都能回查：
 * 对方任务编号、双方各是哪条声明、以及一条同时落在两条声明之内的具体路径或资源标识。
 *
 * 这就是拒绝时必须携带的清单。拿不出这张清单的拒绝一律不合格，无论理由怎么措辞。
 */
function overlapItems(own, other, options) {
  const settings = options || {};
  const exempt = settings.controlPlaneExempt || CONTROL_PLANE_EXEMPT_PREFIXES;
  const items = [];

  const ownDefects = declarationDefects(own);
  const otherDefects = declarationDefects(other);
  const ownIntersectsAll = ownDefects.length > 0;
  const otherIntersectsAll = otherDefects.length > 0;

  for (const ownSurface of surfacesOf(own)) {
    for (const otherSurface of surfacesOf(other)) {
      if (!comparablePair(ownSurface.field, otherSurface.field)) continue;
      for (const ownEntry of ownSurface.entries) {
        if (isControlPlanePath(ownEntry, exempt)) continue;
        for (const otherEntry of otherSurface.entries) {
          if (isControlPlanePath(otherEntry, exempt)) continue;
          const witness = pathsOverlapWitness(ownEntry, otherEntry);
          if (witness === null) continue;
          if (isControlPlanePath(witness, exempt)) continue;
          items.push({
            kind: 'PATH',
            otherTaskId: other.id,
            ownField: ownSurface.field,
            ownDeclaration: ownEntry,
            otherField: otherSurface.field,
            otherDeclaration: otherEntry,
            witness,
          });
        }
      }
    }
  }

  // 一侧声明为全仓或缺字段时，与对方每一条声明都相交。
  // 见证路径取对方那条原始声明——它确实同时落在两份声明之内，照样回查得到。
  //
  // 这类重叠项**不是**两条具体路径撞上了，回查的落点是「那一侧的声明本身有毛病」。
  // 所以每一项都带上 basis 与 defects：拿着它去看 wideDeclaration 那份声明，
  // 能逐字看到缺的是哪个字段、或哪一条写成了全仓。凭空捏一个重叠项在这里不成立。
  if (ownIntersectsAll || otherIntersectsAll) {
    const wide = ownIntersectsAll ? own : other;
    const wideDefects = ownIntersectsAll ? ownDefects : otherDefects;
    const narrow = ownIntersectsAll ? other : own;
    for (const surface of surfacesOf(narrow)) {
      for (const entry of surface.entries) {
        if (isControlPlanePath(entry, exempt)) continue;
        items.push({
          kind: 'PATH',
          otherTaskId: other.id,
          ownField: ownIntersectsAll ? 'write-scope' : surface.field,
          ownDeclaration: ownIntersectsAll ? '（声明为全仓或缺字段）' : entry,
          otherField: ownIntersectsAll ? surface.field : 'write-scope',
          otherDeclaration: ownIntersectsAll ? entry : '（声明为全仓或缺字段）',
          witness: normalizePath(entry),
          wideDeclaration: wide.id,
          basis: 'DECLARATION_INTERSECTS_ALL',
          defects: wideDefects.map((defect) => ({ code: defect.code, field: defect.field })),
        });
      }
    }
  }

  const ownResources = own['exclusive-resources'] || [];
  const otherResources = other['exclusive-resources'] || [];
  for (const ownResource of ownResources) {
    for (const otherResource of otherResources) {
      if (ownResource !== otherResource) continue;
      items.push({
        kind: 'RESOURCE',
        otherTaskId: other.id,
        ownField: 'exclusive-resources',
        ownDeclaration: ownResource,
        otherField: 'exclusive-resources',
        otherDeclaration: otherResource,
        witness: ownResource,
      });
    }
  }

  return items;
}

/**
 * 回查一条重叠项：见证路径必须真的同时落在两份**原始**声明之内。
 * 回查不过的重叠项不得出现在拒绝里——那种拒绝跟计数式阻断没有区别。
 */
function verifyOverlapItem(item, own, other) {
  if (!item || typeof item !== 'object') return false;
  if (item.kind === 'RESOURCE') {
    return (own['exclusive-resources'] || []).includes(item.witness)
      && (other['exclusive-resources'] || []).includes(item.witness);
  }
  const ownEntries = (own['write-scope'] || []).concat(own['forbidden-scope'] || []);
  const otherEntries = (other['write-scope'] || []).concat(other['forbidden-scope'] || []);
  const ownWide = declarationDefects(own).length > 0;
  const otherWide = declarationDefects(other).length > 0;
  const inOwn = ownWide || ownEntries.some((entry) => pathWithin(item.witness, entry));
  const inOther = otherWide || otherEntries.some((entry) => pathWithin(item.witness, entry));
  return inOwn && inOther;
}

// ---------------------------------------------------------------------------
// 开工准入判定
// ---------------------------------------------------------------------------

const REFUSAL_CODES = Object.freeze({
  DECLARATION_UNUSABLE: 'DECLARATION_UNUSABLE',
  SAME_WORKING_COPY: 'SAME_WORKING_COPY',
  WRITE_SURFACE_OVERLAP: 'WRITE_SURFACE_OVERLAP',
  EXCLUSIVE_RESOURCE_OVERLAP: 'EXCLUSIVE_RESOURCE_OVERLAP',
});

/**
 * 开工准入。**每一次进入 ACTIVE 都要重跑这一次判定**——新开工、从暂停恢复、
 * 承接转交、接管半成品，一律重判。只在首次开工判一次的实现不合格。
 *
 * @param {object} input
 *   input.request     本次开工请求的声明（含 id / worktree / 三份声明）
 *   input.registered  在册声明数组；每项额外带 participates 布尔
 *   input.controlPlaneExempt  控制面豁免前缀（默认用本模块的有限清单）
 *
 * 判定只遍历 registered 里**参与判定**的那些声明，两两求交。
 * 遍历顺序、集合大小都不进结论：互不相交的声明再多，结论也是放行。
 */
function decideAdmission(input) {
  const settings = input || {};
  const request = readDeclaration(settings.request);
  const exempt = settings.controlPlaneExempt || CONTROL_PLANE_EXEMPT_PREFIXES;
  const registered = Array.isArray(settings.registered) ? settings.registered : [];

  const refusals = [];
  const overlaps = [];

  const ownDefects = declarationDefects(request);
  if (ownDefects.length > 0) {
    refusals.push({
      code: REFUSAL_CODES.DECLARATION_UNUSABLE,
      message: '本次开工请求的声明不可用，判据拿不到有效输入',
      defects: ownDefects,
    });
  }

  for (const entry of registered) {
    // 尚未取得分支与工作副本的节点不参与判定：刚建好的一批叶子若互相预占写面，
    // 谁都起不来。声明退出判定则要求任务已进入终态**且**分支/工作副本已有处置。
    if (entry && entry.participates === false) continue;
    const other = readDeclaration(entry);
    if (other.id !== null && request.id !== null && other.id === request.id) continue;

    // GIT-1 的唯一硬约束：同一个工作副本里已有一件在执行，第二件必须停。
    // 这条拒绝携带的是工作副本的 realpath 与对方编号，不是别的东西；
    // 它只有在请求确实指向同一个工作副本时才成立，写面不相交也不放宽。
    if (request.worktree && other.worktree && request.worktree === other.worktree) {
      refusals.push({
        code: REFUSAL_CODES.SAME_WORKING_COPY,
        message: `同一个工作副本 ${request.worktree} 里已经有一件在执行（${other.id}）；请为本次开工另开一个独立工作副本`,
        otherTaskId: other.id,
        worktree: request.worktree,
      });
      continue;
    }

    const items = overlapItems(request, other, { controlPlaneExempt: exempt })
      .filter((item) => verifyOverlapItem(item, request, other));
    if (items.length === 0) continue;
    overlaps.push(...items);
  }

  const pathItems = overlaps.filter((item) => item.kind === 'PATH');
  const resourceItems = overlaps.filter((item) => item.kind === 'RESOURCE');
  if (pathItems.length > 0) {
    refusals.push({
      code: REFUSAL_CODES.WRITE_SURFACE_OVERLAP,
      message: '写面与在册声明相交，逐项列出重叠对象如下',
      items: pathItems,
    });
  }
  if (resourceItems.length > 0) {
    refusals.push({
      code: REFUSAL_CODES.EXCLUSIVE_RESOURCE_OVERLAP,
      message: '独占资源与在册声明相交，逐项列出重叠对象如下',
      items: resourceItems,
    });
  }

  // 不相交就放行：判据两侧都要有承载。只写拒绝侧的实现在这里必然给不出 admitted=true。
  const admitted = refusals.length === 0;
  return {
    admitted,
    allowed: admitted,
    refusals,
    overlaps,
    // 依赖顺序是**执行顺序建议**，不是锁：写面与独占资源都不相交时，
    // 即使存在依赖声明也必须放行，拒绝理由里也不得写依赖先后。
    advisoryOrder: Array.isArray(settings.dependsOn) ? settings.dependsOn.slice() : [],
  };
}

/** 人话版拒绝文本，逐项把重叠对象写出来，方便回查。 */
function describeRefusals(refusals) {
  const list = Array.isArray(refusals) ? refusals : [];
  const lines = [];
  for (const refusal of list) {
    lines.push(`${refusal.code}：${refusal.message}`);
    for (const item of refusal.items || []) {
      lines.push(item.kind === 'RESOURCE'
        ? `  - 对方 ${item.otherTaskId}：双方 ${item.ownField} 都声明了资源「${item.witness}」`
        : `  - 对方 ${item.otherTaskId}：我方 ${item.ownField}「${item.ownDeclaration}」× 对方 ${item.otherField}「${item.otherDeclaration}」，重叠于 ${item.witness}`);
    }
    for (const defect of refusal.defects || []) {
      lines.push(`  - ${defect.field}：${defect.message}`);
    }
  }
  return lines.join('\n');
}

// ---------------------------------------------------------------------------
// 越界与回填（§6.4 的三行表）
// ---------------------------------------------------------------------------

/**
 * 实际改动路径与声明不符时怎么办。
 * 校验基准是开工时冻结的声明加上此前已生效的合法回填；
 * 本次改动**不得**先并进声明再判定。
 */
function classifyOutOfScopePaths(input) {
  const settings = input || {};
  const changed = Array.isArray(settings.changedPaths) ? settings.changedPaths : [];
  const own = readDeclaration(settings.declaration);
  const exempt = settings.controlPlaneExempt || CONTROL_PLANE_EXEMPT_PREFIXES;
  const registered = Array.isArray(settings.registered) ? settings.registered : [];

  const inForbidden = [];
  const collidingWithOthers = [];
  const backfillable = [];

  for (const filePath of changed) {
    // 控制面路径既不参与相交判定，也不参与越界判定。只在一侧豁免会让
    // 第二件事永远交不了差——两边必须用同一份清单。
    if (isControlPlanePath(filePath, exempt)) continue;
    if (pathWithinAny(filePath, own['forbidden-scope'] || [])) {
      inForbidden.push({ path: filePath, reason: '落在本任务显式禁区内，不得回填，也不因无人占用而放行' });
      continue;
    }
    if (pathWithinAny(filePath, own['write-scope'] || [])) continue;
    const collisions = [];
    for (const entry of registered) {
      if (entry && entry.participates === false) continue;
      const other = readDeclaration(entry);
      if (other.id !== null && own.id !== null && other.id === own.id) continue;
      for (const surface of surfacesOf(other)) {
        for (const declared of surface.entries) {
          if (pathWithin(filePath, declared)) {
            collisions.push({ otherTaskId: other.id, otherField: surface.field, otherDeclaration: declared });
          }
        }
      }
    }
    if (collisions.length > 0) {
      collidingWithOthers.push({ path: filePath, collisions });
      continue;
    }
    backfillable.push({ path: filePath });
  }

  return {
    // 落在禁区：拒绝提交，不回填。
    inForbidden,
    // 越界且与他人在册声明相交：拒绝提交、显式停机、不产生提交对象。
    collidingWithOthers,
    // 越界但既不在禁区又与谁都不相交：允许，并把实际范围回填进声明。
    backfillable,
    refused: inForbidden.length > 0 || collidingWithOthers.length > 0,
  };
}

/** 回填：把合法的越界路径并进 write-scope，立即对后续判定生效。 */
function backfillWriteScope(declaration, backfillable) {
  const own = readDeclaration(declaration);
  const next = (own['write-scope'] || []).slice();
  for (const entry of Array.isArray(backfillable) ? backfillable : []) {
    const filePath = normalizePath(entry && entry.path);
    if (filePath === '') continue;
    if (next.some((prefix) => pathWithin(filePath, prefix))) continue;
    next.push(filePath);
  }
  return next.sort();
}

module.exports = {
  CONTROL_PLANE_EXEMPT_PREFIXES,
  DECLARATION_FIELDS,
  REFUSAL_CODES,
  WHOLE_REPO_TOKENS,
  isControlPlanePath,
  normalizePath,
  isWholeRepoToken,
  pathsOverlapWitness,
  pathWithin,
  pathWithinAny,
  readDeclaration,
  declarationDefects,
  surfacesOf,
  overlapItems,
  verifyOverlapItem,
  decideAdmission,
  describeRefusals,
  classifyOutOfScopePaths,
  backfillWriteScope,
};
