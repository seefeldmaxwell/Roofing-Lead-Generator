use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use tracing::info;

use crate::services::fema_client::FemaApiClient;

/// Create a SQLite connection pool.
///
/// The database URL is read from the `DATABASE_URL` environment variable.  If
/// the variable is not set the pool defaults to a local file-based database
/// (`sqlite:roofing_lead_gen.db?mode=rwc`).
pub async fn create_pool() -> SqlitePool {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:roofing_lead_gen.db?mode=rwc".to_string());

    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create SQLite connection pool")
}

/// Run all pending migrations from the `migrations/` directory.
///
/// Uses the compile-time `sqlx::migrate!()` macro so the migration SQL is
/// embedded in the binary.
pub async fn run_migrations(pool: &SqlitePool) {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .expect("Failed to run database migrations");

    info!("Database migrations applied successfully");
}

/// Seed the `fema_disasters` table with Florida disaster declarations fetched
/// from the FEMA OpenAPI if the table is currently empty.
///
/// This is safe to call on every startup -- it is a no-op when rows already
/// exist.
pub async fn seed_fema_data(pool: &SqlitePool, fema_client: &FemaApiClient) {
    // Check whether the table already contains data.
    let count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM fema_disasters")
            .fetch_one(pool)
            .await
            .unwrap_or((0,));

    if count.0 > 0 {
        info!(
            "FEMA disasters table already has {} rows -- skipping seed",
            count.0
        );
        return;
    }

    info!("Seeding FEMA disaster data (fetching up to 200 records)...");

    let disasters = fema_client.get_florida_disasters(200).await;

    if disasters.is_empty() {
        tracing::warn!("FEMA API returned no disaster records; skipping seed");
        return;
    }

    let mut inserted = 0u64;

    for d in &disasters {
        let result = sqlx::query(
            r#"
            INSERT INTO fema_disasters (
                disaster_number,
                title,
                disaster_type,
                declaration_date,
                incident_begin_date,
                incident_end_date,
                closeout_date,
                state,
                declared_county,
                incident_type,
                programs_available,
                individual_assistance,
                public_assistance,
                hazard_mitigation,
                fema_url
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&d.disaster_number)
        .bind(&d.title)
        .bind(&d.disaster_type)
        .bind(&d.declaration_date)
        .bind(&d.incident_begin_date)
        .bind(&d.incident_end_date)
        .bind(&d.closeout_date)
        .bind(&d.state)
        .bind(&d.declared_county)
        .bind(&d.incident_type)
        .bind(&d.programs_available)
        .bind(d.individual_assistance)
        .bind(d.public_assistance)
        .bind(d.hazard_mitigation)
        .bind(&d.fema_url)
        .execute(pool)
        .await;

        match result {
            Ok(_) => inserted += 1,
            Err(e) => {
                tracing::warn!(
                    "Failed to insert FEMA disaster {}: {}",
                    d.disaster_number,
                    e
                );
            }
        }
    }

    info!(
        "Seeded {} FEMA disaster records ({} fetched)",
        inserted,
        disasters.len()
    );
}
