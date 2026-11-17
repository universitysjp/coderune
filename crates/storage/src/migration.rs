//! Schema migrations through SeaORM's schema builder.
//!
//! Every table is `if_not_exists`, so a database created by the earlier
//! hand-written migrations opens and keeps its data.

use sea_orm_migration::prelude::*;

mod profile_context;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(ProfileTables),
            Box::new(ProjectTables),
            Box::new(DocumentTables),
            Box::new(CaptureTables),
            Box::new(profile_context::ProfileContextTables),
        ]
    }
}

struct ProfileTables;
impl MigrationName for ProfileTables {
    fn name(&self) -> &str {
        "m001_profile_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for ProfileTables {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("developer_profiles"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).text().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("developer_profile_versions"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("profile_id")).text().not_null())
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("developer_profile_versions_profile_version")
                    .table(Alias::new("developer_profile_versions"))
                    .col(Alias::new("profile_id"))
                    .col(Alias::new("version"))
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("onboarding_draft"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("onboarding_draft"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("developer_profile_versions"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("developer_profiles"))
                    .to_owned(),
            )
            .await
    }
}

struct ProjectTables;
impl MigrationName for ProjectTables {
    fn name(&self) -> &str {
        "m002_project_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for ProjectTables {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("projects"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).text().not_null())
                    .col(ColumnDef::new(Alias::new("updated_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("project_versions"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .integer()
                            .auto_increment()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).text().not_null())
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("project_versions_project_version")
                    .table(Alias::new("project_versions"))
                    .col(Alias::new("project_id"))
                    .col(Alias::new("version"))
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("project_versions"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("projects")).to_owned())
            .await
    }
}

struct DocumentTables;
impl MigrationName for DocumentTables {
    fn name(&self) -> &str {
        "m003_document_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for DocumentTables {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("documents"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).text())
                    .col(ColumnDef::new(Alias::new("kind")).text().not_null())
                    .col(ColumnDef::new(Alias::new("name")).text().not_null())
                    .col(ColumnDef::new(Alias::new("path")).text().not_null())
                    .col(ColumnDef::new(Alias::new("hash")).text().not_null())
                    .col(ColumnDef::new(Alias::new("size")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("text")).text().not_null())
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("imported_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("documents_project_id")
                    .table(Alias::new("documents"))
                    .col(Alias::new("project_id"))
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("documents_hash")
                    .table(Alias::new("documents"))
                    .col(Alias::new("hash"))
                    .if_not_exists()
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Alias::new("documents")).to_owned())
            .await
    }
}

struct CaptureTables;
impl MigrationName for CaptureTables {
    fn name(&self) -> &str {
        "m004_capture_tables"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for CaptureTables {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("contents"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("hash"))
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("text")).text().not_null())
                    .col(ColumnDef::new(Alias::new("size")).integer().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("captures"))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Alias::new("id"))
                            .text()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Alias::new("project_id")).text().not_null())
                    .col(ColumnDef::new(Alias::new("version")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("fingerprint")).text().not_null())
                    .col(ColumnDef::new(Alias::new("data")).text().not_null())
                    .col(ColumnDef::new(Alias::new("created_at")).text().not_null())
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("captures_project_id")
                    .table(Alias::new("captures"))
                    .col(Alias::new("project_id"))
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("captures_project_version")
                    .table(Alias::new("captures"))
                    .col(Alias::new("project_id"))
                    .col(Alias::new("version"))
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("capture_files"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("capture_id")).text().not_null())
                    .col(ColumnDef::new(Alias::new("path")).text().not_null())
                    .col(ColumnDef::new(Alias::new("content_hash")).text().not_null())
                    .col(ColumnDef::new(Alias::new("size")).integer().not_null())
                    .col(ColumnDef::new(Alias::new("category")).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(Alias::new("capture_id"))
                            .col(Alias::new("path")),
                    )
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Alias::new("capture_skipped"))
                    .if_not_exists()
                    .col(ColumnDef::new(Alias::new("capture_id")).text().not_null())
                    .col(ColumnDef::new(Alias::new("path")).text().not_null())
                    .col(ColumnDef::new(Alias::new("reason")).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(Alias::new("capture_id"))
                            .col(Alias::new("path")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(Alias::new("capture_skipped"))
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("capture_files")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("captures")).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Alias::new("contents")).to_owned())
            .await
    }
}
