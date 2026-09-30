import { CuaDriver } from '@trycua/cua-driver';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname } from 'node:path';

const driver = CuaDriver.create(undefined);
const result = { version: '0.30.4', metadata_ready: false, call_tool: false, click: {}, hotkey: {}, type_text: {} };
try {
  await driver.metadata();
  result.metadata_ready = true;
  result.call_tool = typeof driver.callTool === 'function';
  const inventory = JSON.parse(await driver.listToolsJson());
  const tools = Array.isArray(inventory) ? inventory : inventory.tools ?? [];
  for (const [name, target] of [['click', result.click], ['hotkey', result.hotkey], ['type_text', result.type_text]]) {
    const descriptor = tools.find(tool => tool.name === name);
    const properties = descriptor?.inputSchema?.properties ?? descriptor?.input_schema?.properties ?? {};
    target.available = descriptor !== undefined;
    target.exact_target = Object.hasOwn(properties, 'target');
    target.delivery_mode = Object.hasOwn(properties, 'delivery_mode') || Object.hasOwn(properties, 'deliveryMode');
    target.coordinates = (Object.hasOwn(properties, 'x') && Object.hasOwn(properties, 'y')) || Object.hasOwn(properties, 'position');
  }
  result.passed = result.metadata_ready && result.call_tool && result.click.available && result.click.exact_target && result.click.delivery_mode && result.click.coordinates && result.hotkey.available && result.type_text.available;
} catch {
  result.passed = false;
  result.failure = 'schema-probe-failed';
} finally {
  try { await driver.shutdown(); } catch { result.cleanup_error = true; result.passed = false; }
  driver.uniffiDestroy();
}
console.log(JSON.stringify(result));
if (process.argv[2]) {
  mkdirSync(dirname(process.argv[2]), { recursive:true });
  writeFileSync(process.argv[2], JSON.stringify(result,null,2)+'\n');
}
if (!result.passed) process.exitCode = 1;
