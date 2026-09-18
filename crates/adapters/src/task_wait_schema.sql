ALTER TABLE events ADD COLUMN wait_reason TEXT CHECK(wait_reason IS NULL OR (length(wait_reason)>0 AND length(CAST(wait_reason AS BLOB))<=512));
PRAGMA user_version=14;
