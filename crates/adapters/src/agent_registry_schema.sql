CREATE TABLE IF NOT EXISTS agent_registry (
  agent_id TEXT PRIMARY KEY,
  status TEXT NOT NULL CHECK (status IN ('enabled','disabled','revoked')),
  registered_at INTEGER NOT NULL,
  last_seen_at INTEGER,
  updated_at INTEGER NOT NULL
);
PRAGMA user_version=17;
