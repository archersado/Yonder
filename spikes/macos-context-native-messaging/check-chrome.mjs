import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";

const [profileDirectory, extensionDirectory] = process.argv.slice(2);
const chrome = "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome";
const extensionUrl = "chrome-extension://mofdddjaniddgalgegfdjegiegpneokc/test.html";
const child = spawn(chrome, [
  "--headless=new",
  "--disable-gpu",
  "--no-first-run",
  "--remote-debugging-port=0",
  `--user-data-dir=${profileDirectory}`,
  `--disable-extensions-except=${extensionDirectory}`,
  `--load-extension=${extensionDirectory}`,
  "about:blank",
], { stdio: "ignore" });

async function devtoolsPort() {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    try {
      const [port] = (await readFile(`${profileDirectory}/DevToolsActivePort`, "utf8")).trim().split("\n");
      return Number(port);
    } catch {
      await delay(100);
    }
  }
  throw new Error("devtools_timeout");
}

async function cdp(webSocketUrl) {
  const socket = new WebSocket(webSocketUrl);
  await new Promise((resolve, reject) => {
    socket.addEventListener("open", resolve, { once: true });
    socket.addEventListener("error", reject, { once: true });
  });
  let nextId = 0;
  const pending = new Map();
  socket.addEventListener("message", (event) => {
    const message = JSON.parse(event.data);
    const waiter = pending.get(message.id);
    if (!waiter) return;
    pending.delete(message.id);
    if (message.error) waiter.reject(new Error(message.error.message));
    else waiter.resolve(message.result);
  });
  return {
    call(method, params = {}) {
      nextId += 1;
      const id = nextId;
      socket.send(JSON.stringify({ id, method, params }));
      return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
    },
    close() { socket.close(); },
  };
}

async function stopChrome() {
  if (child.exitCode !== null) return;
  child.kill("SIGTERM");
  await Promise.race([
    new Promise((resolve) => child.once("exit", resolve)),
    delay(2000).then(() => child.kill("SIGKILL")),
  ]);
}

try {
  const port = await devtoolsPort();
  const created = await fetch(`http://127.0.0.1:${port}/json/new?${encodeURIComponent(extensionUrl)}`, { method: "PUT" });
  if (!created.ok) throw new Error("target_create_failed");
  const target = await created.json();
  const session = await cdp(target.webSocketDebuggerUrl);
  await session.call("Runtime.enable");
  let value;
  for (let attempt = 0; attempt < 50; attempt += 1) {
    const evaluation = await session.call("Runtime.evaluate", {
      expression: 'document.documentElement.dataset.done === "true" ? document.querySelector("#result").textContent : null',
      returnByValue: true,
    });
    value = evaluation.result.value;
    if (value) break;
    await delay(100);
  }
  const diagnostic = value ? undefined : await session.call("Runtime.evaluate", {
    expression: 'JSON.stringify({href:location.href,ready:document.readyState,result:document.querySelector("#result")?.textContent||null,runtime_id:globalThis.chrome?.runtime?.id||null})',
    returnByValue: true,
  });
  session.close();
  const result = value ? JSON.parse(value) : { status: "result_timeout", accepted: 0, incognito_rejected: false };
  const report = {
    passed: result.status === "passed" && result.accepted === 3 && result.incognito_rejected === true,
    status: result.status,
    accepted: result.accepted || 0,
    incognito_rejected: result.incognito_rejected === true,
    browser: "Google Chrome",
    host_manifest_scope: "current_user",
    ...(diagnostic ? { diagnostic: JSON.parse(diagnostic.result.value) } : {}),
  };
  process.stdout.write(`${JSON.stringify(report)}\n`);
  if (!report.passed) process.exitCode = 1;
} catch (error) {
  process.stdout.write(`${JSON.stringify({ passed: false, status: error.message })}\n`);
  process.exitCode = 1;
} finally {
  await stopChrome();
}
