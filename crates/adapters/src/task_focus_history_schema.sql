CREATE TABLE task_focus_events (
    task_id TEXT NOT NULL,
    sequence INTEGER NOT NULL CHECK(sequence > 0),
    control_id TEXT NOT NULL,
    phase TEXT NOT NULL CHECK(phase IN ('locating','focused','failed')),
    failure TEXT,
    PRIMARY KEY(task_id, sequence),
    UNIQUE(task_id, control_id, phase),
    FOREIGN KEY(task_id, sequence) REFERENCES events(task_id, sequence),
    FOREIGN KEY(task_id, control_id) REFERENCES task_controls(task_id, control_id),
    CHECK((phase='failed' AND failure IS NOT NULL) OR (phase!='failed' AND failure IS NULL))
);
PRAGMA user_version=19;
