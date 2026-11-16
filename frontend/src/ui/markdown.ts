/** Minimal Markdown → HTML for the in-app docs (headings, tables, code, lists, bold). */
export function renderMarkdown(src: string): string {
  const lines = src.trim().split(/\r?\n/);
  const out: string[] = [];
  let i = 0;
  let inCode = false;
  let codeBuf: string[] = [];

  const flushCode = () => {
    if (!inCode) return;
    out.push(`<pre><code>${escapeHtml(codeBuf.join("\n"))}</code></pre>`);
    codeBuf = [];
    inCode = false;
  };

  while (i < lines.length) {
    const line = lines[i];

    if (line.trim().startsWith("```")) {
      if (inCode) flushCode();
      else {
        inCode = true;
        codeBuf = [];
      }
      i++;
      continue;
    }
    if (inCode) {
      codeBuf.push(line);
      i++;
      continue;
    }

    if (line.startsWith("|") && i + 1 < lines.length && /^\|?\s*-+/.test(lines[i + 1])) {
      const rows: string[] = [];
      while (i < lines.length && lines[i].startsWith("|")) {
        if (!/^\|?\s*-+/.test(lines[i])) rows.push(lines[i]);
        i++;
      }
      out.push(renderTable(rows));
      continue;
    }

    if (line.startsWith("# ")) {
      out.push(`<h1>${inline(line.slice(2))}</h1>`);
    } else if (line.startsWith("## ")) {
      out.push(`<h2>${inline(line.slice(3))}</h2>`);
    } else if (line.startsWith("### ")) {
      out.push(`<h3>${inline(line.slice(4))}</h3>`);
    } else if (line.startsWith("- ")) {
      const items: string[] = [];
      while (i < lines.length && lines[i].startsWith("- ")) {
        items.push(`<li>${inline(lines[i].slice(2))}</li>`);
        i++;
      }
      out.push(`<ul>${items.join("")}</ul>`);
      continue;
    } else if (/^\d+\.\s/.test(line)) {
      const items: string[] = [];
      while (i < lines.length && /^\d+\.\s/.test(lines[i])) {
        items.push(`<li>${inline(lines[i].replace(/^\d+\.\s/, ""))}</li>`);
        i++;
      }
      out.push(`<ol>${items.join("")}</ol>`);
      continue;
    } else if (line.trim() === "") {
      // skip
    } else {
      out.push(`<p>${inline(line)}</p>`);
    }
    i++;
  }
  flushCode();
  return out.join("\n");
}

function renderTable(rows: string[]): string {
  const parse = (r: string) =>
    r
      .split("|")
      .slice(1, -1)
      .map((c) => c.trim());
  if (!rows.length) return "";
  const head = parse(rows[0]);
  const body = rows.slice(1).map(parse);
  return `<table><thead><tr>${head.map((h) => `<th>${inline(h)}</th>`).join("")}</tr></thead><tbody>${body
    .map((r) => `<tr>${r.map((c) => `<td>${inline(c)}</td>`).join("")}</tr>`)
    .join("")}</tbody></table>`;
}

function inline(text: string): string {
  let s = escapeHtml(text);
  s = s.replace(/`([^`]+)`/g, "<code>$1</code>");
  s = s.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
  s = s.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');
  return s;
}

function escapeHtml(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}
