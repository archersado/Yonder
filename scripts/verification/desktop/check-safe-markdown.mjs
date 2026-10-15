import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

class Node {
  constructor(tagName = '#text', value = '') { this.tagName = tagName.toUpperCase(); this.value = value; this.children = []; this.attributes = {}; this.className = ''; }
  appendChild(child) { this.children.push(child); return child; }
  replaceChildren(...children) { this.children = children; }
  setAttribute(name, value) { this.attributes[name] = String(value); }
  set textContent(value) { this.value = String(value); this.children = []; }
  get textContent() { return this.tagName === '#TEXT' ? this.value : this.value + this.children.map(child => child.textContent).join(''); }
}

const document = { createElement: tag => new Node(tag), createTextNode: value => new Node('#text', value) };
const window = {};
vm.runInNewContext(readFileSync(new URL('../../../apps/desktop/ui/safe-markdown.js', import.meta.url), 'utf8'), { window, document, URL });

const tags = node => [node.tagName, ...node.children.flatMap(tags)];
const render = markdown => { const container = new Node('div'); window.YondaMarkdown.render(container, markdown); return container; };

const formatted = render('# 标题\n\n- **粗体**\n- *斜体*\n\n> 引用\n\n```js\nalert("text only")\n```\n\n`code` ~~删除~~ [官网](https://example.com)');
const formattedTags = tags(formatted);
for (const tag of ['H2', 'UL', 'LI', 'STRONG', 'EM', 'BLOCKQUOTE', 'PRE', 'CODE', 'DEL']) assert.ok(formattedTags.includes(tag), `缺少 ${tag}`);
assert.ok(formatted.textContent.includes('官网（https://example.com）'));

const hostile = render('<script>danger()</script> ![追踪](https://example.com/pixel) [危险](javascript:danger())');
const hostileTags = tags(hostile);
assert.ok(!hostileTags.includes('SCRIPT') && !hostileTags.includes('IMG') && !hostileTags.includes('A'));
assert.ok(hostile.textContent.includes('<script>danger()</script>') && hostile.textContent.includes('图片：追踪'));
assert.ok(hostile.textContent.includes('[危险](javascript:danger())'));

const streaming = new Node('div');
window.YondaMarkdown.render(streaming, '**尚未');
assert.ok(streaming.textContent.includes('**尚未') && !tags(streaming).includes('STRONG'));
window.YondaMarkdown.render(streaming, '**已经补齐**');
assert.ok(tags(streaming).includes('STRONG') && streaming.textContent.includes('已经补齐'));

const looseList = render('- 第一项\n\n- 第二项\n\n- 第三项');
assert.equal(tags(looseList).filter(tag => tag === 'UL').length, 1, '松散列表不应拆成多个列表');
assert.equal(tags(looseList).filter(tag => tag === 'LI').length, 3, '松散列表应保留所有项目');

const source = readFileSync(new URL('../../../apps/desktop/ui/safe-markdown.js', import.meta.url), 'utf8');
assert.ok(!source.includes('innerHTML'));
console.log(JSON.stringify({ markdown: true, loose_list: true, streaming: true, hostile_content_inert: true, inner_html: false }));
