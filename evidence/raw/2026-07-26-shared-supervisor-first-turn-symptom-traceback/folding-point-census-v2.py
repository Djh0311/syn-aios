#!/usr/bin/env python3
"""折叠点普查 v2 —— 修正 v1 的截断错误。

v1 用 awk 在文件里第一个 #[cfg(test)] 处整体截断，但那个属性多数是**单项属性**
（作用于紧跟的一个 item），不是模块边界。结果 v1 把大量生产代码当成测试丢掉了
（commands.rs 丢 85%、supervisor_orchestrator.rs 丢 69%）。

v2 只剔除真正的测试模块：`#[cfg(test)]` 紧跟 `mod xxx {` 时，按花括号配对剔到模块结束。
单项 #[cfg(test)] 只剔除紧跟的那一个 item（按缩进/花括号配对近似），其余一律计入生产。
"""
import re, sys, json, pathlib

PATTERNS = {
    "map_err(|_|": r"map_err\(\|_\|",
    ".ok()": r"\.ok\(\)",
    "unwrap_or": r"unwrap_or",
    "let _ =": r"let _ =",
    "Err(_)": r"Err\(_\)",
    ".is_err()": r"\.is_err\(\)",
    "unwrap()": r"\.unwrap\(\)",
}

def production_lines(path):
    lines = pathlib.Path(path).read_text(encoding="utf8").splitlines()
    skip = [False] * len(lines)
    i = 0
    while i < len(lines):
        if lines[i].strip() == "#[cfg(test)]":
            # 找到属性作用的那个 item 的起始行
            j = i + 1
            while j < len(lines) and lines[j].strip() == "":
                j += 1
            if j >= len(lines):
                break
            # 从 item 起始行按花括号配对找结束
            depth = 0
            started = False
            k = j
            while k < len(lines):
                depth += lines[k].count("{") - lines[k].count("}")
                if "{" in lines[k]:
                    started = True
                if started and depth <= 0:
                    break
                if not started and lines[k].rstrip().endswith(";"):
                    break
                k += 1
            for m in range(i, min(k + 1, len(lines))):
                skip[m] = True
            i = k + 1
            continue
        i += 1
    return [line for index, line in enumerate(lines) if not skip[index]], len(lines), sum(skip)

report = {"method": "v2：只剔真正的测试项/测试模块，单项 #[cfg(test)] 按花括号配对只剔一个 item", "files": []}
for path in sys.argv[1:]:
    body, total, skipped = production_lines(path)
    text = "\n".join(body)
    counts = {name: len(re.findall(pattern, text)) for name, pattern in PATTERNS.items()}
    report["files"].append({
        "file": path, "totalLines": total, "skippedAsTest": skipped,
        "productionLines": total - skipped, "counts": counts,
        "foldingTotal": sum(counts[k] for k in ("map_err(|_|", ".ok()", "unwrap_or", "let _ =", "Err(_)", ".is_err()")),
    })
print(json.dumps(report, ensure_ascii=False, indent=2))
