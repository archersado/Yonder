CREATE TABLE IF NOT EXISTS task_io_contracts (
    task_id TEXT PRIMARY KEY NOT NULL REFERENCES tasks(id),
    input_summary TEXT NOT NULL,
    expected_output TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS task_results (
    task_id TEXT PRIMARY KEY NOT NULL REFERENCES tasks(id),
    result_sequence INTEGER NOT NULL CHECK(result_sequence > 0),
    summary TEXT NOT NULL,
    FOREIGN KEY(task_id, result_sequence) REFERENCES events(task_id, sequence)
);

PRAGMA user_version = 27;
