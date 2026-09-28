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
        ? '仅当不存在前置条件满足、参数完整且可派发的候选时选择交回。'
        : `已提交动作：${candidate.action_kind}；目标引用：${candidate.target_ref}；前置 Observe：${candidate.preconditions.join(', ')}；预期 Observe：${candidate.expected_observe.join(', ')}。仅当前置条件满足且状态为 true 时可派发。`,
    ]));
    const state = Object.fromEntries(request.candidates.map(candidate => [
      candidate.id,
      {
        dispatchable: candidate.dispatchable && candidate.parameter_complete,
        action_kind: candidate.action_kind,
        target_ref: candidate.target_ref,
        preconditions: candidate.preconditions,
        expected_observe: candidate.expected_observe,
      },
    ]));
    const result = await client.systemOne({
      model: 'jev-latest',
      state: { candidates: state },
      questions: {
        next: choice('基于候选的动作语义、前置 Observe 和预期 Observe 选择下一步。只能选择前置满足且 dispatchable 为 true 的已提交候选；否则选择 handback。不得推断或索取未提供的联系人、消息正文、坐标、控件或 Driver 参数。', criteria),
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
