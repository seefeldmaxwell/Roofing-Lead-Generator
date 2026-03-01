use chrono::NaiveDateTime;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use tracing::warn;

use crate::models::Permit;

/// A permit together with its associated property address (populated from
/// JOINed queries only).
#[derive(Debug, Clone)]
pub struct PermitWithAddress {
    pub permit: Permit,
    pub property_address: String,
}

/// Service for querying permits from the local SQLite database.
pub struct PermitService {
    pool: SqlitePool,
}

impl PermitService {
    /// Create a new `PermitService`.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Return all permits for a given property, ordered by `issued_date` DESC.
    pub async fn get_permits_by_property_id(&self, property_id: i64) -> Vec<Permit> {
        sqlx::query_as::<_, Permit>(
            "SELECT id, permit_number, permit_type, work_category, description, \
             issued_date, completed_date, expiration_date, status, \
             contractor_name, contractor_license, estimated_cost, \
             is_roofing_permit, data_source, property_id \
             FROM permits \
             WHERE property_id = ? \
             ORDER BY issued_date DESC",
        )
        .bind(property_id)
        .fetch_all(&self.pool)
        .await
        .unwrap_or_else(|e| {
            warn!(
                "Failed to fetch permits for property {}: {}",
                property_id, e
            );
            Vec::new()
        })
    }

    /// Return the most recent permits across all properties issued within the
    /// last `days` days.  Each entry includes the property address obtained
    /// via a JOIN.  Results are limited to 50 rows.
    pub async fn get_recent_permits(&self, days: i32) -> Vec<PermitWithAddress> {
        let rows = sqlx::query(
            "SELECT pm.id, pm.permit_number, pm.permit_type, pm.work_category, \
             pm.description, pm.issued_date, pm.completed_date, pm.expiration_date, \
             pm.status, pm.contractor_name, pm.contractor_license, pm.estimated_cost, \
             pm.is_roofing_permit, pm.data_source, pm.property_id, \
             p.address AS property_address \
             FROM permits pm \
             INNER JOIN properties p ON p.id = pm.property_id \
             WHERE pm.issued_date >= datetime('now', '-' || ? || ' days') \
             ORDER BY pm.issued_date DESC \
             LIMIT 50",
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await;

        match rows {
            Ok(rows) => rows.iter().map(Self::map_permit_with_address).collect(),
            Err(e) => {
                warn!("Failed to fetch recent permits (days={}): {}", days, e);
                Vec::new()
            }
        }
    }

    // ── Row mapper ──────────────────────────────────────────────────────────

    /// Map a raw row that includes the joined `property_address` alias into a
    /// `PermitWithAddress`.
    fn map_permit_with_address(row: &SqliteRow) -> PermitWithAddress {
        let permit = Permit {
            id: row.get::<i64, _>("id"),
            permit_number: row.get::<String, _>("permit_number"),
            permit_type: row.get::<String, _>("permit_type"),
            work_category: row.get::<String, _>("work_category"),
            description: row.get::<Option<String>, _>("description"),
            issued_date: row.get::<Option<NaiveDateTime>, _>("issued_date"),
            completed_date: row.get::<Option<NaiveDateTime>, _>("completed_date"),
            expiration_date: row.get::<Option<NaiveDateTime>, _>("expiration_date"),
            status: row.get::<String, _>("status"),
            contractor_name: row.get::<Option<String>, _>("contractor_name"),
            contractor_license: row.get::<Option<String>, _>("contractor_license"),
            estimated_cost: row.get::<Option<f64>, _>("estimated_cost"),
            is_roofing_permit: row.get::<bool, _>("is_roofing_permit"),
            data_source: row.get::<Option<String>, _>("data_source"),
            property_id: row.get::<i64, _>("property_id"),
        };

        let property_address: String = row
            .get::<Option<String>, _>("property_address")
            .unwrap_or_default();

        PermitWithAddress {
            permit,
            property_address,
        }
    }
}
