use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TokenBalances::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(TokenBalances::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .extra("DEFAULT gen_random_uuid()".to_string()),
                    )
                    .col(ColumnDef::new(TokenBalances::UserId).uuid().not_null())
                    .col(ColumnDef::new(TokenBalances::TokenType).string().not_null())
                    .col(ColumnDef::new(TokenBalances::Balance).decimal().not_null().default(0))
                    .col(ColumnDef::new(TokenBalances::LockedBalance).decimal().not_null().default(0))
                    .col(ColumnDef::new(TokenBalances::LastUpdated).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(TokenBalances::CreatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .col(ColumnDef::new(TokenBalances::UpdatedAt).timestamp().not_null().extra("DEFAULT CURRENT_TIMESTAMP".to_string()))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_token_balances_user_id")
                            .from(TokenBalances::Table, TokenBalances::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Create unique constraint on user_id + token_type
        manager
            .create_index(
                Index::create()
                    .name("idx_token_balances_user_token")
                    .table(TokenBalances::Table)
                    .col(TokenBalances::UserId)
                    .col(TokenBalances::TokenType)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TokenBalances::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum TokenBalances {
    Table,
    Id,
    UserId,
    TokenType,
    Balance,
    LockedBalance,
    LastUpdated,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}