ALTER TABLE tasks ADD COLUMN name TEXT;
ALTER TABLE task_creations ADD COLUMN name TEXT;
PRAGMA user_version=5;
