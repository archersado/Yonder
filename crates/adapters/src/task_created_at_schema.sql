ALTER TABLE tasks ADD COLUMN created_at INTEGER NOT NULL DEFAULT 0 CHECK(created_at >= 0);
CREATE INDEX tasks_created_at_id ON tasks(created_at DESC, id DESC);
CREATE INDEX tasks_owner_created_at_id ON tasks(owner_agent_id, created_at DESC, id DESC);
PRAGMA user_version=21;
