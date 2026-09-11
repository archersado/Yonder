import { createHash } from "node:crypto";
import { mkdir, readFile, rename, writeFile } from "node:fs/promises";
import { basename, join } from "node:path";
import { DOMParser, XMLSerializer } from "@xmldom/xmldom";
import { unzipSync, zipSync } from "fflate";

const cases = [
  ["sample.docx", "word/document.xml", "YONDER_DOCX_BEFORE", "YONDER_DOCX_AFTER"],
  ["sample.xlsx", "xl/worksheets/sheet1.xml", "YONDER_XLSX_BEFORE", "YONDER_XLSX_AFTER"],
  ["sample.pptx", "ppt/slides/slide1.xml", "YONDER_PPTX_BEFORE", "YONDER_PPTX_AFTER"],
];
const realCases = [
  ["sample.docx", "word/document.xml", "Basic Integration Printer Test", "Yonder Integration Test"],
  ["sample.xlsx", "xl/sharedStrings.xml", "file name", "Yonder file name"],
  ["sample.pptx", "ppt/slides/slide3.xml", "Graphics Usage", "Yonder Graphics Usage"],
];
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");

async function replaceText(input, output, expectedHash, partName, before, after) {
  const source = await readFile(input);
  if (digest(source) !== expectedHash) throw new Error("document_conflict");
  const parts = unzipSync(source);
  const originalParts = Object.fromEntries(Object.entries(parts).map(([name, bytes]) => [name, digest(bytes)]));
  const document = new DOMParser({ onError: () => {} }).parseFromString(new TextDecoder().decode(parts[partName]), "application/xml");
  let replacements = 0;
  const visit = (node) => {
    if (node.nodeType === 3 && node.data.includes(before)) {
      node.data = node.data.replaceAll(before, after);
      replacements += 1;
    }
    for (let child = node.firstChild; child; child = child.nextSibling) visit(child);
  };
  visit(document);
  if (replacements !== 1) throw new Error(`replacement_count:${replacements}`);
  parts[partName] = new TextEncoder().encode(new XMLSerializer().serializeToString(document));
  const archive = zipSync(parts);
  unzipSync(archive);
  for (const [name, hash] of Object.entries(originalParts)) {
    if (name !== partName && digest(unzipSync(archive)[name]) !== hash) throw new Error(`fidelity:${name}`);
  }
  await mkdir(join(output, ".."), { recursive: true });
  const temporary = `${output}.${process.pid}.tmp`;
  await writeFile(temporary, archive);
  await rename(temporary, output);
  return { replacements, bytes: archive.length };
}

await mkdir("output/node", { recursive: true });
const started = performance.now();
for (const [name, part, before, after] of cases) {
  const input = join("fixtures/synthetic", name);
  const source = await readFile(input);
  const result = await replaceText(input, join("output/node", basename(name)), digest(source), part, before, after);
  console.log(JSON.stringify({ name, ...result }));
}
for (const [name, part, before, after] of realCases) {
  const input = join("fixtures/real", name);
  const source = await readFile(input);
  const result = await replaceText(input, join("output/node", `real-${name}`), digest(source), part, before, after);
  console.log(JSON.stringify({ name: `real-${name}`, ...result }));
}
let conflict = false;
try {
  await replaceText("fixtures/synthetic/sample.docx", "output/node/conflict.docx", "bad-hash", cases[0][1], cases[0][2], cases[0][3]);
} catch (error) {
  conflict = error.message === "document_conflict";
}
if (!conflict) throw new Error("expected_hash 门禁失败");
console.log(JSON.stringify({ conflict, elapsed_ms: Math.round(performance.now() - started) }));
