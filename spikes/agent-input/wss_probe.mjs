#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { createServer } from 'node:https';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const temporary = mkdtempSync(join(tmpdir(), 'yonder-agent-wss-'));
const key = join(temporary, 'key.pem');
const cert = join(temporary, 'cert.pem');
execFileSync('/usr/bin/openssl', ['req', '-x509', '-newkey', 'rsa:2048', '-nodes',
  '-keyout', key, '-out', cert, '-days', '1', '-subj', '/CN=localhost'], { stdio: 'ignore' });

function frame(value) {
  const payload = Buffer.from(JSON.stringify(value));
  if (payload.length < 126) return Buffer.concat([Buffer.from([0x81, payload.length]), payload]);
  const header = Buffer.alloc(4);
  header[0] = 0x81;
  header[1] = 126;
  header.writeUInt16BE(payload.length, 2);
  return Buffer.concat([header, payload]);
}

function readClientFrame(buffer) {
  if (buffer.length < 6) return;
  const opcode = buffer[0] & 0x0f;
  let length = buffer[1] & 0x7f;
  let offset = 2;
  if (length === 126) {
    if (buffer.length < 8) return;
    length = buffer.readUInt16BE(2);
    offset = 4;
  }
  if (buffer.length < offset + 4 + length) return;
  const mask = buffer.subarray(offset, offset + 4);
  const payload = buffer.subarray(offset + 4, offset + 4 + length);
  for (let index = 0; index < payload.length; index++) payload[index] ^= mask[index % 4];
  return [opcode === 1 ? JSON.parse(payload.toString()) : null,
    buffer.subarray(offset + 4 + length), opcode];
}

const server = createServer({ key: await import('node:fs').then(({ readFileSync }) => readFileSync(key)),
  cert: await import('node:fs').then(({ readFileSync }) => readFileSync(cert)) });
let resolveDone;
const done = new Promise(resolve => { resolveDone = resolve; });

server.on('upgrade', (request, socket) => {
  const accept = createHash('sha1').update(request.headers['sec-websocket-key'] +
    '258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest('base64');
  socket.write('HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n' +
    `Sec-WebSocket-Accept: ${accept}\r\n\r\n`);
  let pending = Buffer.alloc(0);
  socket.on('data', chunk => {
    pending = Buffer.concat([pending, chunk]);
    for (let decoded; (decoded = readClientFrame(pending));) {
      const [message, rest, opcode] = decoded;
      pending = rest;
      if (opcode === 8) {
        socket.end(Buffer.from([0x88, 0x00]));
        continue;
      }
      if (!message) continue;
      if (message.method === 'gateway.hello') {
        socket.write(frame({ id: message.id, result: { accepted: true } }));
      } else if (message.method === 'agent.input') {
        if (message.params.session_id !== 'cloud-session-1') throw new Error('会话绑定错误');
        socket.write(frame({ id: message.id, result: { accepted: true } }));
        resolveDone();
      }
    }
  });
});

await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
process.env.NODE_TLS_REJECT_UNAUTHORIZED = '0';
const address = server.address();
const client = new WebSocket(`wss://localhost:${address.port}`);
let phase = 'hello';
client.addEventListener('open', () => client.send(JSON.stringify({ method: 'gateway.hello', id: 'hello-1',
  params: { agent_id: 'cloud-fixture', session_id: 'cloud-session-1', offered_capabilities: ['user_input'] } })));
client.addEventListener('message', event => {
  const message = JSON.parse(event.data);
  if (!message.result?.accepted) throw new Error('Runtime未确认');
  if (phase === 'hello') {
    phase = 'input';
    const now = Date.now();
    client.send(JSON.stringify({ method: 'agent.input', id: 'input-1', params: { input_id: 'input-1',
      session_id: 'cloud-session-1', source: 'voice', content: '测试云端输入', created_at: now,
      deadline: now + 10_000 } }));
  } else {
    client.close();
  }
});

await Promise.race([done, new Promise((_, reject) => setTimeout(() => reject(new Error('WSS确认超时')), 2000))]);
await new Promise(resolve => server.close(resolve));
rmSync(temporary, { recursive: true, force: true });
console.log(JSON.stringify({ transport: 'wss', direction: 'yonder-to-cloud-runtime', result: 'accepted' }));
