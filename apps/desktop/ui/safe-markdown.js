(() => {
  const safeUrl = value => {
    try {
      const url = new URL(value, 'https://yonder.invalid');
      return ['http:', 'https:', 'mailto:'].includes(url.protocol) ? value : null;
    } catch { return null; }
  };

  const text = value => document.createTextNode(value);
  const element = (tag, className) => {
    const node = document.createElement(tag);
    if (className) node.className = className;
    return node;
  };

  const appendInline = (parent, source) => {
    let index = 0;
    const plain = value => parent.appendChild(text(value));
    while (index < source.length) {
      if (source[index] === '\n') {
        parent.appendChild(element('br')); index += 1; continue;
      }
      if (source.startsWith('![', index)) {
        const labelEnd = source.indexOf('](', index + 2), close = labelEnd < 0 ? -1 : source.indexOf(')', labelEnd + 2);
        if (labelEnd >= 0 && close >= 0) {
          const placeholder = element('span', 'md-image-placeholder');
          placeholder.textContent = `图片：${source.slice(index + 2, labelEnd) || '未命名'}`;
          parent.appendChild(placeholder); index = close + 1; continue;
        }
      }
      if (source[index] === '[') {
        const labelEnd = source.indexOf('](', index + 1), close = labelEnd < 0 ? -1 : source.indexOf(')', labelEnd + 2);
        if (labelEnd >= 0 && close >= 0) {
          const label = source.slice(index + 1, labelEnd), target = source.slice(labelEnd + 2, close), accepted = safeUrl(target);
          if (accepted) {
            const link = element('span', 'md-link');
            link.setAttribute('role', 'link'); link.setAttribute('aria-label', `${label}，链接 ${accepted}`);
            link.textContent = `${label}（${accepted}）`;
            parent.appendChild(link); index = close + 1; continue;
          }
        }
      }
      const tokens = [
        ['`', '`', 'code'], ['**', '**', 'strong'], ['~~', '~~', 'del'], ['*', '*', 'em'], ['_', '_', 'em'],
      ];
      let matched = false;
      for (const [open, close, tag] of tokens) {
        if (!source.startsWith(open, index)) continue;
        const end = source.indexOf(close, index + open.length);
        if (end <= index + open.length) continue;
        const node = element(tag);
        const content = source.slice(index + open.length, end);
        if (tag === 'code') node.textContent = content; else appendInline(node, content);
        parent.appendChild(node); index = end + close.length; matched = true; break;
      }
      if (matched) continue;
      let next = index + 1;
      while (next < source.length && source[next] !== '\n' && !['`', '*', '_', '~', '[', '!'].includes(source[next])) next += 1;
      plain(source.slice(index, next)); index = next;
    }
  };

  const isBlockStart = line => /^(?:```|#{1,3}\s|>\s?|[-+*]\s+|\d+[.)]\s+)/.test(line);
  const render = (container, markdown) => {
    const root = element('div', 'markdown-body'), lines = String(markdown ?? '').replace(/\r\n?/g, '\n').split('\n');
    let index = 0;
    while (index < lines.length) {
      const line = lines[index];
      if (!line.trim()) { index += 1; continue; }
      if (line.startsWith('```')) {
        const code = [], language = line.slice(3).trim(); index += 1;
        while (index < lines.length && !lines[index].startsWith('```')) { code.push(lines[index]); index += 1; }
        if (index < lines.length) index += 1;
        const pre = element('pre'), body = element('code');
        if (language && /^[a-z0-9_+#.-]{1,32}$/i.test(language)) body.setAttribute('data-language', language);
        body.textContent = code.join('\n'); pre.appendChild(body); root.appendChild(pre); continue;
      }
      const heading = /^(#{1,3})\s+(.+)$/.exec(line);
      if (heading) {
        const node = element(`h${heading[1].length + 1}`); appendInline(node, heading[2]); root.appendChild(node); index += 1; continue;
      }
      if (/^>\s?/.test(line)) {
        const parts = [];
        while (index < lines.length && /^>\s?/.test(lines[index])) { parts.push(lines[index].replace(/^>\s?/, '')); index += 1; }
        const node = element('blockquote'); appendInline(node, parts.join('\n')); root.appendChild(node); continue;
      }
      const list = /^([-+*])\s+(.+)$/.exec(line) || /^(\d+)[.)]\s+(.+)$/.exec(line);
      if (list) {
        const ordered = /^\d/.test(line), node = element(ordered ? 'ol' : 'ul');
        const itemPattern = ordered ? /^\d+[.)]\s+(.+)$/ : /^[-+*]\s+(.+)$/;
        while (index < lines.length) {
          const item = itemPattern.exec(lines[index]);
          if (!item) {
            if (!lines[index].trim()) {
              let next = index;
              while (next < lines.length && !lines[next].trim()) next += 1;
              if (next < lines.length && itemPattern.test(lines[next])) { index = next; continue; }
            }
            break;
          }
          const child = element('li'); appendInline(child, item[1]); node.appendChild(child); index += 1;
        }
        root.appendChild(node); continue;
      }
      const paragraph = [line]; index += 1;
      while (index < lines.length && lines[index].trim() && !isBlockStart(lines[index])) { paragraph.push(lines[index]); index += 1; }
      const node = element('p'); appendInline(node, paragraph.join('\n')); root.appendChild(node);
    }
    container.replaceChildren(root);
  };

  window.YondaMarkdown = Object.freeze({ render });
})();
