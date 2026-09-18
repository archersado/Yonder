CREATE TABLE task_creations (
    owner_agent_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    task_id TEXT NOT NULL REFERENCES tasks(id),
    description TEXT NOT NULL,
    PRIMARY KEY(owner_agent_id,idempotency_key)
);
PRAGMA user_version=3;
