DROP INDEX task_attempt_active;
ALTER TABLE task_attempts RENAME TO task_attempts_v8;
CREATE TABLE task_attempts (
    task_id TEXT NOT NULL,
    step_id TEXT NOT NULL,
    attempt_id TEXT NOT NULL,
    worker_instance_id TEXT NOT NULL,
    host_session_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK(phase IN ('prepared','observed','unknown','stopped')),
    accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
    result_sequence INTEGER,
    action_succeeded INTEGER CHECK(action_succeeded IN (0,1)),
    observe_valid INTEGER CHECK(observe_valid IN (0,1)),
    unknown_reason TEXT,
    stop_sequence INTEGER,
    control_id TEXT,
    control_kind TEXT CHECK(control_kind IN ('pause','cancel','takeover')),
    PRIMARY KEY(task_id,attempt_id),
    UNIQUE(task_id,accepted_sequence),
    UNIQUE(task_id,result_sequence),
    UNIQUE(task_id,stop_sequence),
    UNIQUE(task_id,control_id),
    FOREIGN KEY(task_id,step_id) REFERENCES task_steps(task_id,step_id),
    FOREIGN KEY(task_id,accepted_sequence) REFERENCES events(task_id,sequence),
    FOREIGN KEY(task_id,result_sequence) REFERENCES events(task_id,sequence),
    FOREIGN KEY(task_id,stop_sequence) REFERENCES events(task_id,sequence),
    CHECK(
      (phase='prepared' AND result_sequence IS NULL AND action_succeeded IS NULL AND observe_valid IS NULL AND unknown_reason IS NULL AND stop_sequence IS NULL AND control_id IS NULL AND control_kind IS NULL) OR
      (phase='observed' AND result_sequence IS NOT NULL AND action_succeeded IS NOT NULL AND observe_valid=1 AND unknown_reason IS NULL AND stop_sequence IS NULL AND control_id IS NULL AND control_kind IS NULL) OR
      (phase='unknown' AND result_sequence IS NOT NULL AND action_succeeded IS NULL AND observe_valid=0 AND unknown_reason IS NOT NULL AND stop_sequence IS NULL AND control_id IS NULL AND control_kind IS NULL) OR
      (phase='stopped' AND result_sequence IS NOT NULL AND action_succeeded IS NOT NULL AND observe_valid=1 AND unknown_reason IS NULL AND stop_sequence IS NOT NULL AND control_id IS NOT NULL AND control_kind IS NOT NULL)
    )
);
INSERT INTO task_attempts(task_id,step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,result_sequence,action_succeeded,observe_valid,unknown_reason)
SELECT task_id,step_id,attempt_id,worker_instance_id,host_session_id,phase,accepted_sequence,result_sequence,action_succeeded,observe_valid,unknown_reason FROM task_attempts_v8;
DROP TABLE task_attempts_v8;
CREATE UNIQUE INDEX task_attempt_active ON task_attempts(task_id) WHERE phase!='stopped';
PRAGMA user_version=9;
