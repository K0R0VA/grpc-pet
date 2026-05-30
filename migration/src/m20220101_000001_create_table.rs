use sea_orm_migration::sea_orm::Schema;
use sea_orm_migration::{prelude::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let connection = manager.get_connection();
        let schema = Schema::new(builder);

        let stmt = builder.build(&schema.create_table_from_entity(entities::user::Entity));
        connection.execute(stmt).await?;

        let stmt = builder.build(&schema.create_table_from_entity(entities::route::Entity));
        connection.execute(stmt).await?;

        let stmt = builder.build(&schema.create_table_from_entity(entities::alert::Entity));
        connection.execute(stmt).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let builder = manager.get_database_backend();
        let connection = manager.get_connection();
        connection.execute(builder.build(TableDropStatement::new().table(entities::alert::Entity))).await?;
        connection.execute(builder.build(TableDropStatement::new().table(entities::route::Entity))).await?;
        connection.execute(builder.build(TableDropStatement::new().table(entities::user::Entity))).await?;
        Ok(())
    }
}
