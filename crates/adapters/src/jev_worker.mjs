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
    const criteria = Object.fromEntries(request.candidates.map(candidate => [candidate.id, null]));
    const state = Object.fromEntries(request.candidates.map(candidate => [
      candidate.id,
      candidate.dispatchable && candidate.parameter_complete,
    ]));
    const result = await client.systemOne({
      model: 'jev-latest',
      state: { candidates: state },
      questions: {
        next: choice('选择数组第二值为 true 的候选；没有 true 候选时选择 handback。', criteria),
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
