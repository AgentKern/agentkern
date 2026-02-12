use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL must be set for migrations")?;

    tracing::info!("📦 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("🚀 Running migrations...");

    // Identity Pillar
    tracing::info!("Running Identity migrations...");
    let mut migrator = sqlx::migrate!("../../packages/pillars/identity/migrations");
    // Ignore missing versions because other pillars might have applied migrations
    // that are not in this pillar's migration source.
    migrator.set_ignore_missing(true).run(&pool).await?;

    // Arbiter Pillar
    tracing::info!("Running Arbiter migrations...");
    let mut arbiter_migrator = sqlx::migrate!("../../packages/pillars/arbiter/migrations");
    arbiter_migrator.set_ignore_missing(true).run(&pool).await?;

    // Treasury Pillar (Schema created by code mostly but if we add sql files later)
    // sqlx::migrate!("../../packages/pillars/treasury/migrations").run(&pool).await?;

    tracing::info!("✅ All migrations completed successfully.");
    Ok(())
}
