import process from "node:process";

const MAX_MESSAGE_BYTES = 1024 * 1024;

export function decodeFrames(buffer) {
  const messages = [];
  while (buffer.length >= 4) {
    const length = buffer.readUInt32LE();
    if (length > MAX_MESSAGE_BYTES) throw new Error("入站消息超过 1 MiB");
    if (buffer.length < length + 4) break;
    messages.push(JSON.parse(buffer.subarray(4, length + 4).toString("utf8")));
    buffer = buffer.subarray(length + 4);
  }
  return { messages, pending: buffer };
}

export function encodeFrame(message) {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  if (body.length > MAX_MESSAGE_BYTES) throw new Error("出站消息超过 1 MiB");
  const header = Buffer.allocUnsafe(4);
  header.writeUInt32LE(body.length);
  return Buffer.concat([header, body]);
}

if (process.argv[1]?.endsWith("native-host.mjs")) {
  let pending = Buffer.alloc(0);
  process.stdin.on("data", (chunk) => {
    const decoded = decodeFrames(Buffer.concat([pending, chunk]));
    pending = decoded.pending;
    for (const message of decoded.messages) {
      process.stdout.write(encodeFrame({ ok: true, request_id: message.request_id ?? null }));
    }
  });
  process.stdin.on("end", () => {
    if (pending.length) throw new Error("Native Messaging 帧不完整");
  });
}
