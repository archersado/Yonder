CREATE TABLE task_browser_refs (
  task_id TEXT PRIMARY KEY REFERENCES tasks(id),
  external_task_ref TEXT NOT NULL UNIQUE CHECK(external_task_ref GLOB 'ego:[0-9]*'),
  ownership TEXT NOT NULL CHECK(ownership IN ('agent','agentDelegatedToUser','user')),
  managed_pages INTEGER NOT NULL CHECK(managed_pages BETWEEN 0 AND 100),
  finished INTEGER NOT NULL CHECK(finished IN (0,1)),
  updated_sequence INTEGER NOT NULL,
  FOREIGN KEY(task_id,updated_sequence) REFERENCES events(task_id,sequence)
);
PRAGMA user_version=12;
