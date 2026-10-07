CREATE TABLE IF NOT EXISTS task_goal_verifications (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    verification_id TEXT NOT NULL,
    observation_sequence INTEGER NOT NULL CHECK(observation_sequence > 0),
    verified_sequence INTEGER NOT NULL CHECK(verified_sequence > observation_sequence),
    outcome TEXT NOT NULL CHECK(outcome IN ('achieved','not-achieved')),
    PRIMARY KEY(task_id, verified_sequence),
    UNIQUE(task_id, verification_id),
    FOREIGN KEY(task_id, verified_sequence) REFERENCES events(task_id, sequence)
);
CREATE INDEX IF NOT EXISTS task_goal_verifications_latest ON task_goal_verifications(task_id, verified_sequence DESC);
PRAGMA user_version=24;
