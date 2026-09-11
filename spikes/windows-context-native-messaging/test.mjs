import assert from "node:assert/strict";
import { decodeFrames, encodeFrame } from "./native-host.mjs";

const first = encodeFrame({ request_id: "中文-1" });
const partial = decodeFrames(first.subarray(0, 3));
assert.deepEqual(partial.messages, []);

const input = Buffer.concat([partial.pending, first.subarray(3), encodeFrame({ request_id: "2" })]);
assert.deepEqual(decodeFrames(input).messages, [{ request_id: "中文-1" }, { request_id: "2" }]);

assert.throws(() => decodeFrames(Buffer.from([1, 0, 16, 0])), /超过 1 MiB/);
console.log("PASS: UTF-8、分片、连续帧与超限拒绝");
