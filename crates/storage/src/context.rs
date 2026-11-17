//! Current-profile projection reads and writes.
//!
//! The profile JSON remains the source of truth. These rows are a derived,
//! exact-field view that a purpose filter narrows before anything is read.

use coderune_domain::{
    now_iso, ContextSensitivity, DeveloperProfile, PersonalContextEntry, PersonalContextEntryKind,
    PROFILE_CONTEXT_PROJECTION_VERSION,
};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};

use crate::{
    developer_profile, profile_context_entry, profile_context_index_state, Result, Storage,
    StorageError,
};

/// Exact selectors for a current-profile context read. Values are explicit,
/// never ordered, so a new sensitivity cannot silently grant access.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextReadFilter {
    pub profile_id: String,
    pub sensitivities: Vec<ContextSensitivity>,
    pub kinds: Vec<PersonalContextEntryKind>,
    pub field_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileContextSnapshot {
    pub profile: DeveloperProfile,
    pub entries: Vec<PersonalContextEntry>,
}

impl Storage {
    /// Rebuild the current profile projection when it is missing, stale, or
    /// produced by another projection version. Opening a database calls this.
    pub(crate) fn refresh_profile_context_if_stale(&self) -> Result<()> {
        self.run(async {
            let tx = self.conn.begin().await?;
            let profile_model = developer_profile::Entity::find_by_id("developer")
                .one(&tx)
                .await?;
            let Some(profile_model) = profile_model else {
                tx.commit().await?;
                return Ok(());
            };
            let profile: DeveloperProfile = serde_json::from_str(&profile_model.data)?;
            let state = profile_context_index_state::Entity::find_by_id("developer")
                .one(&tx)
                .await?;
            let fresh = state.as_ref().is_some_and(|state| {
                state.profile_version == profile.version as i32
                    && state.projection_version == PROFILE_CONTEXT_PROJECTION_VERSION as i32
            });
            if !fresh {
                let projected_at = now_iso();
                write_profile_context(&tx, &profile, &projected_at).await?;
            }
            tx.commit().await?;
            Ok(())
        })
    }

    /// Read the pinned current profile and only the permitted exact entries in
    /// one database snapshot. Stale index state is an error, never a fallback.
    pub fn read_profile_context(
        &self,
        filter: &ContextReadFilter,
    ) -> Result<ProfileContextSnapshot> {
        self.run(async {
            let tx = self.conn.begin().await?;
            let profile_model = developer_profile::Entity::find_by_id(&filter.profile_id)
                .one(&tx)
                .await?
                .ok_or_else(|| StorageError::NotFound("developer profile".to_string()))?;
            let profile: DeveloperProfile = serde_json::from_str(&profile_model.data)?;
            let state = profile_context_index_state::Entity::find_by_id(&filter.profile_id)
                .one(&tx)
                .await?
                .ok_or_else(|| {
                    StorageError::ContextIndex("missing current profile projection".to_string())
                })?;
            if state.profile_version != profile.version as i32
                || state.projection_version != PROFILE_CONTEXT_PROJECTION_VERSION as i32
            {
                return Err(StorageError::ContextIndex(format!(
                    "indexed version {} does not match profile version {}",
                    state.profile_version, profile.version
                )));
            }
            let entries = if filter.sensitivities.is_empty()
                || filter.kinds.is_empty()
                || filter.field_paths.is_empty()
            {
                Vec::new()
            } else {
                profile_context_entry::Entity::find()
                    .filter(profile_context_entry::Column::ProfileId.eq(&filter.profile_id))
                    .filter(
                        profile_context_entry::Column::ProfileVersion.eq(profile.version as i32),
                    )
                    .filter(
                        profile_context_entry::Column::Sensitivity.is_in(
                            filter
                                .sensitivities
                                .iter()
                                .map(|value| value.as_str().to_string()),
                        ),
                    )
                    .filter(
                        profile_context_entry::Column::EntryKind
                            .is_in(filter.kinds.iter().map(|value| value.as_str().to_string())),
                    )
                    .filter(
                        profile_context_entry::Column::FieldPath.is_in(filter.field_paths.clone()),
                    )
                    .order_by_asc(profile_context_entry::Column::FieldPath)
                    .order_by_asc(profile_context_entry::Column::SourcePath)
                    .all(&tx)
                    .await?
                    .into_iter()
                    .map(context_entry_from_model)
                    .collect::<Result<Vec<_>>>()?
            };
            tx.commit().await?;
            Ok(ProfileContextSnapshot { profile, entries })
        })
    }
}

/// Replace one profile's current projection and index state in the caller's
/// transaction. The profile JSON and its history are never changed here.
pub(crate) async fn write_profile_context(
    tx: &sea_orm::DatabaseTransaction,
    profile: &DeveloperProfile,
    projected_at: &str,
) -> Result<()> {
    let entries = profile.context_entries()?;
    profile_context_entry::Entity::delete_many()
        .filter(profile_context_entry::Column::ProfileId.eq(&profile.id))
        .exec(tx)
        .await?;
    for entry in &entries {
        let field_source = entry
            .field_source
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        profile_context_entry::Entity::insert(profile_context_entry::ActiveModel {
            id: sea_orm::ActiveValue::NotSet,
            profile_id: Set(profile.id.clone()),
            profile_version: Set(profile.version as i32),
            entry_kind: Set(entry.kind.as_str().to_string()),
            field_path: Set(entry.field_path.clone()),
            source_path: Set(entry.source_path.clone()),
            sensitivity: Set(entry.sensitivity.as_str().to_string()),
            search_text: Set(entry.search_text.clone()),
            payload: Set(serde_json::to_string(&entry.payload)?),
            field_source: Set(field_source),
            content_hash: Set(entry.content_hash.clone()),
            projection_version: Set(PROFILE_CONTEXT_PROJECTION_VERSION as i32),
            created_at: Set(projected_at.to_string()),
        })
        .exec(tx)
        .await?;
    }
    profile_context_index_state::Entity::insert(profile_context_index_state::ActiveModel {
        profile_id: Set(profile.id.clone()),
        profile_version: Set(profile.version as i32),
        projection_version: Set(PROFILE_CONTEXT_PROJECTION_VERSION as i32),
        updated_at: Set(projected_at.to_string()),
    })
    .on_conflict(
        OnConflict::column(profile_context_index_state::Column::ProfileId)
            .update_columns([
                profile_context_index_state::Column::ProfileVersion,
                profile_context_index_state::Column::ProjectionVersion,
                profile_context_index_state::Column::UpdatedAt,
            ])
            .to_owned(),
    )
    .exec(tx)
    .await?;
    Ok(())
}

fn context_entry_from_model(model: profile_context_entry::Model) -> Result<PersonalContextEntry> {
    let kind = model
        .entry_kind
        .parse::<PersonalContextEntryKind>()
        .map_err(|_| {
            StorageError::ContextIndex(format!("unknown entry kind {}", model.entry_kind))
        })?;
    let sensitivity = model
        .sensitivity
        .parse::<ContextSensitivity>()
        .map_err(|_| {
            StorageError::ContextIndex(format!("unknown sensitivity {}", model.sensitivity))
        })?;
    Ok(PersonalContextEntry {
        kind,
        field_path: model.field_path,
        source_path: model.source_path,
        sensitivity,
        search_text: model.search_text,
        payload: serde_json::from_str(&model.payload)?,
        field_source: model
            .field_source
            .as_deref()
            .map(serde_json::from_str)
            .transpose()?,
        content_hash: model.content_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::profile_draft;
    use coderune_domain::{
        ContextSharingPolicy, ExperienceStage, PersonalContextPurpose, ProfileDraft,
        SelfReportedSkill, SkillRating,
    };
    use sea_orm::{ConnectionTrait, Database};

    fn context_filter(purpose: PersonalContextPurpose, paths: &[&str]) -> ContextReadFilter {
        let policy = ContextSharingPolicy::new(purpose);
        ContextReadFilter {
            profile_id: "developer".to_string(),
            sensitivities: policy.allowed_sensitivities(),
            kinds: policy.allowed_kinds(),
            field_paths: paths.iter().map(|path| (*path).to_string()).collect(),
        }
    }

    #[test]
    fn opens_a_database_created_by_the_legacy_schema() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("context-bridge.sqlite");
        let url = format!("sqlite://{}?mode=rwc", path.display());
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let profile = ProfileDraft {
            display_name: "Legacy".to_string(),
            experience_stage: Some(ExperienceStage::Experienced),
            skills_statement: "Rust".to_string(),
            learning_goals: vec!["Keep data".to_string()],
            ..Default::default()
        }
        .into_profile(None)
        .unwrap();
        let data = serde_json::to_string(&profile).unwrap();
        runtime.block_on(async {
            let conn = Database::connect(&url).await.unwrap();
            conn.execute_unprepared(
                "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
                 CREATE TABLE developer_profiles (id TEXT PRIMARY KEY, version INTEGER NOT NULL, data TEXT NOT NULL, created_at TEXT NOT NULL, updated_at TEXT NOT NULL);",
            )
            .await
            .unwrap();
            conn.execute_unprepared(&format!(
                "INSERT INTO developer_profiles VALUES ('developer', 1, '{}', '{}', '{}');",
                data.replace('\'', "''"),
                profile.created_at,
                profile.updated_at
            ))
            .await
            .unwrap();
        });
        drop(runtime);
        let storage = Storage::open(&path).unwrap();
        let loaded = storage.get_profile().unwrap().unwrap();
        assert_eq!(loaded.display_name, "Legacy");
        assert_eq!(loaded.version, 1);
        let snapshot = storage
            .read_profile_context(&context_filter(
                PersonalContextPurpose::Coding,
                &["experienceStage", "skillsStatement"],
            ))
            .unwrap();
        assert_eq!(snapshot.entries.len(), 2);
    }

    #[test]
    fn confirmation_writes_the_current_context_projection() {
        let mut storage = Storage::in_memory().unwrap();
        storage.confirm_profile(profile_draft()).unwrap();
        let snapshot = storage
            .read_profile_context(&context_filter(
                PersonalContextPurpose::Coding,
                &["experienceStage", "skillsStatement"],
            ))
            .unwrap();
        assert_eq!(snapshot.profile.version, 1);
        assert_eq!(snapshot.entries.len(), 2);
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.source_path == "/skillsStatement"
                && entry.field_path == "skillsStatement"
                && entry.search_text == "Rust"));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.kind == PersonalContextEntryKind::ExperienceStage));
    }

    #[test]
    fn profile_edit_replaces_the_projection_but_keeps_history() {
        let mut storage = Storage::in_memory().unwrap();
        storage.confirm_profile(profile_draft()).unwrap();
        let second = storage
            .confirm_profile(ProfileDraft {
                skills_statement: "Rust and SQLite".to_string(),
                self_reported_skills: vec![SelfReportedSkill {
                    topic: "Rust".to_string(),
                    rating: Some(SkillRating::Intermediate),
                }],
                ..profile_draft()
            })
            .unwrap();
        assert_eq!(second.version, 2);
        let snapshot = storage
            .read_profile_context(&context_filter(
                PersonalContextPurpose::Coding,
                &["experienceStage", "skillsStatement", "skill:rust"],
            ))
            .unwrap();
        assert_eq!(snapshot.profile.version, 2);
        assert_eq!(snapshot.entries.len(), 3);
        assert!(snapshot
            .entries
            .iter()
            .all(|entry| entry.source_path != "/selfReportedSkills/0"
                || entry.search_text == "Rust (Intermediate)"));
        assert!(snapshot
            .entries
            .iter()
            .any(|entry| entry.source_path == "/selfReportedSkills/0"));
        assert_eq!(storage.get_profile_versions().unwrap().len(), 2);
    }

    #[test]
    fn delete_all_removes_the_current_context_projection() {
        let mut storage = Storage::in_memory().unwrap();
        storage.confirm_profile(profile_draft()).unwrap();
        storage.delete_all_data().unwrap();
        assert!(storage
            .read_profile_context(&context_filter(
                PersonalContextPurpose::Coding,
                &["experienceStage", "skillsStatement"],
            ))
            .is_err());
        storage.confirm_profile(profile_draft()).unwrap();
        let snapshot = storage
            .read_profile_context(&context_filter(
                PersonalContextPurpose::Coding,
                &["experienceStage", "skillsStatement"],
            ))
            .unwrap();
        assert_eq!(snapshot.entries.len(), 2);
    }

    #[test]
    fn context_reads_return_nothing_for_an_empty_policy() {
        let mut storage = Storage::in_memory().unwrap();
        storage.confirm_profile(profile_draft()).unwrap();
        let snapshot = storage
            .read_profile_context(&context_filter(
                PersonalContextPurpose::Verification,
                &["experienceStage", "skillsStatement"],
            ))
            .unwrap();
        assert!(snapshot.entries.is_empty());
        assert_eq!(snapshot.profile.version, 1);
    }
}
