// 纯函数不变量探针：直接 bundle 真实 KnowledgeGraphView.tsx，不复刻算法。
// 用法：node graph-layout-invariant-probe.mjs   （cwd 任意；路径由 import.meta.url 推出）
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rawDirectory = new URL("./", import.meta.url);
const appRoot = "/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell";
const entry = path.join(appRoot, "src/views/knowledge/KnowledgeGraphView.tsx");
const outFile = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "r3e-graph-probe-")), "graph.mjs");

execFileSync(path.join(appRoot, "node_modules/.bin/esbuild"), [
  entry,
  "--bundle",
  "--platform=node",
  "--format=esm",
  "--target=node22",
  "--jsx=automatic",
  "--loader:.css=empty",
  `--outfile=${outFile}`,
], { stdio: ["ignore", "ignore", "inherit"] });

const {
  deterministicKnowledgeGraphPosition,
  knowledgeGraphMinZoom,
  knowledgeGraphLayoutRadius,
  knowledgeGraphRingPlan,
  knowledgeGraphRingCapacity,
  KNOWLEDGE_GRAPH_NODE_WIDTH,
  KNOWLEDGE_GRAPH_NODE_HEIGHT,
  KNOWLEDGE_GRAPH_NODE_PITCH,
  KNOWLEDGE_GRAPH_READABLE_ZOOM,
} = await import(outFile);

const sampled = [1, 2, 3, 5, 6, 7, 8, 12, 13, 20, 37, 40, 41, 61, 100, 129, 222, 341, 484, 485, 511, 512];
const report = {
  phase: "pure-function-invariant",
  nodeBox: { width: KNOWLEDGE_GRAPH_NODE_WIDTH, height: KNOWLEDGE_GRAPH_NODE_HEIGHT },
  requiredMinCenterDistance: KNOWLEDGE_GRAPH_NODE_PITCH,
  readableZoom: KNOWLEDGE_GRAPH_READABLE_ZOOM,
  checkedTotals: [],
  determinismChecked: 0,
  failures: [],
};

// 全量扫 1..512 的最小中心距（O(n^2) 但 512 上限可接受）
for (let total = 1; total <= 512; total += 1) {
  const points = Array.from({ length: total }, (_, index) => deterministicKnowledgeGraphPosition(index, total));
  let minDistance = Infinity;
  let worstPair = null;
  for (let i = 0; i < total; i += 1) {
    for (let j = i + 1; j < total; j += 1) {
      const dx = points[i].x - points[j].x;
      const dy = points[i].y - points[j].y;
      const distance = Math.hypot(dx, dy);
      if (distance < minDistance) {
        minDistance = distance;
        worstPair = [i, j];
      }
    }
  }
  const ok = total === 1 || minDistance >= KNOWLEDGE_GRAPH_NODE_PITCH;
  if (!ok) {
    report.failures.push({
      kind: "min_center_distance_below_pitch",
      total,
      minDistance: Math.round(minDistance * 100) / 100,
      worstPair,
    });
  }
  if (sampled.includes(total)) {
    const rings = knowledgeGraphRingPlan(total);
    report.checkedTotals.push({
      total,
      minCenterDistance: total === 1 ? null : Math.round(minDistance * 100) / 100,
      meetsPitch: ok,
      ringCount: rings.length,
      rings: rings.map((ring) => ({ radius: Math.round(ring.radius * 100) / 100, count: ring.count })),
      layoutRadius: Math.round(knowledgeGraphLayoutRadius(total) * 100) / 100,
      layoutExtent: {
        width: Math.round((2 * knowledgeGraphLayoutRadius(total) + KNOWLEDGE_GRAPH_NODE_WIDTH) * 100) / 100,
        height: Math.round((2 * knowledgeGraphLayoutRadius(total) + KNOWLEDGE_GRAPH_NODE_HEIGHT) * 100) / 100,
      },
      minZoom: Math.round(knowledgeGraphMinZoom(total) * 10000) / 10000,
    });
  }
}

// 确定性：同一 (index,total) 反复调用恒等
for (const total of sampled) {
  for (let index = 0; index < Math.min(total, 24); index += 1) {
    const first = deterministicKnowledgeGraphPosition(index, total);
    const second = deterministicKnowledgeGraphPosition(index, total);
    report.determinismChecked += 1;
    if (first.x !== second.x || first.y !== second.y) {
      report.failures.push({ kind: "non_deterministic", total, index, first, second });
    }
  }
}

// 环容量自洽：每环上相邻弦长 ≥ pitch
for (const total of sampled) {
  for (const ring of knowledgeGraphRingPlan(total)) {
    if (ring.count < 2) continue;
    const chord = 2 * ring.radius * Math.sin(Math.PI / ring.count);
    if (chord < KNOWLEDGE_GRAPH_NODE_PITCH - 1e-9) {
      report.failures.push({
        kind: "ring_chord_below_pitch",
        total,
        radius: ring.radius,
        count: ring.count,
        chord: Math.round(chord * 100) / 100,
        capacity: knowledgeGraphRingCapacity(ring.radius),
      });
    }
  }
}

report.outcome = report.failures.length === 0 ? "PASS_PURE_FUNCTION_INVARIANT" : "FAIL_PURE_FUNCTION_INVARIANT";
fs.writeFileSync(
  fileURLToPath(new URL("./graph-layout-invariant-probe.json", rawDirectory)),
  `${JSON.stringify(report, null, 2)}\n`,
  "utf8",
);
console.log(JSON.stringify({
  outcome: report.outcome,
  scannedTotals: "1..512 全量",
  determinismChecked: report.determinismChecked,
  failures: report.failures.slice(0, 5),
  sample: report.checkedTotals.filter((entry) => [1, 2, 6, 7, 12, 40, 100, 512].includes(entry.total)),
}, null, 2));
process.exitCode = report.failures.length === 0 ? 0 : 1;
