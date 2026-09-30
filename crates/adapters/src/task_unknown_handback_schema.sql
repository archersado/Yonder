ALTER TABLE task_attempts ADD COLUMN handback_sequence INTEGER;
DROP INDEX task_attempt_active;
CREATE UNIQUE INDEX task_attempt_active ON task_attempts(task_id) WHERE phase!='stopped' AND handback_sequence IS NULL;
PRAGMA user_version=23;
