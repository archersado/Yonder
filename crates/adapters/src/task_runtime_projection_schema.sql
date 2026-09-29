CREATE TABLE task_runtime_projections (
    task_id TEXT NOT NULL,
    sequence INTEGER NOT NULL CHECK(sequence > 0),
    payload TEXT NOT NULL,
    PRIMARY KEY(task_id, sequence),
    FOREIGN KEY(task_id, sequence) REFERENCES events(task_id, sequence)
);
PRAGMA user_version=22;
