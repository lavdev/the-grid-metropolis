const CODE_TOKEN_PREFIX = '\u0000CODE_';
const HIGHLIGHT_TOKEN_PREFIX = '\u0000HL_';

function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function safeHref(value: string): string | null {
  const href = value.trim();
  if (/^(https?:\/\/|\/|#|C:[\\/])/i.test(href)) return href;
  return null;
}

function renderInline(source: string): string {
  const codeTokens: string[] = [];
  let html = escapeHtml(source);

  html = html.replace(/`([^`\n]+)`/g, (_match, code: string) => {
    const token = `${CODE_TOKEN_PREFIX}${codeTokens.length}\u0000`;
    codeTokens.push(`<code>${code}</code>`);
    return token;
  });

  html = html.replace(/!?\[([^\]]+)\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g, (match, label: string, rawHref: string) => {
    if (match.startsWith('!')) return label;
    const href = safeHref(rawHref);
    return href
      ? `<a href="${escapeHtml(href)}" target="_blank" rel="noreferrer">${renderInline(label)}</a>`
      : label;
  });

  html = html
    .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
    .replace(/__(.+?)__/g, '<strong>$1</strong>')
    .replace(/~~(.+?)~~/g, '<del>$1</del>')
    .replace(/(^|[^\w])\*([^*\n]+)\*(?!\w)/g, '$1<em>$2</em>')
    .replace(/(^|[^\w])_([^_\n]+)_(?!\w)/g, '$1<em>$2</em>');

  return html.replace(/\u0000CODE_(\d+)\u0000/g, (_match, index: string) => codeTokens[Number(index)] || '');
}

function languageFamily(language: string): string {
  const normalized = language.toLowerCase();
  if (['ts', 'tsx', 'typescript', 'js', 'jsx', 'javascript'].includes(normalized)) return 'typescript';
  if (['rs', 'rust'].includes(normalized)) return 'rust';
  if (['py', 'python'].includes(normalized)) return 'python';
  if (['json', 'jsonc'].includes(normalized)) return 'json';
  if (['sql'].includes(normalized)) return 'sql';
  if (['sh', 'bash', 'shell', 'zsh'].includes(normalized)) return 'bash';
  if (['html', 'xml', 'svg'].includes(normalized)) return 'html';
  if (['css', 'scss', 'less'].includes(normalized)) return 'css';
  if (['md', 'markdown'].includes(normalized)) return 'markdown';
  return 'generic';
}

export function highlightCode(source: string, language: string): string {
  const family = languageFamily(language);
  const tokens: string[] = [];
  let html = escapeHtml(source);

  const protect = (pattern: RegExp, className: string) => {
    html = html.replace(pattern, (value) => {
      const token = `${HIGHLIGHT_TOKEN_PREFIX}${tokens.length}\u0000`;
      tokens.push(`<span class="code-token-${className}">${value}</span>`);
      return token;
    });
  };

  // Strings are protected first so comment markers inside a string are not misclassified.
  protect(/&quot;(?:\\.|[^&quot;\\])*&quot;|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`/g, 'string');

  if (family === 'sql') protect(/--[^\n]*/g, 'comment');
  else if (family === 'python' || family === 'bash' || family === 'markdown') protect(/(^|\n)\s*#[^\n]*/g, 'comment');
  else protect(/\/\/[^\n]*|\/\*[\s\S]*?\*\//g, 'comment');

  const keywords: Record<string, string[]> = {
    rust: ['as', 'async', 'await', 'const', 'crate', 'else', 'enum', 'fn', 'for', 'if', 'impl', 'in', 'let', 'loop', 'match', 'mod', 'move', 'mut', 'pub', 'ref', 'return', 'self', 'Self', 'struct', 'trait', 'type', 'use', 'where', 'while'],
    typescript: ['as', 'async', 'await', 'break', 'case', 'catch', 'class', 'const', 'continue', 'else', 'export', 'extends', 'finally', 'for', 'from', 'function', 'if', 'implements', 'import', 'interface', 'let', 'new', 'of', 'return', 'static', 'switch', 'throw', 'try', 'type', 'var', 'while'],
    python: ['as', 'async', 'await', 'class', 'def', 'elif', 'else', 'for', 'from', 'if', 'import', 'in', 'is', 'lambda', 'match', 'None', 'pass', 'raise', 'return', 'try', 'while', 'with', 'yield'],
    json: ['true', 'false', 'null'],
    sql: ['SELECT', 'FROM', 'WHERE', 'INSERT', 'INTO', 'VALUES', 'UPDATE', 'SET', 'DELETE', 'CREATE', 'TABLE', 'ALTER', 'DROP', 'JOIN', 'LEFT', 'RIGHT', 'INNER', 'ON', 'AS', 'AND', 'OR', 'NOT', 'NULL', 'ORDER', 'BY', 'GROUP', 'LIMIT'],
    bash: ['case', 'do', 'done', 'elif', 'else', 'esac', 'fi', 'for', 'function', 'if', 'in', 'then', 'until', 'while'],
    css: ['@media', '@supports', 'important'],
    html: [],
    markdown: [],
    generic: ['class', 'const', 'else', 'export', 'false', 'for', 'function', 'if', 'import', 'let', 'new', 'null', 'return', 'true', 'type', 'var', 'while'],
  };

  const keywordList = keywords[family] || keywords.generic;
  if (keywordList.length > 0) {
    const escapedKeywords = keywordList.map((keyword) => keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')).join('|');
    const flags = family === 'sql' ? 'g' : 'g';
    protect(new RegExp(`\\b(?:${escapedKeywords})\\b`, flags), 'keyword');
  }

  protect(/\b(?:0x[\da-f]+|\d+(?:\.\d+)?)\b/gi, 'number');

  if (family === 'html') {
    protect(/&lt;\/?[\w:-]+(?:\s+[^&]*?)?\/??&gt;/g, 'tag');
  }

  return html.replace(/\u0000HL_(\d+)\u0000/g, (_match, index: string) => tokens[Number(index)] || '');
}

function isTableSeparator(line: string): boolean {
  const cells = line.trim().replace(/^\||\|$/g, '').split('|');
  return cells.length > 0 && cells.every((cell) => /^\s*:?-{3,}:?\s*$/.test(cell));
}

function tableCells(line: string): string[] {
  return line.trim().replace(/^\||\|$/g, '').split('|').map((cell) => cell.trim());
}

/**
 * Small, dependency-free Markdown renderer for terminal messages.
 * All text is escaped before formatting so {@html} never executes user content.
 */
export function renderMarkdown(source: string): string {
  if (!source) return '';

  const lines = source.replace(/\r\n?/g, '\n').split('\n');
  const output: string[] = [];
  let paragraph: string[] = [];

  const flushParagraph = () => {
    if (paragraph.length === 0) return;
    output.push(`<p>${paragraph.map(renderInline).join('<br />')}</p>`);
    paragraph = [];
  };

  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    const trimmed = line.trim();

    const fence = trimmed.match(/^```\s*([\w+-]*)\s*$/);
    if (fence) {
      flushParagraph();
      const language = fence[1] ? ` class="language-${escapeHtml(fence[1])}"` : '';
      const codeLines: string[] = [];
      index += 1;
      while (index < lines.length && !/^```\s*$/.test(lines[index].trim())) {
        codeLines.push(lines[index]);
        index += 1;
      }
      if (index < lines.length) index += 1;
      output.push(`<pre><code${language}>${highlightCode(codeLines.join('\n'), fence[1])}</code></pre>`);
      continue;
    }

    const heading = trimmed.match(/^(#{1,6})\s+(.+?)\s*#*$/);
    if (heading) {
      flushParagraph();
      const level = heading[1].length;
      output.push(`<h${level}>${renderInline(heading[2])}</h${level}>`);
      index += 1;
      continue;
    }

    if (trimmed.includes('|') && index + 1 < lines.length && isTableSeparator(lines[index + 1])) {
      flushParagraph();
      const header = tableCells(line);
      index += 2;
      const rows: string[][] = [];
      while (index < lines.length && lines[index].trim().includes('|') && lines[index].trim() !== '') {
        rows.push(tableCells(lines[index]));
        index += 1;
      }
      output.push(`<div class="md-table-wrap"><table><thead><tr>${header.map((cell) => `<th>${renderInline(cell)}</th>`).join('')}</tr></thead><tbody>${rows.map((row) => `<tr>${header.map((_cell, cellIndex) => `<td>${renderInline(row[cellIndex] || '')}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`);
      continue;
    }

    if (/^\s*[-*_](?:\s*[-*_]){2,}\s*$/.test(line)) {
      flushParagraph();
      output.push('<hr />');
      index += 1;
      continue;
    }

    if (/^\s*[-*+]\s+/.test(line)) {
      flushParagraph();
      const items: string[] = [];
      while (index < lines.length) {
        const item = lines[index].match(/^\s*[-*+]\s+(.+)$/);
        if (!item) break;
        items.push(`<li>${renderInline(item[1])}</li>`);
        index += 1;
      }
      output.push(`<ul>${items.join('')}</ul>`);
      continue;
    }

    if (/^\s*\d+[.)]\s+/.test(line)) {
      flushParagraph();
      const items: string[] = [];
      while (index < lines.length) {
        const item = lines[index].match(/^\s*\d+[.)]\s+(.+)$/);
        if (!item) break;
        items.push(`<li>${renderInline(item[1])}</li>`);
        index += 1;
      }
      output.push(`<ol>${items.join('')}</ol>`);
      continue;
    }

    if (/^\s*>\s?/.test(line)) {
      flushParagraph();
      const quote: string[] = [];
      while (index < lines.length) {
        const item = lines[index].match(/^\s*>\s?(.*)$/);
        if (!item) break;
        quote.push(renderInline(item[1]));
        index += 1;
      }
      output.push(`<blockquote>${quote.join('<br />')}</blockquote>`);
      continue;
    }

    if (trimmed === '') {
      flushParagraph();
    } else {
      paragraph.push(line);
    }
    index += 1;
  }

  flushParagraph();
  return output.join('');
}
