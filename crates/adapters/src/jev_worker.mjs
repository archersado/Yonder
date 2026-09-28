import { createInterface } from 'node:readline';
import { pathToFileURL } from 'node:url';

const sdk = await import(pathToFileURL(process.argv[2]).href);
const { choice, TypeSafeClient } = sdk;

for await (const line of createInterface({ input: process.stdin, crlfDelay: Infinity })) {
  let request;
  try {
    request = JSON.parse(line);
  } catch {
    process.exitCode = 2;
    break;
  }

  try {
    const client = new TypeSafeClient({
      apiKey: request.api_key,
      baseURL: request.endpoint,
      logLevel: 'off',
      timeout: request.timeout_ms,
      retry: { maxRetries: 0 },
    });
    const criteria = Object.fromEntries(request.candidates.map(candidate => [
      candidate.id,
      candidate.id === 'handback'
        ? '仅当没有可派发的已提交候选时选择交回。'
        : '这是已提交且受支持的候选；当其状态为 true 时可派发该候选。',
    ]));
    const state = Object.fromEntries(request.candidates.map(candidate => [
      candidate.id,
      candidate.dispatchable && candidate.parameter_complete,
    ]));
    const result = await client.systemOne({
      model: 'jev-latest',
      state: { candidates: state },
      questions: {
        next: choice('选择一个候选。必须优先选择值为 true 且不是 handback 的已提交候选；只有不存在这类候选时才选择 handback。', criteria),
      },
    });
    const answer = result.answers.next;
    if (answer.type !== 'choice') throw new Error('unexpected-answer-type');
    process.stdout.write(JSON.stringify({
      candidate_id: answer.choice,
      confidence: answer.confidence,
    }) + '\n');
  } catch {
    process.exitCode = 3;
  }
}
