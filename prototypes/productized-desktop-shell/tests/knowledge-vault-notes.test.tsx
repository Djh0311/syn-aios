// L3 知识库第一片离线断言：受限 md 渲染器各语法 + 非支持语法逐字 + XSS 纯文本 +
// wikilink 命中/未命中 + 编辑保存 + 空态（包 §三.9/§十.4·2026-07-20）。
import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";
import { KnowledgeVaultNotesView } from "../src/views/KnowledgeBaseView";
import { extractWikilinks, parseMarkdown } from "../src/lib/knowledgeVault";
import type { KnowledgeVaultNote, KnowledgeVaultNoteSummary } from "../src/lib/tauri";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(`[knowledge-vault-notes] ${message}`);
  }
}

const noop = () => {};

// 1) 渲染器：ATX 标题/粗斜/行内码/围栏代码块/列表一层/外链/wikilink 节点树。
{
  const blocks = parseMarkdown(
    [
      "# 标题一",
      "",
      "段落 **粗** 与 *斜* 与 `码` 与 [[另一条笔记]] 与 https://example.com/x 。",
      "",
      "- 第一项",
      "- 第二项",
      "",
      "1. 甲",
      "2. 乙",
      "",
      "```",
      "const a = 1 < 2;",
      "```",
      "",
      "#### 四级",
    ].join("\n"),
  );
  assert(blocks[0].kind === "heading" && blocks[0].level === 1, "ATX 一级标题解析失败");
  const paragraph = blocks.find((block) => block.kind === "paragraph");
  assert(paragraph && paragraph.kind === "paragraph", "段落块缺失");
  const kinds = paragraph.inlines.map((inline) => inline.kind);
  assert(kinds.includes("bold") && kinds.includes("italic") && kinds.includes("code"), "粗/斜/行内码解析失败");
  assert(kinds.includes("wikilink") && kinds.includes("link"), "wikilink/外链解析失败");
  const lists = blocks.filter((block) => block.kind === "list");
  assert(lists.length === 2 && lists[0].kind === "list" && !lists[0].ordered && lists[1].kind === "list" && lists[1].ordered, "无序/有序列表解析失败");
  assert(lists[0].kind === "list" && lists[0].items.length === 2, "一层列表条数不对");
  const code = blocks.find((block) => block.kind === "code_block");
  assert(code && code.kind === "code_block" && code.text === "const a = 1 < 2;", "围栏代码块必须逐字保留");
  assert(blocks.some((block) => block.kind === "heading" && block.level === 4), "四级标题解析失败");
}

// 2) 非支持语法逐字：表格/图片/嵌套列表/HTML 一律纯文本原样。
{
  const blocks = parseMarkdown(["| a | b |", "", "![图](x.png)", "", "  - 嵌套项", "", "<div>raw</div>"].join("\n"));
  const text = JSON.stringify(blocks);
  assert(text.includes("| a | b |"), "表格必须逐字纯文本");
  assert(text.includes("![图](x.png)"), "图片语法必须逐字纯文本");
  assert(text.includes("  - 嵌套项"), "嵌套列表不进列表块（一层不嵌套）");
  assert(!blocks.some((block) => block.kind === "list"), "嵌套/无外层列表时不许造列表块");
  assert(text.includes("<div>raw</div>"), "HTML 必须逐字纯文本");
}

// 3) XSS 样例=纯文本输出（渲染 markup 不出现可执行标签）。
{
  const body = "x <script>alert(1)</script> y <img src=x onerror=alert(1)>";
  const markup = renderToStaticMarkup(<MarkdownProbe body={body} />);
  assert(!markup.includes("<script>"), "script 必须转义为纯文本");
  assert(markup.includes("&lt;script&gt;"), "script 样例应按纯文本转义显示");
  assert(!/<img\b/.test(markup), "img onerror 不得渲染为标签");
}

function MarkdownProbe({ body }: { body: string }) {
  const blocks = parseMarkdown(body);
  return (
    <>
      {blocks.map((block, index) => (
        <p key={index}>{JSON.stringify(block)}</p>
      ))}
    </>
  );
}

// 4) extractWikilinks 提取与去重。
{
  assertDeep(extractWikilinks("见 [[甲]] 与 [[乙]] 再 [[甲]]"), ["甲", "乙"], "wikilink 提取去重失败");
}

function assertDeep(actual: unknown, expected: unknown, message: string) {
  assert(JSON.stringify(actual) === JSON.stringify(expected), `${message} actual=${JSON.stringify(actual)}`);
}

// 5) wikilink 命中=打开该笔记（大小写不敏感）；未命中=出「新建《标题》」且用户点建才建。
const noteA: KnowledgeVaultNoteSummary = { slug: "alpha", title: "Alpha", mtime_ms: 1, outlinks: [] };
const noteB: KnowledgeVaultNoteSummary = { slug: "beta", title: "Beta", mtime_ms: 2, outlinks: ["Alpha"] };
const selectedB: KnowledgeVaultNote = { slug: "beta", title: "Beta", body: "# Beta\n\n回链 [[Alpha]] 与 [[Gamma]]。", mtime_ms: 2 };

{
  const opened: string[] = [];
  const created: string[] = [];
  const view = (
    <KnowledgeVaultNotesView
      loadState="ready"
      notes={[noteA, noteB]}
      selected={selectedB}
      editing={false}
      draft=""
      newTitle={null}
      pendingLinkTitle={null}
      onSelect={(slug) => opened.push(slug)}
      onStartNew={noop}
      onNewTitleChange={noop}
      onCreateNew={noop}
      onCancelNew={noop}
      onStartEdit={noop}
      onDraftChange={noop}
      onSaveEdit={noop}
      onCancelEdit={noop}
      onOpenLink={(title) => {
        const hit = [noteA, noteB].find((note) => note.title.toLowerCase() === title.toLowerCase());
        if (hit) opened.push(hit.slug);
        else created.push(title);
      }}
      onCreateFromLink={noop}
      onDismissLink={noop}
    />
  );
  const markup = renderToStaticMarkup(view);
  assert(markup.includes("knowledge-vault-wikilink"), "wikilink 必须渲染为可点链接");
  assert(markup.includes("回链"), "正文必须渲染");
  // 命中（大小写不敏感）
  const links = findWikilinkButtons(view);
  assert(links.length === 2, `应渲染 2 个 wikilink 按钮,实际 ${links.length}`);
  links[0].props.onClick();
  assertDeep(opened, ["alpha"], "命中 wikilink 必须按标题匹配打开笔记");
  // 未命中
  links[1].props.onClick();
  assertDeep(created, ["Gamma"], "未命中 wikilink 必须进新建问询面");
}

function findWikilinkButtons(root: React.ReactNode): { props: { onClick: () => void } }[] {
  const found: { props: { onClick: () => void } }[] = [];
  const walk = (node: React.ReactNode) => {
    if (!React.isValidElement(node)) return;
    const element = node as React.ReactElement & { type: unknown; props?: Record<string, unknown> };
    if (element.type === "button" && typeof element.props?.className === "string" && element.props.className.includes("knowledge-vault-wikilink")) {
      found.push(element as never);
    }
    if (typeof element.type === "function") {
      const rendered = (element.type as (props: Record<string, unknown>) => React.ReactNode)(element.props ?? {});
      walk(rendered);
    }
    React.Children.forEach(element.props?.children as React.ReactNode, walk);
  };
  walk(root);
  return found;
}

// 6) 未命中面：「新建《Gamma》」按钮在（用户那一下才建）。
{
  const markup = renderToStaticMarkup(
    <KnowledgeVaultNotesView
      loadState="ready"
      notes={[noteA, noteB]}
      selected={null}
      editing={false}
      draft=""
      newTitle={null}
      pendingLinkTitle="Gamma"
      onSelect={noop}
      onStartNew={noop}
      onNewTitleChange={noop}
      onCreateNew={noop}
      onCancelNew={noop}
      onStartEdit={noop}
      onDraftChange={noop}
      onSaveEdit={noop}
      onCancelEdit={noop}
      onOpenLink={noop}
      onCreateFromLink={noop}
      onDismissLink={noop}
    />,
  );
  assert(markup.includes("新建《Gamma》"), "未命中双链必须出「新建《标题》」按钮");
  assert(markup.includes("还不存在"), "未命中必须明说笔记不存在");
}

// 7) 编辑保存：textarea 改稿→保存走 onSaveEdit；取消弃改。
{
  let saved = 0;
  let cancelled = 0;
  const drafts: string[] = [];
  const view = (
    <KnowledgeVaultNotesView
      loadState="ready"
      notes={[noteA]}
      selected={{ slug: "alpha", title: "Alpha", body: "旧文", mtime_ms: 1 }}
      editing
      draft="旧文"
      newTitle={null}
      pendingLinkTitle={null}
      onSelect={noop}
      onStartNew={noop}
      onNewTitleChange={noop}
      onCreateNew={noop}
      onCancelNew={noop}
      onStartEdit={noop}
      onDraftChange={(value) => drafts.push(value)}
      onSaveEdit={() => {
        saved += 1;
      }}
      onCancelEdit={() => {
        cancelled += 1;
      }}
      onOpenLink={noop}
      onCreateFromLink={noop}
      onDismissLink={noop}
    />
  );
  const markup = renderToStaticMarkup(view);
  assert(markup.includes("<textarea"), "编辑态必须出 textarea");
  const saveButton = findButtonByText(view, "保存");
  saveButton.props.onClick();
  assert(saved === 1, "保存必须走 onSaveEdit 一次");
  const cancelButton = findButtonByText(view, "取消");
  cancelButton.props.onClick();
  assert(cancelled === 1, "取消必须走 onCancelEdit 一次");
}

function findButtonByText(root: React.ReactNode, text: string): { props: { onClick: () => void } } {
  const buttons = (function collect(node: React.ReactNode, acc: React.ReactElement[]): React.ReactElement[] {
    if (!React.isValidElement(node)) return acc;
    const element = node as React.ReactElement & { type: unknown; props?: Record<string, unknown> };
    if (element.type === "button") acc.push(element);
    if (typeof element.type === "function") {
      acc = collect((element.type as (props: Record<string, unknown>) => React.ReactNode)(element.props ?? {}), acc);
    }
    React.Children.forEach(element.props?.children as React.ReactNode, (child) => {
      acc = collect(child, acc);
    });
    return acc;
  })(root, []);
  const hit = buttons.find((button) => {
    const text2 = renderToStaticMarkup(button).replace(/<[^>]*>/g, "");
    return text2 === text;
  });
  assert(hit, `缺少按钮「${text}」`);
  return hit as never;
}

// 8) 空态=EmptyState 定式（必答下一步）；unavailable=说明只桌面壳可读写。
{
  const markup = renderToStaticMarkup(
    <KnowledgeVaultNotesView
      loadState="ready"
      notes={[]}
      selected={null}
      editing={false}
      draft=""
      newTitle={null}
      pendingLinkTitle={null}
      onSelect={noop}
      onStartNew={noop}
      onNewTitleChange={noop}
      onCreateNew={noop}
      onCancelNew={noop}
      onStartEdit={noop}
      onDraftChange={noop}
      onSaveEdit={noop}
      onCancelEdit={noop}
      onOpenLink={noop}
      onCreateFromLink={noop}
      onDismissLink={noop}
    />,
  );
  assert(markup.includes("vault 里还没有笔记"), "空态必须说实话");
  assert(markup.includes("新建笔记"), "空态必须给下一步");
  const unavailable = renderToStaticMarkup(
    <KnowledgeVaultNotesView
      loadState="unavailable"
      notes={[]}
      selected={null}
      editing={false}
      draft=""
      newTitle={null}
      pendingLinkTitle={null}
      onSelect={noop}
      onStartNew={noop}
      onNewTitleChange={noop}
      onCreateNew={noop}
      onCancelNew={noop}
      onStartEdit={noop}
      onDraftChange={noop}
      onSaveEdit={noop}
      onCancelEdit={noop}
      onOpenLink={noop}
      onCreateFromLink={noop}
      onDismissLink={noop}
    />,
  );
  assert(unavailable.includes("读不到"), "unavailable 必须说明读不到");
}

console.log("knowledge-vault-notes: 渲染器语法/逐字/XSS/wikilink 两态/编辑保存/空态断言全过");
