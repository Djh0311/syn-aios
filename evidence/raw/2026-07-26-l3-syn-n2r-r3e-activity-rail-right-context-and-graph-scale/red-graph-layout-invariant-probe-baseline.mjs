// RED（纯函数）：直接 bundle §3.1 基线副本里的 KnowledgeGraphView，复算改动前的
// deterministicKnowledgeGraphPosition。不复刻算法，也不改任何仓内文件。

import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rawDirectory = new URL("./", import.meta.url);
const appRoot = "/Users/yoyi/workspace/product-line/prototypes/productized-desktop-shell";
const baselineEntry = fileURLToPath(new URL(
  "./baseline/prototypes/productized-desktop-shell/src/views/knowledge/KnowledgeGraphView.tsx",
  rawDirectory,
));
const outFile = path.join(fs.mkdtempSync(path.join(os.tmpdir(), "r3e-red-graph-probe-")), "graph.mjs");

// 基线副本在仓外，直接喂路径的话 esbuild 解析不到 react / @xyflow/react。
// 用 JS API 把副本内容按 stdin 送进去、resolveDir 指回真实源码目录：
// 内容仍是逐字节的基线，依赖解析走真实 node_modules。
const { build } = await import(path.join(appRoot, "node_modules/esbuild/lib/main.js"));
await build({
  stdin: {
    contents: fs.readFileSync(baselineEntry, "utf8"),
    resolveDir: path.join(appRoot, "src/views/knowledge"),
    sourcefile: "KnowledgeGraphView.baseline.tsx",
    loader: "tsx",
  },
  bundle: true,
  platform: "node",
  format: "esm",
  target: "node22",
  jsx: "automatic",
  loader: { ".css": "empty" },
  outfile: outFile,
  logLevel: "silent",
});

const { deterministicKnowledgeGraphPosition } = await import(outFile);

const NODE_WIDTH = 136;
const report = {
  phase: "pre-implementation-red-pure-function",
  source: "evidence baseline 副本（派发 SHA-256 c9151ffaf0ba406956ea2fd3ef40ff8d737d684578ecb263469eacea668dd781）",
  claim: "改动前半轴恒为 110/160、与 total 无关：节点数一增，最小中心距就掉到节点盒宽以下",
  nodeWidth: NODE_WIDTH,
  samples: [],
};

for (const total of [6, 7, 12, 40, 100, 512]) {
  const points = Array.from({ length: total }, (_, index) => deterministicKnowledgeGraphPosition(index, total));
  let minDistance = Infinity;
  let worstPair = null;
  for (let i = 0; i < total; i += 1) {
    for (let j = i + 1; j < total; j += 1) {
      const distance = Math.hypot(points[i].x - points[j].x, points[i].y - points[j].y);
      if (distance < minDistance) {
        minDistance = distance;
        worstPair = [i, j];
      }
    }
  }
  const xs = points.map((point) => point.x);
  const ys = points.map((point) => point.y);
  report.samples.push({
    total,
    minCenterDistance: Math.round(minDistance * 100) / 100,
    belowNodeWidth: minDistance < NODE_WIDTH,
    worstPair,
    worstPairPoints: worstPair ? [points[worstPair[0]], points[worstPair[1]]] : null,
    duplicateCoordinateCount: total - new Set(points.map(({ x, y }) => `${x}:${y}`)).size,
    layoutExtent: { width: Math.max(...xs) - Math.min(...xs), height: Math.max(...ys) - Math.min(...ys) },
  });
}

report.outcome = report.samples.some((sample) => sample.belowNodeWidth) ? "RED_ESTABLISHED" : "RED_NOT_ESTABLISHED";
fs.writeFileSync(
  fileURLToPath(new URL("./red-graph-layout-invariant-probe-baseline.json", rawDirectory)),
  `${JSON.stringify(report, null, 2)}\n`,
  "utf8",
);
console.log(JSON.stringify(report, null, 2));
