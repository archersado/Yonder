CREATE TABLE task_steps (
    task_id TEXT NOT NULL,
    step_id TEXT NOT NULL,
    label TEXT NOT NULL,
    accepted_sequence INTEGER NOT NULL CHECK(accepted_sequence > 0),
    PRIMARY KEY(task_id,step_id),
    UNIQUE(task_id,accepted_sequence),
    FOREIGN KEY(task_id,accepted_sequence) REFERENCES events(task_id,sequence)
);
PRAGMA user_version=6;
