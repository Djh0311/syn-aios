// L3 知识库第一片：受限 markdown 渲染器（零新依赖·纯函数 parse→节点树）。
// 口径（任务包 §二.5）：ATX 标题 #~####、段落、**粗**/*斜*/`行内码`、``` 围栏代码块、
// `-`/`1.` 列表（一层不嵌套）、[[wikilink]]、https?:// 外链；其余语法一律纯文本逐字。
// 渲染层（KnowledgeBaseView）只用文本节点，禁 innerHTML。

export type MdInline =
  | { kind: "text"; text: string }
  | { kind: "bold"; text: string }
  | { kind: "italic"; text: string }
  | { kind: "code"; text: string }
  | { kind: "wikilink"; title: string }
  | { kind: "link"; url: string };

export type MdBlock =
  | { kind: "heading"; level: 1 | 2 | 3 | 4; inlines: MdInline[] }
  | { kind: "paragraph"; inlines: MdInline[] }
  | { kind: "code_block"; text: string }
  | { kind: "list"; ordered: boolean; items: MdInline[][] };

const INLINE_RE =
  /(\[\[([^\]]+)\]\])|(`([^`]+)`)|(\*\*([^*]+)\*\*)|(\*([^*]+)\*)|((https?):\/\/[^\s<>"')\]]+)/g;

export function parseInlines(text: string): MdInline[] {
  const segments: MdInline[] = [];
  let cursor = 0;
  for (const match of text.matchAll(INLINE_RE)) {
    const index = match.index ?? 0;
    if (index > cursor) segments.push({ kind: "text", text: text.slice(cursor, index) });
    if (match[2] !== undefined) {
      segments.push({ kind: "wikilink", title: match[2].trim() });
    } else if (match[4] !== undefined) {
      segments.push({ kind: "code", text: match[4] });
    } else if (match[6] !== undefined) {
      segments.push({ kind: "bold", text: match[6] });
    } else if (match[8] !== undefined) {
      segments.push({ kind: "italic", text: match[8] });
    } else if (match[9] !== undefined) {
      segments.push({ kind: "link", url: match[9] });
    }
    cursor = index + match[0].length;
  }
  if (cursor < text.length) segments.push({ kind: "text", text: text.slice(cursor) });
  return segments.filter((segment) => segment.kind !== "text" || segment.text.length > 0);
}

const HEADING_RE = /^(#{1,4})\s+(.*)$/;
const UNORDERED_RE = /^-\s+(.*)$/;
const ORDERED_RE = /^\d+\.\s+(.*)$/;
const FENCE_RE = /^```/;

export function parseMarkdown(body: string): MdBlock[] {
  const blocks: MdBlock[] = [];
  const lines = body.replace(/\r\n/g, "\n").split("\n");
  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    if (FENCE_RE.test(line)) {
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !FENCE_RE.test(lines[index])) {
        codeLines.push(lines[index]);
        index += 1;
      }
      index += 1; // 收尾围栏（缺收尾=到文末，逐字保留）
      blocks.push({ kind: "code_block", text: codeLines.join("\n") });
      continue;
    }
    if (line.trim() === "") {
      index += 1;
      continue;
    }
    const heading = line.match(HEADING_RE);
    if (heading) {
      blocks.push({
        kind: "heading",
        level: heading[1].length as 1 | 2 | 3 | 4,
        inlines: parseInlines(heading[2]),
      });
      index += 1;
      continue;
    }
    const unordered = line.match(UNORDERED_RE);
    const ordered = line.match(ORDERED_RE);
    if (unordered || ordered) {
      const isOrdered = Boolean(ordered);
      const items: MdInline[][] = [];
      while (index < lines.length) {
        const itemLine = lines[index];
        const itemMatch = isOrdered ? itemLine.match(ORDERED_RE) : itemLine.match(UNORDERED_RE);
        if (!itemMatch) break;
        items.push(parseInlines(itemMatch[1]));
        index += 1;
      }
      blocks.push({ kind: "list", ordered: isOrdered, items });
      continue;
    }
    // 段落：连续非空非块行合并（软换行=空格）；表格/图片/HTML 等其余语法落到此=逐字纯文本。
    const paragraphLines: string[] = [];
    while (
      index < lines.length &&
      lines[index].trim() !== "" &&
      !FENCE_RE.test(lines[index]) &&
      !HEADING_RE.test(lines[index]) &&
      !UNORDERED_RE.test(lines[index]) &&
      !ORDERED_RE.test(lines[index])
    ) {
      paragraphLines.push(lines[index]);
      index += 1;
    }
    blocks.push({ kind: "paragraph", inlines: parseInlines(paragraphLines.join(" ")) });
  }
  return blocks;
}

export function extractWikilinks(body: string): string[] {
  const links: string[] = [];
  for (const match of body.matchAll(/\[\[([^\]]+)\]\]/g)) {
    const title = match[1].trim();
    if (title && !links.includes(title)) links.push(title);
  }
  return links;
}
