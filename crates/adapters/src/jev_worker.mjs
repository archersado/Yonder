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
    if (request.mode === 'verify-step') {
      const result = await client.systemOne({
        model: request.model,
        state: {
          step_intent: request.step_label,
          action_kind: request.action_kind,
          expected_observe: request.expected_observe,
          observed_facts: request.observed_facts,
          observation_summary: request.observation_summary,
        },
        questions: {
          completion: choice(
            '只判断当前步骤是否已由最新观察证明完成。动作发生或界面变化本身不等于步骤完成；观察明确与步骤不符时选 incomplete，证据不足时选 uncertain。不得判断整个用户任务，不得生成下一动作。',
            {
              complete: '最新结构事实和观察摘要明确证明当前步骤意图已经成立。',
              incomplete: '最新观察明确证明当前步骤尚未成立或结果与步骤目标不符。',
              uncertain: '现有观察不足以可靠判断当前步骤是否完成。',
            },
          ),
        },
      });
      const answer = result.answers.completion;
      if (answer.type !== 'choice') throw new Error('unexpected-answer-type');
      process.stdout.write(JSON.stringify({ outcome: answer.choice, confidence: answer.confidence }) + '\n');
      continue;
    }
    if (request.mode !== 'choose') throw new Error('unexpected-mode');
    const criteria = Object.fromEntries(request.candidates.map(candidate => [
      candidate.id,
      `任务整体目标：${request.task_goal}；当前步骤：第 ${request.current_step_index + 1}/${request.plan_steps.length} 步「${request.step_label}」；完整计划步骤：${request.plan_steps.map((step, index) => `${index + 1}.${step}`).join(' → ')}；已提交动作：${candidate.action_kind}；控件角色：${candidate.element_role ?? '未提供'}；控件短标签：${candidate.element_label ?? '未提供'}；目标引用：${candidate.target_ref}；前置 Observe 已由 Yonder 验证：${candidate.preconditions.join(', ')}；预期 Observe：${candidate.expected_observe.join(', ')}。该候选可直接派发。`,
    ]));
    const state = Object.fromEntries(request.candidates.map(candidate => [
      candidate.id,
      {
        dispatchable: candidate.dispatchable && candidate.parameter_complete,
        action_kind: candidate.action_kind,
        target_ref: candidate.target_ref,
        preconditions: candidate.preconditions,
        expected_observe: candidate.expected_observe,
        element_role: candidate.element_role,
        element_label: candidate.element_label,
      },
    ]));
    const result = await client.systemOne({
      model: request.model,
      state: {
        task_goal: request.task_goal,
        plan_steps: request.plan_steps,
        current_step_index: request.current_step_index,
        step_intent: request.step_label,
        observed_facts: request.observed_facts,
        candidates: state,
      },
      questions: {
        next: choice('结合任务整体目标、完整计划步骤和当前步骤位置，在 Yonder 已依据本轮真实 Observe 筛出的可派发候选中选择最能完成当前步骤且服务于整体目标的控件。不要仅凭“播放”“确认”等通用短标签选择全局控件；语义等价时选择第一个。不得推断或索取未提供的联系人、消息正文、坐标、控件或 Driver 参数。低置信、未知返回或调用异常由 Yonder Application 统一交回慢脑。', criteria),
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
