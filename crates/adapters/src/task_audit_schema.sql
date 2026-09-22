CREATE TABLE task_artifact_manifests (
    task_id TEXT NOT NULL REFERENCES tasks(id),
    version INTEGER NOT NULL CHECK(version > 0),
    created_sequence INTEGER NOT NULL CHECK(created_sequence > 0),
    item_count INTEGER NOT NULL CHECK(item_count >= 0),
    PRIMARY KEY(task_id, version)
);
CREATE TABLE task_artifact_manifest_items (
    task_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    ordinal INTEGER NOT NULL CHECK(ordinal > 0),
    reference_id TEXT NOT NULL,
    availability TEXT NOT NULL CHECK(availability IN ('available','missing','changed','unverified')),
    PRIMARY KEY(task_id, version, ordinal),
    FOREIGN KEY(task_id, version) REFERENCES task_artifact_manifests(task_id, version)
);
CREATE TABLE task_user_confirmations (
    confirmation_id TEXT PRIMARY KEY,
    task_id TEXT NOT NULL UNIQUE REFERENCES tasks(id),
    result_sequence INTEGER NOT NULL CHECK(result_sequence > 0),
    confirmation_sequence INTEGER NOT NULL UNIQUE CHECK(confirmation_sequence > 0),
    manifest_version INTEGER NOT NULL CHECK(manifest_version > 0),
    comment TEXT CHECK(comment IS NULL OR (length(comment) > 0 AND length(CAST(comment AS BLOB)) <= 2048)),
    confirmed_by TEXT NOT NULL,
    FOREIGN KEY(task_id, confirmation_sequence) REFERENCES events(task_id, sequence),
    FOREIGN KEY(task_id, manifest_version) REFERENCES task_artifact_manifests(task_id, version)
);
CREATE TABLE task_audit_quota_state (
    id INTEGER PRIMARY KEY CHECK(id = 1),
    max_bytes INTEGER NOT NULL CHECK(max_bytes > 0),
    reserve_bytes INTEGER NOT NULL CHECK(reserve_bytes > 0)
);
INSERT INTO task_audit_quota_state(id, max_bytes, reserve_bytes)
VALUES (1, 2147483648, 1073741824)
ON CONFLICT(id) DO NOTHING;
PRAGMA user_version=16;
