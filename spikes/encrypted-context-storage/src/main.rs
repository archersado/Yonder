use aes_gcm::{Aes256Gcm, KeyInit, aead::Aead};
use rusqlite::{Connection, params};
use std::{error::Error, fs, path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let evidence = PathBuf::from("evidence");
    fs::create_dir_all(&evidence)?;
    let db_path = evidence.join("context.db");
    let _ = fs::remove_file(&db_path);
    let started = Instant::now();

    let mut db = Connection::open(&db_path)?;
    db.pragma_update(None, "key", "yonder-e0-s6-test-key")?;
    let cipher_version: String = db.query_row("PRAGMA cipher_version", [], |row| row.get(0))?;
    if cipher_version.is_empty() {
        return Err("未加载 SQLCipher".into());
    }
    db.execute_batch(
        "CREATE TABLE tasks(id TEXT PRIMARY KEY, state TEXT NOT NULL);
         CREATE TABLE events(task_id TEXT NOT NULL, sequence INTEGER NOT NULL, kind TEXT NOT NULL, UNIQUE(task_id, sequence));
         CREATE TABLE outbox(task_id TEXT NOT NULL, sequence INTEGER NOT NULL, payload TEXT NOT NULL, UNIQUE(task_id, sequence));
         CREATE VIRTUAL TABLE context_fts USING fts5(content, tokenize='trigram');",
    )?;
    db.execute(
        "INSERT INTO context_fts(content) VALUES (?1), (?2)",
        params!["季度产品路线图", "browser automation report"],
    )?;
    let chinese_hits: i64 = db.query_row(
        "SELECT count(*) FROM context_fts WHERE context_fts MATCH '产品路'",
        [],
        |row| row.get(0),
    )?;
    let english_hits: i64 = db.query_row(
        "SELECT count(*) FROM context_fts WHERE context_fts MATCH 'automation'",
        [],
        |row| row.get(0),
    )?;

    {
        let tx = db.transaction()?;
        tx.execute("INSERT INTO tasks VALUES ('task-1', 'running')", [])?;
        tx.execute("INSERT INTO events VALUES ('task-1', 1, 'started')", [])?;
        tx.execute("INSERT INTO outbox VALUES ('task-1', 1, '{}')", [])?;
        tx.commit()?;
    }
    let failed = db.transaction()?;
    failed.execute("UPDATE tasks SET state='failed' WHERE id='task-1'", [])?;
    failed.execute("INSERT INTO events VALUES ('task-1', 2, 'failed')", [])?;
    let conflict_rejected = failed
        .execute("INSERT INTO outbox VALUES ('task-1', 1, 'duplicate')", [])
        .is_err();
    drop(failed);
    let state: String = db.query_row("SELECT state FROM tasks WHERE id='task-1'", [], |row| {
        row.get(0)
    })?;
    let event_count: i64 = db.query_row("SELECT count(*) FROM events", [], |row| row.get(0))?;
    let outbox_count: i64 = db.query_row("SELECT count(*) FROM outbox", [], |row| row.get(0))?;
    drop(db);

    let header = fs::read(&db_path)?;
    let encrypted_header = !header.starts_with(b"SQLite format 3\0");
    let wrong_key_rejected = Connection::open(&db_path)
        .and_then(|wrong| {
            wrong.pragma_update(None, "key", "wrong-key")?;
            wrong.query_row("SELECT count(*) FROM sqlite_master", [], |row| {
                row.get::<_, i64>(0)
            })
        })
        .is_err();

    let key = [7u8; 32];
    let nonce = [9u8; 12];
    let cipher = Aes256Gcm::new_from_slice(&key)?;
    let plaintext = b"Yonder attachment fixture";
    let ciphertext = cipher.encrypt((&nonce).into(), plaintext.as_slice())?;
    let decrypted = cipher.decrypt((&nonce).into(), ciphertext.as_slice())?;
    let mut tampered = ciphertext.clone();
    tampered[0] ^= 1;
    let tamper_rejected = cipher
        .decrypt((&nonce).into(), tampered.as_slice())
        .is_err();

    if !(encrypted_header
        && wrong_key_rejected
        && chinese_hits == 1
        && english_hits == 1
        && conflict_rejected
        && state == "running"
        && event_count == 1
        && outbox_count == 1
        && decrypted == plaintext
        && tamper_rejected)
    {
        return Err("验证断言失败".into());
    }
    println!(
        "{{\"cipher_version\":\"{}\",\"encrypted_header\":true,\"wrong_key_rejected\":true,\"fts_chinese_hits\":1,\"fts_english_hits\":1,\"transaction_rollback\":true,\"attachment_roundtrip\":true,\"tamper_rejected\":true,\"elapsed_ms\":{}}}",
        cipher_version,
        started.elapsed().as_millis()
    );
    Ok(())
}
