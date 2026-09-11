// 从 Rust 自动生成，请勿手改。
export type Version = "2.0";
export type Capability = "task.read";
export type GetParams = { agent_id: string, capability: Capability, deadline: number, task_id: string, };
export type EventsParams = { agent_id: string, capability: Capability, deadline: number, task_id: string, after_sequence: string, limit: number, };
export type Request = { "method": "task.get", jsonrpc: Version, id: string, params: GetParams, } | { "method": "task.events", jsonrpc: Version, id: string, params: EventsParams, };
export type TaskStatus = "created" | "running" | "waiting-for-user" | "paused" | "interrupted" | "completed" | "failed" | "cancelled";
export type TaskSnapshot = { task_id: string, status: TaskStatus, sequence: string, };
export type TaskEvent = { previous: TaskStatus, status: TaskStatus, sequence: string, };
export type QueryResult = { "kind": "snapshot", task: TaskSnapshot, } | { "kind": "events", task_id: string, events: Array<TaskEvent>, };
export type RpcError = { code: number, message: string, };
export type Response = { jsonrpc: Version, id: string, result: QueryResult, } | { jsonrpc: Version, id: string | null, error: RpcError, };
