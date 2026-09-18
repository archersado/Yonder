CREATE TABLE task_attempts (
    task_id TEXT NOT NULL,
    step_id TEXT NOT NULL,
    attempt_id TEXT NOT NULL,
    worker_instance_id TEXT NOT NULL,
    host_session_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK(phase='prepared'),
    accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
    PRIMARY KEY(task_id,attempt_id),
    UNIQUE(task_id,accepted_sequence),
    FOREIGN KEY(task_id,step_id) REFERENCES task_steps(task_id,step_id),
    FOREIGN KEY(task_id,accepted_sequence) REFERENCES events(task_id,sequence)
);
CREATE UNIQUE INDEX task_attempt_active ON task_attempts(task_id) WHERE phase='prepared';
PRAGMA user_version=7;
