use sea_orm::{Database as SeaDatabase, DatabaseConnection, DbErr};
use std::time::Duration;

pub type Database = DatabaseConnection;

pub async fn create_database_connection(database_url: &str) -> Result<Database, DbErr> {
    let mut opt = sea_orm::ConnectOptions::new(database_url.to_owned());
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true)
        .sqlx_logging_level(log::LevelFilter::Info);

    SeaDatabase::connect(opt).await
}

pub async fn run_migrations(db: &Database) -> Result<(), DbErr> {
    // For now, we'll handle migrations manually with the CLI
    // In a production setup, you might want to use sea-orm-migration
    tracing::info!("Database migrations should be run manually using CLI commands");
    Ok(())
}