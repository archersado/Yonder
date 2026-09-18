CREATE TABLE task_attempts_v11 (
    task_id TEXT NOT NULL, step_id TEXT NOT NULL, attempt_id TEXT NOT NULL,
    worker_instance_id TEXT NOT NULL, host_session_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK(phase IN ('prepared','observed','unknown','stopped')),
    accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
    result_sequence INTEGER, action_succeeded INTEGER CHECK(action_succeeded IN (0,1)),
    observe_valid INTEGER CHECK(observe_valid IN (0,1)), unknown_reason TEXT,
    stop_sequence INTEGER, control_id TEXT,
    control_kind TEXT CHECK(control_kind IN ('pause','cancel','takeover')),
    PRIMARY KEY(task_id,attempt_id),
    UNIQUE(task_id,accepted_sequence), UNIQUE(task_id,result_sequence),
    UNIQUE(task_id,stop_sequence), UNIQUE(task_id,control_id),
    FOREIGN KEY(task_id,step_id) REFERENCES task_steps(task_id,step_id),
    FOREIGN KEY(task_id,accepted_sequence) REFERENCES events(task_id,sequence),
    FOREIGN KEY(task_id,result_sequence) REFERENCES events(task_id,sequence),
    FOREIGN KEY(task_id,stop_sequence) REFERENCES events(task_id,sequence),
    CHECK(
      (phase='prepared' AND result_sequence IS NULL AND action_succeeded IS NULL AND observe_valid IS NULL AND unknown_reason IS NULL AND stop_sequence IS NULL AND control_id IS NULL AND control_kind IS NULL) OR
      (phase='observed' AND result_sequence IS NOT NULL AND action_succeeded IS NOT NULL AND observe_valid=1 AND unknown_reason IS NULL AND stop_sequence IS NULL AND control_id IS NULL AND control_kind IS NULL) OR
      (phase='unknown' AND result_sequence IS NOT NULL AND action_succeeded IS NULL AND observe_valid=0 AND unknown_reason IS NOT NULL AND stop_sequence IS NULL AND control_id IS NULL AND control_kind IS NULL) OR
      (phase='stopped' AND result_sequence IS NOT NULL AND action_succeeded IS NOT NULL AND observe_valid=1 AND unknown_reason IS NULL AND stop_sequence IS NOT NULL AND ((control_id IS NULL AND control_kind IS NULL) OR (control_id IS NOT NULL AND control_kind IS NOT NULL)))
    )
);
INSERT INTO task_attempts_v11 SELECT * FROM task_attempts;
CREATE TABLE task_controls_v11 (
  task_id TEXT PRIMARY KEY REFERENCES tasks(id),
  attempt_id TEXT NOT NULL, control_id TEXT NOT NULL,
  kind TEXT NOT NULL CHECK(kind IN ('pause','cancel','takeover')),
  phase TEXT NOT NULL CHECK(phase IN ('pending','stopped')),
  accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
  stopped_sequence INTEGER,
  FOREIGN KEY(task_id,attempt_id) REFERENCES task_attempts_v11(task_id,attempt_id),
  FOREIGN KEY(task_id,accepted_sequence) REFERENCES events(task_id,sequence),
  FOREIGN KEY(task_id,stopped_sequence) REFERENCES events(task_id,sequence),
  UNIQUE(task_id,control_id),
  CHECK((phase='pending' AND stopped_sequence IS NULL) OR (phase='stopped' AND stopped_sequence IS NOT NULL))
);
INSERT INTO task_controls_v11 SELECT * FROM task_controls;
DROP TABLE task_controls;
DROP INDEX task_attempt_active;
DROP TABLE task_attempts;
ALTER TABLE task_attempts_v11 RENAME TO task_attempts;
ALTER TABLE task_controls_v11 RENAME TO task_controls;
CREATE UNIQUE INDEX task_attempt_active ON task_attempts(task_id) WHERE phase!='stopped';
PRAGMA user_version=11;
