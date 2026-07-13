#!/usr/bin/env python3
# audit_events 撞号历史修复脚本(任务包 2026-07-13-audit-events-collision-data-repair-package-v1)
# 用法: repair_script.py check | apply | post
# 红线:只动 workflow-state.v0.json;revision/meta/其余记录不动;意外即 sys.exit(非0)。
import json, sys, os, re, hashlib, collections

ROOT = os.path.expanduser('~/Library/Application Support/CodexGovernanceWorkbench/workflow-state')
MAIN = os.path.join(ROOT, 'workflow-state.v0.json')
EV = os.path.dirname(os.path.abspath(__file__))
WINDOW_START_HASH = 'bf3e6f473c05e9b67adfd4f3135b2541503e76183dc0f5a8648ec2b636d8846d'

def sha(b): return hashlib.sha256(b).hexdigest()
def canonical(rec): return json.dumps(rec, sort_keys=True, ensure_ascii=False)

def norm_nc(s): return re.sub(r'[^a-z0-9]', '-', s.lower())        # 逐字符归一(不折叠)
def norm_c(s):  return re.sub(r'[^a-z0-9]+', '-', s.lower())       # 折叠连续
NORMS = [('nocollapse', norm_nc), ('collapse', norm_c)]

def load():
    raw = open(MAIN, 'rb').read()
    return raw, json.loads(raw.decode('utf-8'))

def groups(m):
    by = collections.defaultdict(list)
    for i, e in enumerate(m.get('audit_events', [])):
        k = e.get('event_id')
        if k: by[k].append(i)
    return {k: v for k, v in by.items() if len(v) > 1}

def old_schema(m):
    return [i for i, e in enumerate(m.get('audit_events', []))
            if not e.get('event_id') and e.get('audit_event_id')]

def fidelity(raw, m):
    for tag, cand in [('indent2', json.dumps(m, indent=2, ensure_ascii=False)),
                      ('indent2+nl', json.dumps(m, indent=2, ensure_ascii=False) + '\n')]:
        if cand.encode('utf-8') == raw: return tag
    return None

def plan(m):
    ev = m['audit_events']
    all_ids = collections.Counter(e.get('event_id') for e in ev if e.get('event_id'))
    mapping, methods = [], collections.Counter()
    for old_id, idxs in sorted(groups(m).items()):
        parts = old_id.split(':')
        if len(parts) != 4 or parts[0] != 'audit':
            sys.exit(f'STOP unexpected id shape: {old_id}')
        kind_slug, mid, ts = parts[1], parts[2], parts[3]
        for i in idxs:
            rec = ev[i]
            raws = []
            if kind_slug == 'authorized-prepared-dispatch-created':
                p = rec.get('project_director_planned_task_id')
                if p: raws.append(f'authorized-prepared-dispatch:{p}:{ts}')
            elif kind_slug == 'project-director-task-plan-created':
                tr = rec.get('target_ref') or ''
                raws += [tr, tr.split(':', 1)[-1]]
            new_id, method = None, None
            for raw_src in raws:
                for ntag, fn in NORMS:
                    if fn(raw_src)[:96] == mid:               # 等式核验(法证同式)
                        new_id = f'audit:{kind_slug}:{fn(raw_src)}:{ts}'
                        method = f'detrunc/{ntag}'
                        break
                if new_id: break
            if not new_id:                                     # 死规则兜底
                new_id = f'{old_id}:{sha(canonical(rec).encode())[:12]}'
                method = 'sha-fallback'
            mapping.append({'index': i, 'old_id': old_id, 'new_id': new_id,
                            'kind': kind_slug, 'method': method})
            methods[method] += 1
    news = [x['new_id'] for x in mapping]
    if len(set(news)) != len(news): sys.exit('STOP new ids collide internally')
    for n in news:
        if all_ids.get(n): sys.exit(f'STOP new id already exists: {n}')
    return mapping, methods

def main():
    mode = sys.argv[1] if len(sys.argv) > 1 else 'check'
    raw, m = load()
    if mode == 'check':
        g = groups(m); tot = sum(len(v) for v in g.values())
        print(f'live hash={sha(raw)[:16]} match_window_start={sha(raw)==WINDOW_START_HASH}')
        print(f'events={len(m["audit_events"])} collision_groups={len(g)} records={tot}')
        print(f'old_schema_idx={old_schema(m)}')
        print(f'fidelity={fidelity(raw, m)}')
        mp, methods = plan(m)
        print(f'plan: {len(mp)} renames, methods={dict(methods)}')
        return
    if mode == 'apply':
        if sha(raw) != WINDOW_START_HASH: sys.exit('STOP store changed since window start')
        fid = fidelity(raw, m)
        mp, methods = plan(m)
        ev = m['audit_events']
        for x in mp: ev[x['index']]['event_id'] = x['new_id']
        osch = old_schema(m)
        for i in osch: ev[i]['event_id'] = ev[i]['audit_event_id']
        out = (json.dumps(m, indent=2, ensure_ascii=False) + ('\n' if fid == 'indent2+nl' else '')) \
              if fid else json.dumps(m, indent=2, ensure_ascii=False)
        tmp = MAIN + '.repair-tmp'
        with open(tmp, 'w', encoding='utf-8') as f: f.write(out)
        json.load(open(tmp))                                   # 合法性再核
        os.replace(tmp, MAIN)
        json.dump({'fidelity_mode': fid or 'semantic', 'renames': mp,
                   'old_schema_added_event_id': osch, 'methods': dict(methods)},
                  open(os.path.join(EV, 'mapping.json'), 'w'), ensure_ascii=False, indent=2)
        print(f'applied: {len(mp)} renames + {len(osch)} field-adds; fidelity={fid or "semantic"}')
        return
    if mode == 'post':
        before = json.load(open(os.path.join(EV, 'backup-before', 'workflow-state.v0.json')))
        g = groups(m)
        print(f'post: events={len(m["audit_events"])} (before={len(before["audit_events"])}) dup_groups={len(g)}')
        mp = json.load(open(os.path.join(EV, 'mapping.json')))
        renamed = {x['index'] for x in mp['renames']}
        added = set(mp['old_schema_added_event_id'])
        bad = 0
        for i, (a, b) in enumerate(zip(before['audit_events'], m['audit_events'])):
            da = {k: v for k, v in a.items()}; db = {k: v for k, v in b.items()}
            if i in renamed:
                da.pop('event_id'); db.pop('event_id')
            if i in added:
                db.pop('event_id')
            if canonical(da) != canonical(db):
                bad += 1; print(f'  UNEXPECTED diff at [{i}]')
        top_same = all(canonical(before[k]) == canonical(m[k]) if isinstance(before.get(k),(dict,list))
                       else before.get(k) == m.get(k)
                       for k in before.keys() if k != 'audit_events')
        print(f'per-record unexpected diffs={bad}; other top-level fields identical={top_same}')
        sys.exit(0 if (bad == 0 and len(g) == 0 and top_same) else 1)

main()
