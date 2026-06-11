import React from "react";
import { renderToStaticMarkup } from "react-dom/server.browser";

export type ReactElementLike = React.ReactElement & {
  type: unknown;
  props?: Record<string, unknown>;
};

export function findButtonByText(root: React.ReactNode, text: string): ReactElementLike | null {
  return findElement(root, (element) => element.type === "button" && visibleText(element).trim() === text);
}

export function findButtonContainingText(root: React.ReactNode, textParts: string[]): ReactElementLike | null {
  return findElement(
    root,
    (element) => element.type === "button" && textParts.every((textPart) => visibleText(element).includes(textPart)),
  );
}

export function findElement(
  root: React.ReactNode,
  predicate: (element: ReactElementLike) => boolean,
): ReactElementLike | null {
  if (!React.isValidElement(root)) return null;
  const element = root as ReactElementLike;
  if (predicate(element)) return element;

  const rendered = renderComposite(element);
  if (rendered !== element) {
    const match = findElement(rendered, predicate);
    if (match) return match;
  }

  const children = element.props?.children;
  const childArray = React.Children.toArray(children as React.ReactNode);
  for (const child of childArray) {
    const match = findElement(child, predicate);
    if (match) return match;
  }
  return null;
}

export function visibleText(root: React.ReactNode): string {
  if (root === null || root === undefined || typeof root === "boolean") return "";
  if (typeof root === "string" || typeof root === "number") return String(root);
  if (Array.isArray(root)) return root.map(visibleText).join("");
  if (!React.isValidElement(root)) return "";

  return renderToStaticMarkup(root)
    .replace(/<[^>]*>/g, "")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#x27;/g, "'")
    .replace(/&quot;/g, '"');
}

export function buttonTextsInMarkup(markup: string): string[] {
  return (markup.match(/<button\b[\s\S]*?<\/button>/g) ?? []).map((button) =>
    button
      .replace(/<[^>]*>/g, "")
      .replace(/&nbsp;/g, " ")
      .replace(/&amp;/g, "&")
      .replace(/&lt;/g, "<")
      .replace(/&gt;/g, ">")
      .replace(/&#x27;/g, "'")
      .replace(/&quot;/g, '"')
      .trim(),
  );
}

function renderComposite(element: ReactElementLike): React.ReactNode {
  if (typeof element.type !== "function") return element;
  const Component = element.type as (props: Record<string, unknown>) => React.ReactNode;
  return Component(element.props ?? {});
}

export function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

export function assertDeepEqual(actual: unknown, expected: unknown, message: string) {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(`${message}\nactual: ${actualJson}\nexpected: ${expectedJson}`);
  }
}
