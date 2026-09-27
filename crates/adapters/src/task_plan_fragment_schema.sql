CREATE TABLE IF NOT EXISTS task_plan_fragments (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    plan_id TEXT NOT NULL,
    plan_version INTEGER NOT NULL CHECK(plan_version > 0),
    owner_agent_id TEXT NOT NULL,
    accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
    current_slot INTEGER NOT NULL DEFAULT 0 CHECK(current_slot >= 0),
    fragment_json TEXT NOT NULL,
    PRIMARY KEY(task_id, plan_id, plan_version)
);
CREATE INDEX IF NOT EXISTS task_plan_fragments_task_id ON task_plan_fragments(task_id, accepted_sequence DESC);
PRAGMA user_version=20;
