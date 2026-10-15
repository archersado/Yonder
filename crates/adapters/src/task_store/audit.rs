use super::*;

impl yonder_application::TaskAuditStore for SqliteTaskStore {
    fn supports_audit(&self) -> bool {
        true
    }

    fn get_audit(&mut self, task_id: &str) -> Result<yonder_application::TaskAudit, Error> {
        if !yonder_application::valid_id(task_id) {
            return Err(Error::InvalidInput);
        }
        let manifest: Option<(i64, i64, i64)> = self
            .0
            .query_row(
                "SELECT version, created_sequence, item_count FROM task_artifact_manifests WHERE task_id=?1 ORDER BY version DESC LIMIT 1",
                [task_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(storage)?;
        let confirmation: Option<(String, i64, i64, Option<String>, String)> = self
            .0
            .query_row(
                "SELECT confirmation_id, result_sequence, manifest_version, comment, confirmed_by FROM task_user_confirmations WHERE task_id=?1",
                [task_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
            )
            .optional()
            .map_err(storage)?;
        let manifest = match manifest {
            Some((version, _created_sequence, item_count)) => {
                Some(yonder_application::TaskArtifactManifest {
                    task_id: task_id.to_owned(),
                    version: u64::try_from(version).map_err(|_| Error::StorageUnavailable)?,
                    item_count: u16::try_from(item_count).map_err(|_| Error::StorageUnavailable)?,
                })
            }
            None => None,
        };
        let confirmation = match confirmation {
            Some((confirmation_id, result_sequence, manifest_version, comment, confirmed_by)) => {
                Some(yonder_application::TaskUserConfirmation {
                    task_id: task_id.to_owned(),
                    confirmation_id,
                    result_sequence: u64::try_from(result_sequence)
                        .map_err(|_| Error::StorageUnavailable)?,
                    manifest_version: u64::try_from(manifest_version)
                        .map_err(|_| Error::StorageUnavailable)?,
                    comment,
                    confirmed_by,
                })
            }
            None => None,
        };
        Ok(yonder_application::TaskAudit {
            manifest,
            confirmation,
        })
    }

    fn publish_artifact_manifest(
        &mut self,
        task_id: &str,
        expected_sequence: u64,
        items: &[TaskArtifactManifestItem],
    ) -> Result<(Task, TaskArtifactManifest), Error> {
        if !yonder_application::valid_id(task_id)
            || expected_sequence == 0
            || expected_sequence >= i64::MAX as u64
            || items.len() > MAX_ARTIFACT_MANIFEST_ITEMS
            || items.iter().enumerate().any(|(index, item)| {
                item.ordinal != u16::try_from(index + 1).unwrap_or(0)
                    || !yonder_application::valid_id(&item.reference_id)
            })
        {
            return Err(Error::InvalidInput);
        }
        let mut unique = std::collections::HashSet::with_capacity(items.len());
        if items
            .iter()
            .any(|item| !unique.insert(item.reference_id.as_str()))
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, task_name, source): (
            String,
            i64,
            String,
            Option<String>,
            String,
        ) = tx
            .query_row(
                "SELECT state,sequence,owner_agent_id,name,source FROM tasks WHERE id=?1",
                [task_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(storage)?
            .ok_or(Error::NotFound)?;
        if state == "created" {
            return Err(Error::StopRequired);
        }
        if expected_sequence != u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? {
            return Err(Error::Conflict);
        }
        Self::ensure_audit_capacity(&tx)?;
        let version: i64 = tx
            .query_row(
                "SELECT COALESCE(MAX(version),0)+1 FROM task_artifact_manifests WHERE task_id=?1",
                [task_id],
                |row| row.get(0),
            )
            .map_err(storage)?;
        if version <= 0 {
            return Err(Error::StorageUnavailable);
        }
        let next_sequence = sequence.checked_add(1).ok_or(Error::StorageUnavailable)?;
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND sequence=?3",
                params![next_sequence, task_id, sequence],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute(
            "INSERT INTO events(task_id,sequence,previous,state) VALUES (?1,?2,?3,?3)",
            params![task_id, next_sequence, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id,sequence) VALUES (?1,?2)",
            params![task_id, next_sequence],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO task_artifact_manifests(task_id,version,created_sequence,item_count) VALUES (?1,?2,?3,?4)",
            params![task_id, version, next_sequence, i64::try_from(items.len()).map_err(|_| Error::InvalidInput)?],
        )
        .map_err(storage)?;
        for item in items {
            tx.execute(
                "INSERT INTO task_artifact_manifest_items(task_id,version,ordinal,reference_id,availability) VALUES (?1,?2,?3,?4,?5)",
                params![task_id, version, i64::from(item.ordinal), item.reference_id, artifact_availability_name(item.availability)],
            )
            .map_err(storage)?;
        }
        tx.commit().map_err(storage)?;
        let task = Task {
            id: task_id.to_owned(),
            owner_agent_id,
            name: task_name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(next_sequence).map_err(|_| Error::StorageUnavailable)?,
        };
        let manifest = TaskArtifactManifest {
            task_id: task_id.to_owned(),
            version: u64::try_from(version).map_err(|_| Error::StorageUnavailable)?,
            item_count: u16::try_from(items.len()).map_err(|_| Error::StorageUnavailable)?,
        };
        Ok((task, manifest))
    }

    fn artifact_manifest_items(
        &mut self,
        task_id: &str,
        version: u64,
        after_ordinal: u16,
        limit: usize,
    ) -> Result<Vec<TaskArtifactManifestItem>, Error> {
        if !yonder_application::valid_id(task_id)
            || version == 0
            || version >= i64::MAX as u64
            || !(1..=101).contains(&limit)
        {
            return Err(Error::InvalidInput);
        }
        let exists: bool = self
            .0
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM task_artifact_manifests WHERE task_id=?1 AND version=?2)",
                params![task_id, version as i64],
                |row| row.get(0),
            )
            .map_err(storage)?;
        if !exists {
            return Err(Error::NotFound);
        }
        let mut statement = self
            .0
            .prepare(
                "SELECT ordinal,reference_id,availability FROM task_artifact_manifest_items WHERE task_id=?1 AND version=?2 AND ordinal>?3 ORDER BY ordinal LIMIT ?4",
            )
            .map_err(storage)?;
        statement
            .query_map(
                params![
                    task_id,
                    version as i64,
                    i64::from(after_ordinal),
                    limit as i64
                ],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .map_err(storage)?
            .map(|row| {
                let (ordinal, reference_id, availability) = row.map_err(storage)?;
                Ok(TaskArtifactManifestItem {
                    ordinal: u16::try_from(ordinal).map_err(|_| Error::StorageUnavailable)?,
                    reference_id,
                    availability: artifact_availability(&availability)?,
                })
            })
            .collect()
    }

    fn confirm_result(
        &mut self,
        task_id: &str,
        expected_sequence: u64,
        confirmation_id: &str,
        comment: Option<&str>,
    ) -> Result<Task, Error> {
        if !yonder_application::valid_id(task_id)
            || !yonder_application::valid_id(confirmation_id)
            || expected_sequence == 0
            || expected_sequence >= i64::MAX as u64
            || comment.is_some_and(|value| {
                value.trim().is_empty()
                    || value.as_bytes().len() > 2048
                    || value.chars().any(char::is_control)
            })
        {
            return Err(Error::InvalidInput);
        }
        let tx = self
            .0
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(storage)?;
        let (state, sequence, owner_agent_id, name, source): (
            String,
            i64,
            String,
            Option<String>,
            String,
        ) = tx
            .query_row(
                "SELECT state, sequence, owner_agent_id, name, source FROM tasks WHERE id=?1",
                [task_id],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                    ))
                },
            )
            .optional()
            .map_err(storage)?
            .ok_or(Error::NotFound)?;
        if state != "completed" && state != "failed" {
            return Err(Error::InvalidInput);
        }
        let existing: Option<(String, String, i64, Option<String>)> = tx
            .query_row(
                "SELECT confirmation_id, task_id, result_sequence, comment FROM task_user_confirmations WHERE confirmation_id=?1 OR task_id=?2",
                params![confirmation_id, task_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .optional()
            .map_err(storage)?;
        if let Some((
            existing_confirmation_id,
            existing_task_id,
            result_sequence,
            existing_comment,
        )) = existing
        {
            if existing_confirmation_id == confirmation_id
                && existing_task_id == task_id
                && expected_sequence
                    == u64::try_from(result_sequence).map_err(|_| Error::StorageUnavailable)?
                && existing_comment.as_deref() == comment
            {
                return Ok(Task {
                    id: task_id.to_owned(),
                    owner_agent_id,
                    name,
                    source: task_source(&source)?,
                    status: status(&state)?,
                    sequence: u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)?,
                });
            }
            return Err(Error::Conflict);
        }
        if expected_sequence != u64::try_from(sequence).map_err(|_| Error::StorageUnavailable)? {
            return Err(Error::Conflict);
        }
        Self::ensure_audit_capacity(&tx)?;
        let manifest_version: i64 = tx
            .query_row(
                "SELECT version FROM task_artifact_manifests WHERE task_id=?1 ORDER BY version DESC LIMIT 1",
                [task_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?
            .unwrap_or(1);
        let create_empty_manifest = !tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM task_artifact_manifests WHERE task_id=?1)",
                [task_id],
                |row| row.get::<_, bool>(0),
            )
            .map_err(storage)?;
        let next_sequence = sequence.checked_add(1).ok_or(Error::StorageUnavailable)?;
        if tx
            .execute(
                "UPDATE tasks SET sequence=?1 WHERE id=?2 AND sequence=?3",
                params![next_sequence, task_id, sequence],
            )
            .map_err(storage)?
            != 1
        {
            return Err(Error::Conflict);
        }
        tx.execute(
            "INSERT INTO events(task_id, sequence, previous, state) VALUES (?1, ?2, ?3, ?4)",
            params![task_id, next_sequence, state, state],
        )
        .map_err(storage)?;
        tx.execute(
            "INSERT INTO outbox(task_id, sequence) VALUES (?1, ?2)",
            params![task_id, next_sequence],
        )
        .map_err(storage)?;
        if create_empty_manifest {
            tx.execute(
                "INSERT INTO task_artifact_manifests(task_id, version, created_sequence, item_count) VALUES (?1, 1, ?2, 0)",
                params![task_id, next_sequence],
            )
            .map_err(storage)?;
        }
        tx.execute(
            "INSERT INTO task_user_confirmations(confirmation_id, task_id, result_sequence, confirmation_sequence, manifest_version, comment, confirmed_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'local-user')",
            params![confirmation_id, task_id, sequence, next_sequence, manifest_version, comment],
        )
        .map_err(storage)?;
        tx.commit().map_err(storage)?;
        Ok(Task {
            id: task_id.to_owned(),
            owner_agent_id,
            name,
            source: task_source(&source)?,
            status: status(&state)?,
            sequence: u64::try_from(next_sequence).map_err(|_| Error::StorageUnavailable)?,
        })
    }
}
