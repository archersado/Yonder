ALTER TABLE task_controls ADD COLUMN focus_phase TEXT CHECK(focus_phase IN ('locating','focused','failed'));
ALTER TABLE task_controls ADD COLUMN focus_failure TEXT;
ALTER TABLE task_controls ADD COLUMN focus_sequence INTEGER;
PRAGMA user_version=13;
