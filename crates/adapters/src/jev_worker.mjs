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
      `已提交动作：${candidate.action_kind}；目标引用：${candidate.target_ref}；前置 Observe 已由 Yonder 验证：${candidate.preconditions.join(', ')}；预期 Observe：${candidate.expected_observe.join(', ')}。该候选可直接派发。`,
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
      state: { observed_facts: request.observed_facts, candidates: state },
      questions: {
        next: choice('Yonder 已依据本轮真实 Observe 筛除不合格候选。请在已提交的可派发候选中快速选择下一步；语义等价时选择第一个。不得推断或索取未提供的联系人、消息正文、坐标、控件或 Driver 参数。低置信、未知返回或调用异常由 Yonder Application 统一交回慢脑。', criteria),
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
