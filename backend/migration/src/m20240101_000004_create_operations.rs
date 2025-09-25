use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Operations::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Operations::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_string()),
                    )
                    .col(ColumnDef::new(Operations::UserId).uuid().not_null())
                    .col(ColumnDef::new(Operations::OperationType).string().not_null())
                    .col(ColumnDef::new(Operations::Status).string().not_null().default("pending"))
                    .col(ColumnDef::new(Operations::Amount).decimal())
                    .col(ColumnDef::new(Operations::TokenType).string())
                    .col(ColumnDef::new(Operations::BtcTxHash).string())
                    .col(ColumnDef::new(Operations::SorobanTxHash).string())
                    .col(ColumnDef::new(Operations::Metadata).json())
                    .col(ColumnDef::new(Operations::ErrorMessage).text())
                    .col(ColumnDef::new(Operations::CompletedAt).timestamp())
                    .col(ColumnDef::new(Operations::CreatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(Operations::UpdatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_operations_user_id")
                            .from(Operations::Table, Operations::UserId)
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
                    .name("idx_operations_user_id")
                    .table(Operations::Table)
                    .col(Operations::UserId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_operations_status")
                    .table(Operations::Table)
                    .col(Operations::Status)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_operations_type")
                    .table(Operations::Table)
                    .col(Operations::OperationType)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_operations_btc_tx_hash")
                    .table(Operations::Table)
                    .col(Operations::BtcTxHash)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Operations::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Operations {
    Table,
    Id,
    UserId,
    OperationType,
    Status,
    Amount,
    TokenType,
    BtcTxHash,
    SorobanTxHash,
    Metadata,
    ErrorMessage,
    CompletedAt,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}