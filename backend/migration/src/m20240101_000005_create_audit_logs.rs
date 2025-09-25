use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_string()),
                    )
                    .col(ColumnDef::new(AuditLogs::UserId).uuid())
                    .col(ColumnDef::new(AuditLogs::EntityType).string().not_null())
                    .col(ColumnDef::new(AuditLogs::EntityId).uuid())
                    .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                    .col(ColumnDef::new(AuditLogs::OldValues).json())
                    .col(ColumnDef::new(AuditLogs::NewValues).json())
                    .col(ColumnDef::new(AuditLogs::IpAddress).string())
                    .col(ColumnDef::new(AuditLogs::UserAgent).text())
                    .col(ColumnDef::new(AuditLogs::SessionId).string())
                    .col(ColumnDef::new(AuditLogs::Metadata).json())
                    .col(ColumnDef::new(AuditLogs::CreatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_audit_logs_user_id")
                            .from(AuditLogs::Table, AuditLogs::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for efficient querying
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_user_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_entity")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::EntityType)
                    .col(AuditLogs::EntityId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_action")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::Action)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_created_at")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::CreatedAt)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Id,
    UserId,
    EntityType,
    EntityId,
    Action,
    OldValues,
    NewValues,
    IpAddress,
    UserAgent,
    SessionId,
    Metadata,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}