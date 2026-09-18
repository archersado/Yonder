CREATE TABLE task_controls (
  task_id TEXT PRIMARY KEY REFERENCES tasks(id),
  attempt_id TEXT NOT NULL,
  control_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('pause','cancel','takeover')),
  phase TEXT NOT NULL CHECK(phase IN ('pending','stopped')),
  accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
  stopped_sequence INTEGER,
  FOREIGN KEY(task_id,attempt_id) REFERENCES task_attempts(task_id,attempt_id),
  FOREIGN KEY(task_id,accepted_sequence) REFERENCES events(task_id,sequence),
  FOREIGN KEY(task_id,stopped_sequence) REFERENCES events(task_id,sequence),
  UNIQUE(task_id,control_id),
  CHECK((phase='pending' AND stopped_sequence IS NULL) OR (phase='stopped' AND stopped_sequence IS NOT NULL))
);
PRAGMA user_version=10;
