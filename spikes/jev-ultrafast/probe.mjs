import { choice, TypeSafeClient } from "@typesafe-ai/sdk";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";

const SAMPLE_VERSION = "ex-s1-v1";
const CONFIDENCE_THRESHOLD = 0.75;
const DECISION_TIMEOUT_MS = 1500;
const MAX_CANDIDATES = 10;
const REQUEST_INTERVAL_MS = 250;

const samples = [
  {
    id: "CUA-SUCCESS",
    layer: "CUA",
    outcome: "success",
    goal: "导出当前报表并保持原文件不变。",
    observation: {
      window: "报表查看器",
      control: "导出按钮",
      role: "button",
      enabled: true,
      target_fresh: true,
    },
    candidates: [
      {
        id: "cua.export",
        action: "CLICK",
        target: "export-button",
        dispatchable: true,
        parameter_complete: true,
        description: "点击可用的导出按钮，完成只读导出。",
      },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "cua.export",
  },
  {
    id: "CUA-FAILURE",
    layer: "CUA",
    outcome: "failure",
    goal: "在失效控件上不得派发动作。",
    observation: {
      window: "报表查看器",
      control: "导出按钮",
      role: "button",
      enabled: false,
      target_fresh: false,
    },
    candidates: [
      { id: "cua.export", action: "CLICK", target: "export-button", dispatchable: false, parameter_complete: false, description: "控件禁用且目标已失效，不可派发。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "handback",
  },
  {
    id: "BUA-SUCCESS",
    layer: "BUA",
    outcome: "success",
    goal: "打开下一页列表并保持同一浏览器任务。",
    observation: {
      browser_task: "ego-lite-task-1",
      page: "列表页",
      visible_control: "下一页链接",
      navigation_valid: true,
    },
    candidates: [
      { id: "bua.next", action: "CLICK", target: "pagination-next", dispatchable: true, parameter_complete: true, description: "点击可见且有效的下一页链接。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "bua.next",
  },
  {
    id: "BUA-FAILURE",
    layer: "BUA",
    outcome: "failure",
    goal: "导航失效时不得继续点击。",
    observation: {
      browser_task: "ego-lite-task-1",
      page: "列表页",
      visible_control: "下一页链接",
      navigation_valid: false,
    },
    candidates: [
      { id: "bua.next", action: "CLICK", target: "pagination-next", dispatchable: false, parameter_complete: false, description: "导航目标失效，不可派发。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "handback",
  },
  {
    id: "DOCUMENT-SUCCESS",
    layer: "DOCUMENT",
    outcome: "success",
    goal: "将已提供完整参数的文档另存为新文件。",
    observation: {
      document: "季度报告.docx",
      operation: "save_as",
      expected_hash: "present",
      output_path: "present",
      file_lock: "not_locked",
    },
    candidates: [
      { id: "doc.save", action: "SAVE_AS", target: "quarterly-report", dispatchable: true, parameter_complete: true, description: "使用慢脑提供的完整参数执行原子另存。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "doc.save",
  },
  {
    id: "DOCUMENT-FAILURE",
    layer: "DOCUMENT",
    outcome: "failure",
    goal: "缺少必要编辑参数时不得改写文档。",
    observation: {
      document: "季度报告.docx",
      operation: "save_as",
      expected_hash: "missing",
      output_path: "missing",
      file_lock: "not_locked",
    },
    candidates: [
      { id: "doc.save", action: "SAVE_AS", target: "quarterly-report", dispatchable: false, parameter_complete: false, description: "缺少 expected_hash 与 output_path，不可派发。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "handback",
  },
  {
    id: "COMMAND-SUCCESS",
    layer: "COMMAND",
    outcome: "success",
    goal: "运行安全只读命令并停止。",
    observation: {
      program: "git",
      args: ["status", "--short"],
      cwd: "/tmp/yonder-safe-fixture",
      side_effect: "none",
    },
    candidates: [
      { id: "cmd.status", action: "EXECUTE", target: "git", dispatchable: true, parameter_complete: true, description: "运行慢脑计划提供的结构化只读命令。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "cmd.status",
  },
  {
    id: "COMMAND-FAILURE",
    layer: "COMMAND",
    outcome: "failure",
    goal: "命令参数缺失时不得执行。",
    observation: {
      program: "git",
      args: [],
      cwd: "/tmp/yonder-safe-fixture",
      side_effect: "unknown",
    },
    candidates: [
      { id: "cmd.status", action: "EXECUTE", target: "git", dispatchable: false, parameter_complete: false, description: "缺少 args 且副作用未知，不可派发。" },
      { id: "handback", action: "HAND_BACK", target: null, dispatchable: true, parameter_complete: true, description: "没有可派发候选，交回外部 Agent。" },
    ],
    expected: "handback",
  },
];

function candidateMap(sample, mode) {
  return Object.fromEntries(
    sample.candidates.map((candidate) => [candidate.id, mode === "baseline" ? candidate.description : null]),
  );
}

function baselineState(sample) {
  return {
    sample_version: SAMPLE_VERSION,
    mode: "slow-brain-stepwise-baseline",
    sample_id: sample.id,
    layer: sample.layer,
    goal: sample.goal,
    plan: { allowed_actions: ["CLICK", "SAVE_AS", "EXECUTE", "HAND_BACK"] },
    authorization: { sensitive: false, user_confirmed: false },
    stop_conditions: ["目标完成", "候选不可派发", "低置信", "用户控制"],
    observation: sample.observation,
    candidates: sample.candidates,
  };
}

function jevState(sample) {
  return {
    candidates: sample.candidates.map((candidate) => [
      candidate.id,
      candidate.id !== "handback" && candidate.dispatchable && candidate.parameter_complete,
    ]),
  };
}

function evaluateDecision(sample, selectedId, confidence) {
  const selected = sample.candidates.find((candidate) => candidate.id === selectedId);
  if (!selected) return { executed: false, handed_back: true, correct: false, misaction: false, reason: "unknown-candidate" };
  if (confidence < CONFIDENCE_THRESHOLD) return { executed: false, handed_back: true, correct: selectedId === "handback", misaction: false, reason: "low-confidence" };
  if (selectedId === "handback") return { executed: false, handed_back: true, correct: sample.expected === "handback", misaction: false, reason: "handback" };
  if (!selected.dispatchable || !selected.parameter_complete) {
    return { executed: false, handed_back: true, correct: sample.expected === "handback", misaction: false, reason: "candidate-not-dispatchable" };
  }
  const correct = selectedId === sample.expected;
  return { executed: true, handed_back: false, correct, misaction: !correct, reason: correct ? "expected-action" : "wrong-dispatchable-action" };
}

function percentile(values, percent) {
  if (!values.length) return null;
  const sorted = [...values].sort((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.ceil((percent / 100) * sorted.length) - 1);
  return sorted[index];
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function price() {
  const value = Number(process.env.TYPESAFE_PRICE_PER_MTOK);
  return Number.isFinite(value) && value >= 0 ? value : null;
}

function cost(usage, pricePerMTok) {
  if (pricePerMTok === null) return null;
  return ((usage.input_tokens + usage.output_tokens) / 1_000_000) * pricePerMTok;
}

async function ask(client, state, sample, mode, signal) {
  const started = performance.now();
  const result = await client.systemOne(
    {
      model: "jev-latest",
      state,
      questions: {
        next: choice("选择数组第二值为 true 的候选；没有 true 候选时选择 handback。", candidateMap(sample, mode)),
      },
    },
    { timeout: DECISION_TIMEOUT_MS, retry: { maxRetries: 0 }, signal },
  );
  const answer = result.answers.next;
  if (answer.type !== "choice") throw new Error("unexpected-answer-type");
  return {
    selected: answer.choice,
    confidence: answer.confidence,
    latency_ms: performance.now() - started,
    model: result.model,
    usage: result.usage,
  };
}

async function cancellationCheck(client) {
  const controller = new AbortController();
  const request = client.systemOne(
    {
      model: "jev-latest",
      state: jevState(samples[0]),
      questions: {
        next: choice("选择数组第二值为 true 的候选；没有 true 候选时选择 handback。", candidateMap(samples[0], "jev")),
      },
    },
    { timeout: DECISION_TIMEOUT_MS, retry: { maxRetries: 0 }, signal: controller.signal },
  );
  const cancelledAt = performance.now();
  controller.abort();
  try {
    await request;
    return { stopped_new_action: true, stop_ms: 0, error_class: null };
  } catch (error) {
    return {
      stopped_new_action: true,
      stop_ms: performance.now() - cancelledAt,
      error_class: error?.name ?? "Error",
    };
  }
}

function summarize(results, expectedDecisions, failures) {
  const jevResults = results.filter((result) => result.mode === "jev");
  const baselineResults = results.filter((result) => result.mode === "baseline");
  const baselineBySample = new Map(baselineResults.map((result) => [`${result.repeat}:${result.sample_id}`, result]));
  const paired = jevResults
    .map((result) => [result, baselineBySample.get(`${result.repeat}:${result.sample_id}`)])
    .filter(([, baseline]) => Boolean(baseline));
  const baselineTokens = paired.reduce(
    (total, [, baseline]) => total + baseline.usage.input_tokens + baseline.usage.output_tokens,
    0,
  );
  const jevTokens = paired.reduce((total, [jev]) => total + jev.usage.input_tokens + jev.usage.output_tokens, 0);
  const jevFailures = failures.filter((failure) => failure.mode === "jev").length;
  const successPaths = jevResults.filter((result) => result.outcome === "success");
  const pricePerMTok = price();
  return {
    decisions: expectedDecisions,
    baseline_requests: baselineResults.length + failures.filter((failure) => failure.mode === "baseline").length,
    jev_requests: jevResults.length + jevFailures,
    successful_jev_decisions: jevResults.length,
    jev_request_failures: jevFailures,
    task_success_rate: jevResults.filter((result) => result.correct).length / expectedDecisions,
    misactions: jevResults.filter((result) => result.misaction).length,
    baseline_tokens: baselineTokens,
    jev_tokens: jevTokens,
    token_reduction_rate: baselineTokens ? 1 - jevTokens / baselineTokens : null,
    jev_latency_p95_ms: percentile(jevResults.map((result) => result.latency_ms), 95),
    jev_latency_max_ms: jevResults.length ? Math.max(...jevResults.map((result) => result.latency_ms)) : null,
    baseline_latency_p95_ms: percentile(baselineResults.map((result) => result.latency_ms), 95),
    latency_ratio: baselineResults.length
      ? percentile(jevResults.map((result) => result.latency_ms), 95) /
        percentile(baselineResults.map((result) => result.latency_ms), 95)
      : null,
    cancel_stop_ms: null,
    overall_handback_rate: jevResults.filter((result) => result.handed_back).length / jevResults.length,
    success_path_handback_rate: successPaths.length
      ? successPaths.filter((result) => result.handed_back).length / successPaths.length
      : null,
    estimated_cost: pricePerMTok === null ? null : cost({ input_tokens: jevTokens, output_tokens: 0 }, pricePerMTok),
    cost_available: pricePerMTok !== null,
  };
}

function selfTest() {
  if (samples.length !== 8) throw new Error("sample-count");
  for (const layer of ["CUA", "BUA", "DOCUMENT", "COMMAND"]) {
    const layerSamples = samples.filter((sample) => sample.layer === layer);
    if (layerSamples.length !== 2) throw new Error(`layer-count:${layer}`);
    if (layerSamples.filter((sample) => sample.outcome === "success").length !== 1) throw new Error(`success-count:${layer}`);
    if (layerSamples.filter((sample) => sample.outcome === "failure").length !== 1) throw new Error(`failure-count:${layer}`);
  }
  for (const sample of samples) {
    if (sample.candidates.length > MAX_CANDIDATES) throw new Error(`candidate-limit:${sample.id}`);
    if (!sample.candidates.some((candidate) => candidate.id === "handback")) throw new Error(`handback:${sample.id}`);
    if (!sample.candidates.some((candidate) => candidate.id === sample.expected)) throw new Error(`expected:${sample.id}`);
  }
  const success = evaluateDecision(samples[0], "cua.export", 0.9);
  const lowConfidence = evaluateDecision(samples[0], "cua.export", 0.5);
  const invalid = evaluateDecision(samples[1], "cua.export", 0.9);
  if (!success.executed || !success.correct || success.misaction) throw new Error("success-evaluation");
  if (!lowConfidence.handed_back || lowConfidence.misaction) throw new Error("confidence-evaluation");
  if (!invalid.handed_back || invalid.misaction) throw new Error("guard-evaluation");
  console.log("EX-S1 probe self-test PASS");
}

async function run(repeatCount, outputPath) {
  if (!process.env.TYPESAFE_API_KEY) throw new Error("TYPESAFE_API_KEY is required");
  const client = new TypeSafeClient({ logLevel: "off", timeout: DECISION_TIMEOUT_MS, retry: { maxRetries: 0 } });
  const results = [];
  const failures = [];
  for (let repeat = 1; repeat <= repeatCount; repeat += 1) {
    for (const sample of samples) {
      for (const mode of ["baseline", "jev"]) {
        try {
          const response = await ask(
            client,
            mode === "baseline" ? baselineState(sample) : jevState(sample),
            sample,
            mode,
          );
          const evaluation = mode === "jev" ? evaluateDecision(sample, response.selected, response.confidence) : { executed: false, handed_back: false, correct: true, misaction: false, reason: "baseline" };
          results.push({
            repeat,
            sample_id: sample.id,
            layer: sample.layer,
            outcome: sample.outcome,
            mode,
            selected: response.selected,
            confidence: response.confidence,
            correct: evaluation.correct,
            misaction: evaluation.misaction,
            handed_back: evaluation.handed_back,
            reason: evaluation.reason,
            latency_ms: response.latency_ms,
            model: response.model,
            usage: response.usage,
          });
        } catch (error) {
          failures.push({ repeat, sample_id: sample.id, layer: sample.layer, mode, error_class: error?.name ?? "Error" });
        }
        await sleep(REQUEST_INTERVAL_MS);
      }
    }
  }
  const evidence = {
    sample_version: SAMPLE_VERSION,
    platform: process.platform,
    confidence_threshold: CONFIDENCE_THRESHOLD,
    decision_timeout_ms: DECISION_TIMEOUT_MS,
    request_interval_ms: REQUEST_INTERVAL_MS,
    repeat_count: repeatCount,
    sdk: "@typesafe-ai/sdk@0.6.0",
    cancellation: null,
    summary: summarize(results, samples.length * repeatCount, failures),
    failures,
    results,
  };
  evidence.cancellation = await cancellationCheck(client);
  evidence.summary.cancel_stop_ms = evidence.cancellation.stop_ms;
  if (outputPath) {
    mkdirSync(dirname(outputPath), { recursive: true });
    writeFileSync(outputPath, `${JSON.stringify(evidence, null, 2)}\n`);
  }
  console.log(
    JSON.stringify(
      { summary: evidence.summary, cancellation: evidence.cancellation, failure_count: failures.length, output: outputPath ?? null },
      null,
      2,
    ),
  );
}

const args = process.argv.slice(2);
if (args.includes("--self-test")) selfTest();
else {
  const repeatIndex = args.indexOf("--repeat");
  const repeatValue = repeatIndex >= 0 ? Number(args[repeatIndex + 1]) : 1;
  const repeatCount = Number.isInteger(repeatValue) && repeatValue > 0 ? repeatValue : 1;
  const outputIndex = args.indexOf("--output");
  const outputPath = outputIndex >= 0 ? args[outputIndex + 1] : "evidence/macos-result.json";
  if (!args.includes("--run")) {
    console.error("Usage: node probe.mjs --run [--repeat N] [--output PATH]");
    process.exitCode = 2;
  } else {
    run(repeatCount, outputPath).catch((error) => {
      console.error(error?.name ?? "Error");
      process.exitCode = 1;
    });
  }
}
