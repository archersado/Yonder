CREATE TABLE tasks (
    id TEXT PRIMARY KEY NOT NULL,
    state TEXT NOT NULL,
    sequence INTEGER NOT NULL CHECK(sequence > 0)
);
CREATE TABLE events (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    sequence INTEGER NOT NULL CHECK(sequence > 0),
    previous TEXT NOT NULL,
    state TEXT NOT NULL,
    PRIMARY KEY(task_id, sequence)
);
CREATE TABLE outbox (
    task_id TEXT NOT NULL,
    sequence INTEGER NOT NULL,
    delivered INTEGER NOT NULL DEFAULT 0 CHECK(delivered IN (0,1)),
    PRIMARY KEY(task_id, sequence),
    FOREIGN KEY(task_id, sequence) REFERENCES events(task_id, sequence)
);
PRAGMA user_version=1;
