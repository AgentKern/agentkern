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

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    tracing::info!("📦 Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    tracing::info!("🚀 Running migrations...");

    // Identity Pillar
    tracing::info!("Running Identity migrations...");
    sqlx::migrate!("../../packages/pillars/identity/migrations")
        .run(&pool)
        .await?;
        
    // Treasury Pillar (Schema created by code mostly but if we add sql files later)
    // sqlx::migrate!("../../packages/pillars/treasury/migrations").run(&pool).await?;

    tracing::info!("✅ All migrations completed successfully.");
    Ok(())
}
