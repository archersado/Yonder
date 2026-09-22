CREATE TABLE jev_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    config_json TEXT NOT NULL
);
PRAGMA user_version=16;
