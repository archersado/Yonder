use super::*;

impl SqliteTaskStore {
    pub fn open_unencrypted(path: &Path) -> Result<Self, Error> {
        Self::initialize(Connection::open(path).map_err(storage)?, true)
    }

    pub fn open(path: &Path, database_key: &[u8; 32]) -> Result<Self, Error> {
        let db = Connection::open(path).map_err(storage)?;
        // SAFETY: 连接句柄有效；同步调用期间 key 的 32 字节保持存活。
        let result =
            unsafe { rusqlite::ffi::sqlite3_key(db.handle(), database_key.as_ptr().cast(), 32) };
        if result != rusqlite::ffi::SQLITE_OK {
            return Err(Error::StorageUnavailable);
        }
        let version: String = db
            .query_row("PRAGMA cipher_version", [], |r| r.get(0))
            .map_err(storage)?;
        if version.is_empty() {
            return Err(Error::StorageUnavailable);
        }
        Self::initialize(db, false)
    }

    pub(super) fn initialize(mut db: Connection, migrate_plaintext: bool) -> Result<Self, Error> {
        // 在任何初始化写入前拒绝不可读格式及未知版本，包括已有加密库。
        let schema: i64 = db
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(storage)?;
        if ![
            0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        ]
        .contains(&schema)
        {
            return Err(Error::StorageUnavailable);
        }
        if schema != 0 && schema != SQLITE_SCHEMA_VERSION {
            if !migrate_plaintext {
                return Err(Error::StorageUnavailable);
            }
            if let Some(path) = db.path().filter(|path| !path.is_empty()) {
                let nonce = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| Error::StorageUnavailable)?
                    .as_nanos();
                let backup = format!(
                    "{path}.pre-attempt-v{schema}-{}-{nonce}.db",
                    std::process::id()
                );
                // SQLite自己复制一致快照；目标已存在或磁盘失败时拒绝升级。
                db.execute("VACUUM INTO ?1", [&backup]).map_err(storage)?;
            }
        }
        db.busy_timeout(Duration::from_secs(2)).map_err(storage)?;
        db.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")
            .map_err(storage)?;
        let tx = db
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let schema: i64 = tx
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(storage)?;
        if schema == 0 {
            let objects: i64 = tx
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name NOT LIKE 'sqlite_%'",
                    [],
                    |r| r.get(0),
                )
                .map_err(storage)?;
            if objects != 0 {
                return Err(Error::StorageUnavailable);
            }
            tx.execute_batch(include_str!("../task_schema.sql"))
                .map_err(storage)?;
        } else if ![
            2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26,
        ]
        .contains(&schema)
        {
            return Err(Error::StorageUnavailable);
        }
        if schema == 0 || schema == 2 {
            tx.execute_batch(include_str!("../task_registration_schema.sql"))
                .map_err(storage)?;
        }
        if schema == 4 {
            // AD-TM-06：仅兼容未使用实验列，绝不清理或伪造恢复记录。
            let unsafe_records: i64 = tx.query_row("SELECT (SELECT count(*) FROM tasks WHERE deleted!=0)+(SELECT count(*) FROM events WHERE kind!='transition')", [], |r| r.get(0)).map_err(storage)?;
            if unsafe_records != 0 {
                return Err(Error::StorageUnavailable);
            }
            tx.execute_batch("ALTER TABLE tasks DROP COLUMN deleted; ALTER TABLE events DROP COLUMN kind; PRAGMA user_version=3;").map_err(storage)?;
        }
        if schema < 5 {
            tx.execute_batch(include_str!("../task_name_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 6 {
            tx.execute_batch(include_str!("../task_step_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 7 {
            tx.execute_batch(include_str!("../task_attempt_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 8 {
            tx.execute_batch(include_str!("../task_attempt_result_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 9 {
            tx.execute_batch(include_str!("../task_stop_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 10 {
            tx.execute_batch(include_str!("../task_control_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 11 {
            tx.execute_batch(include_str!("../task_advance_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 12 {
            tx.execute_batch(include_str!("../task_browser_reference_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 13 {
            tx.execute_batch(include_str!("../task_focus_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 14 {
            tx.execute_batch(include_str!("../task_wait_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 15 {
            tx.execute_batch(include_str!("../task_presentation_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 16 {
            tx.execute_batch(include_str!("../jev_config_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 17 {
            tx.execute_batch(include_str!("../agent_registry_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 18 {
            tx.execute_batch(include_str!("../task_audit_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 19 {
            tx.execute_batch(include_str!("../task_focus_history_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 20 {
            tx.execute_batch(include_str!("../task_plan_fragment_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 21 {
            tx.execute_batch(include_str!("../task_created_at_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 22 {
            tx.execute_batch(include_str!("../task_runtime_projection_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 23 {
            tx.execute_batch(include_str!("../task_unknown_handback_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 24 {
            tx.execute_batch(include_str!("../task_goal_verification_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 25 {
            tx.execute_batch(include_str!("../execution_experience_schema.sql"))
                .map_err(storage)?;
        }
        if schema < 26 {
            tx.execute_batch(include_str!("../task_runtime_control_stop_schema.sql"))
                .map_err(storage)?;
        }
        // AD-TM-21：曾有中断构建把版本号推进到 v20，却完整遗漏 schema 18
        // 的三张业务表。全缺意味着不存在该组业务记录，可以只建空表；部分
        // 缺失可能已经丢失关联事实，必须失败关闭，不能猜测或补造。
        let audit_tables: i64 = tx
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name IN ('task_artifact_manifests','task_artifact_manifest_items','task_user_confirmations')",
                [],
                |row| row.get(0),
            )
            .map_err(storage)?;
        match audit_tables {
            0 => tx
                .execute_batch(include_str!("../task_audit_repair_schema.sql"))
                .map_err(storage)?,
            3 => {}
            _ => return Err(Error::StorageUnavailable),
        }
        // 旧发布曾可能在完成版本号写入后中断，留下标称 v20 但缺少该无业务
        // 数据的单例配额表。只补建可推导的配额状态；任务、事件和 Outbox 一律
        // 不在这里猜测或重建。
        tx.execute_batch(
            "CREATE TABLE IF NOT EXISTS task_audit_quota_state (\
                id INTEGER PRIMARY KEY CHECK(id = 1),\
                max_bytes INTEGER NOT NULL CHECK(max_bytes > 0),\
                reserve_bytes INTEGER NOT NULL CHECK(reserve_bytes > 0)\
            );\
            INSERT INTO task_audit_quota_state(id, max_bytes, reserve_bytes)\
            VALUES (1, 2147483648, 1073741824) ON CONFLICT(id) DO NOTHING;",
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Self(db))
    }
}
