CREATE TABLE IF NOT EXISTS execution_experiences (
    platform TEXT NOT NULL,
    driver TEXT NOT NULL,
    application_id TEXT NOT NULL,
    action_kind TEXT NOT NULL,
    control_role TEXT NOT NULL,
    interaction TEXT NOT NULL,
    precondition TEXT NOT NULL,
    success_condition TEXT NOT NULL,
    success_count INTEGER NOT NULL CHECK(success_count > 0),
    failure_count INTEGER NOT NULL DEFAULT 0 CHECK(failure_count >= 0),
    last_verified_at_ms INTEGER NOT NULL CHECK(last_verified_at_ms > 0),
    PRIMARY KEY(platform,driver,application_id,action_kind,control_role,interaction,precondition,success_condition)
);
CREATE INDEX IF NOT EXISTS execution_experiences_recall ON execution_experiences(platform,driver,application_id,action_kind,control_role,success_count DESC,last_verified_at_ms DESC);
PRAGMA user_version=25;
