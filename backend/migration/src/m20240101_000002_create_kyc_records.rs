use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(KycRecords::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(KycRecords::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_string()),
                    )
                    .col(ColumnDef::new(KycRecords::UserId).uuid().not_null())
                    .col(ColumnDef::new(KycRecords::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(KycRecords::TierLevel).integer().not_null().default(0))
                    .col(ColumnDef::new(KycRecords::DocumentType).string())
                    .col(ColumnDef::new(KycRecords::DocumentNumber).string())
                    .col(ColumnDef::new(KycRecords::VerificationData).json())
                    .col(ColumnDef::new(KycRecords::ApprovedBy).uuid())
                    .col(ColumnDef::new(KycRecords::ApprovedAt).timestamp())
                    .col(ColumnDef::new(KycRecords::ExpiresAt).timestamp())
                    .col(ColumnDef::new(KycRecords::CreatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(KycRecords::UpdatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_kyc_records_user_id")
                            .from(KycRecords::Table, KycRecords::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes
        manager
            .create_index(
                Index::create()
                    .name("idx_kyc_records_user_id")
                    .table(KycRecords::Table)
                    .col(KycRecords::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_kyc_records_status")
                    .table(KycRecords::Table)
                    .col(KycRecords::Status)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(KycRecords::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum KycRecords {
    Table,
    Id,
    UserId,
    Status,
    TierLevel,
    DocumentType,
    DocumentNumber,
    VerificationData,
    ApprovedBy,
    ApprovedAt,
    ExpiresAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}