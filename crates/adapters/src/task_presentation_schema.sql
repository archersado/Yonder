ALTER TABLE tasks ADD COLUMN source TEXT NOT NULL DEFAULT 'legacy' CHECK(source IN ('local-agent','cloud-agent','legacy'));
ALTER TABLE tasks ADD COLUMN observation_step_id TEXT;
ALTER TABLE tasks ADD COLUMN observation_result TEXT CHECK(observation_result IS NULL OR observation_result IN ('matched','not-matched','unknown'));
ALTER TABLE tasks ADD COLUMN observation_summary TEXT CHECK(observation_summary IS NULL OR (length(observation_summary)>0 AND length(CAST(observation_summary AS BLOB))<=2048));
ALTER TABLE tasks ADD COLUMN next_intent TEXT CHECK(next_intent IS NULL OR (length(next_intent)>0 AND length(CAST(next_intent AS BLOB))<=1024));
CREATE TABLE task_presentation_events (
    task_id TEXT NOT NULL,
    sequence INTEGER NOT NULL CHECK(sequence > 0),
    kind TEXT NOT NULL CHECK(kind IN ('source','observation','next-intent')),
    payload TEXT NOT NULL,
    PRIMARY KEY(task_id, sequence),
    FOREIGN KEY(task_id, sequence) REFERENCES events(task_id, sequence)
);
PRAGMA user_version=15;
