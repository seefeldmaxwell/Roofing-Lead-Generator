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

/// Seed sample property, owner, permit, insurance claim, and fire incident
/// data if the properties table is empty.  This makes every page functional
/// out of the box without external API calls.
pub async fn seed_sample_data(pool: &SqlitePool) {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM properties")
        .fetch_one(pool)
        .await
        .unwrap_or((0,));

    if count.0 > 0 {
        info!("Properties table already has {} rows -- skipping sample seed", count.0);
        return;
    }

    info!("Seeding sample property data (20 Florida properties)...");

    // ── Owners ──────────────────────────────────────────────────────────────
    let owners: Vec<(&str, &str, &str, &str, &str, &str, &str, &str, i64)> = vec![
        ("Maria", "Rodriguez", "2801 NW 42nd Ave", "Miami", "FL", "33142", "(305) 555-0101", "maria.rodriguez@email.com", 52),
        ("James", "Thompson", "1450 Brickell Ave Apt 3202", "Miami", "FL", "33131", "(305) 555-0102", "james.thompson@email.com", 67),
        ("Ana", "Garcia", "900 S Federal Hwy", "Fort Lauderdale", "FL", "33316", "(954) 555-0103", "ana.garcia@email.com", 45),
        ("Robert", "Williams", "3300 NE 36th St", "Fort Lauderdale", "FL", "33308", "(954) 555-0104", "robert.williams@email.com", 73),
        ("Sofia", "Martinez", "500 S Rosemary Ave", "West Palm Beach", "FL", "33401", "(561) 555-0105", "sofia.martinez@email.com", 38),
        ("Michael", "Johnson", "1100 S Orlando Ave", "Orlando", "FL", "32801", "(407) 555-0106", "michael.johnson@email.com", 55),
        ("Laura", "Hernandez", "4200 W Cypress St", "Tampa", "FL", "33607", "(813) 555-0107", "laura.hernandez@email.com", 41),
        ("David", "Brown", "800 2nd Ave NE", "St. Petersburg", "FL", "33701", "(727) 555-0108", "david.brown@email.com", 62),
        ("Carmen", "Lopez", "1200 N Federal Hwy", "Boca Raton", "FL", "33432", "(561) 555-0109", "carmen.lopez@email.com", 48),
        ("William", "Davis", "5600 Collins Ave", "Miami Beach", "FL", "33140", "(305) 555-0110", "william.davis@email.com", 59),
        ("Patricia", "Wilson", "2100 NE 163rd St", "Miami", "FL", "33162", "(305) 555-0111", "patricia.wilson@email.com", 65),
        ("Carlos", "Perez", "7800 NW 25th St", "Doral", "FL", "33122", "(305) 555-0112", "carlos.perez@email.com", 43),
        ("Jennifer", "Anderson", "3900 N Ocean Blvd", "Pompano Beach", "FL", "33062", "(954) 555-0113", "jennifer.anderson@email.com", 51),
        ("Luis", "Gonzalez", "1500 Main St", "Sarasota", "FL", "34236", "(941) 555-0114", "luis.gonzalez@email.com", 57),
        ("Sarah", "Taylor", "600 N Atlantic Ave", "Daytona Beach", "FL", "32118", "(386) 555-0115", "sarah.taylor@email.com", 36),
        ("Jose", "Rivera", "2200 SW 1st Ave", "Homestead", "FL", "33030", "(305) 555-0116", "jose.rivera@email.com", 70),
        ("Michelle", "Moore", "4100 Bayshore Blvd", "Tampa", "FL", "33611", "(813) 555-0117", "michelle.moore@email.com", 44),
        ("Thomas", "Clark", "1800 Atlantic Blvd", "Jacksonville", "FL", "32207", "(904) 555-0118", "thomas.clark@email.com", 58),
        ("Nicole", "Lee", "3200 S Congress Ave", "Boynton Beach", "FL", "33426", "(561) 555-0119", "nicole.lee@email.com", 33),
        ("Richard", "Walker", "900 Duval St", "Key West", "FL", "33040", "(305) 555-0120", "richard.walker@email.com", 61),
    ];

    for (first, last, addr, city, state, zip, phone, email, age) in &owners {
        sqlx::query(
            "INSERT INTO owners (first_name, last_name, mailing_address, mailing_city, mailing_state, mailing_zip, phone_number, email, age) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(first).bind(last).bind(addr).bind(city).bind(state).bind(zip).bind(phone).bind(email).bind(age)
        .execute(pool).await.ok();
    }

    // ── Properties ──────────────────────────────────────────────────────────
    // (address, city, zip, county, parcel_id, year_built, sqft, lot_size, prop_type, est_value, beds, baths, roof_type, roof_age, lat, lon, flood_zone, is_flood, is_high_risk_flood, fire_risk, wildfire_score, is_wildfire, risk_score, lead_priority, owner_id)
    let properties: Vec<(&str, &str, &str, &str, &str, i64, f64, f64, &str, f64, i64, i64, &str, i64, f64, f64, &str, bool, bool, &str, i64, bool, i64, &str, i64)> = vec![
        ("2801 NW 42nd Ave", "Miami", "33142", "Miami-Dade", "01-3127-015-0010", 1985, 1850.0, 6500.0, "Single Family", 425000.0, 3, 2, "Shingle", 22, 25.8103, -80.2452, "AE", true, true, "Moderate", 35, false, 72, "High", 1),
        ("1450 Brickell Ave Unit 3202", "Miami", "33131", "Miami-Dade", "01-0108-070-3202", 2010, 1200.0, 0.0, "Condo", 620000.0, 2, 2, "Flat/Built-Up", 14, 25.7589, -80.1918, "AE", true, true, "Low", 10, false, 45, "Medium", 2),
        ("900 S Federal Hwy", "Fort Lauderdale", "33316", "Broward", "50-42-09-AB-0120", 1972, 2200.0, 8200.0, "Single Family", 380000.0, 4, 2, "Tile", 28, 26.0998, -80.1371, "VE", true, true, "Moderate", 40, false, 85, "High", 3),
        ("3300 NE 36th St", "Fort Lauderdale", "33308", "Broward", "50-42-13-BC-0080", 1998, 2600.0, 9500.0, "Single Family", 550000.0, 4, 3, "Shingle", 18, 26.1496, -80.1158, "X", false, false, "Low", 15, false, 38, "Medium", 4),
        ("500 S Rosemary Ave", "West Palm Beach", "33401", "Palm Beach", "74-43-19-09-15-000", 2005, 1600.0, 0.0, "Townhouse", 485000.0, 3, 2, "Tile", 12, 26.7095, -80.0564, "X", false, false, "Low", 8, false, 22, "Low", 5),
        ("1100 S Orlando Ave", "Orlando", "32801", "Orange", "25-22-29-4567-01-020", 1990, 1950.0, 7200.0, "Single Family", 310000.0, 3, 2, "Shingle", 20, 28.5345, -81.3785, "A", true, true, "Moderate", 30, false, 65, "High", 6),
        ("4200 W Cypress St", "Tampa", "33607", "Hillsborough", "A-29-14-ZZ-000000-00190.0", 1978, 2100.0, 7800.0, "Single Family", 340000.0, 3, 2, "Shingle", 25, 27.9524, -82.5091, "AE", true, true, "Moderate", 28, false, 70, "High", 7),
        ("800 2nd Ave NE", "St. Petersburg", "33701", "Pinellas", "19-31-17-12345-001-0010", 2002, 1800.0, 5500.0, "Single Family", 445000.0, 3, 2, "Metal", 8, 27.7753, -82.6304, "X", false, false, "Low", 12, false, 25, "Low", 8),
        ("1200 N Federal Hwy", "Boca Raton", "33432", "Palm Beach", "06-43-47-08-05-000-0340", 1988, 2400.0, 9000.0, "Single Family", 520000.0, 4, 3, "Tile", 19, 26.3586, -80.0831, "AE", true, true, "Low", 18, false, 55, "Medium", 9),
        ("5600 Collins Ave", "Miami Beach", "33140", "Miami-Dade", "02-3227-034-0250", 1965, 1400.0, 0.0, "Condo", 510000.0, 2, 1, "Flat/Built-Up", 32, 25.8366, -80.1209, "VE", true, true, "Low", 8, false, 78, "High", 10),
        ("2100 NE 163rd St", "Miami", "33162", "Miami-Dade", "06-2230-004-0110", 1975, 1700.0, 6000.0, "Single Family", 355000.0, 3, 2, "Shingle", 24, 25.9282, -80.1628, "AE", true, true, "Moderate", 32, false, 68, "High", 11),
        ("7800 NW 25th St", "Doral", "33122", "Miami-Dade", "35-3022-000-0340", 2015, 2800.0, 5000.0, "Single Family", 650000.0, 5, 3, "Tile", 9, 25.8000, -80.3400, "X", false, false, "Low", 5, false, 15, "Low", 12),
        ("3900 N Ocean Blvd", "Pompano Beach", "33062", "Broward", "48-43-03-01-0040", 1980, 1600.0, 5800.0, "Single Family", 395000.0, 3, 2, "Shingle", 21, 26.2486, -80.0788, "VE", true, true, "Moderate", 38, false, 75, "High", 13),
        ("1500 Main St", "Sarasota", "34236", "Sarasota", "2022-0000-008", 1992, 2000.0, 7000.0, "Single Family", 410000.0, 3, 2, "Tile", 16, 27.3342, -82.5380, "AE", true, false, "Moderate", 25, false, 50, "Medium", 14),
        ("600 N Atlantic Ave", "Daytona Beach", "32118", "Volusia", "4226-01-00-0110", 2000, 1500.0, 4500.0, "Single Family", 285000.0, 3, 2, "Shingle", 15, 29.2270, -81.0150, "AE", true, true, "High", 55, true, 62, "Medium", 15),
        ("2200 SW 1st Ave", "Homestead", "33030", "Miami-Dade", "30-6030-001-0180", 1970, 1300.0, 8000.0, "Single Family", 260000.0, 3, 1, "Shingle", 30, 25.4688, -80.4800, "AH", true, true, "High", 60, true, 88, "High", 16),
        ("4100 Bayshore Blvd", "Tampa", "33611", "Hillsborough", "A-14-30-ZZ-000000-00250.0", 2008, 3200.0, 10000.0, "Single Family", 780000.0, 5, 4, "Tile", 10, 27.8890, -82.4878, "AE", true, false, "Low", 14, false, 30, "Low", 17),
        ("1800 Atlantic Blvd", "Jacksonville", "32207", "Duval", "073680-0000", 1995, 2100.0, 7500.0, "Single Family", 320000.0, 4, 2, "Shingle", 17, 30.3149, -81.6379, "X", false, false, "Moderate", 22, false, 42, "Medium", 18),
        ("3200 S Congress Ave", "Boynton Beach", "33426", "Palm Beach", "00-43-45-27-05-000-0100", 2012, 1900.0, 4800.0, "Townhouse", 375000.0, 3, 2, "Tile", 7, 26.5254, -80.0922, "X", false, false, "Low", 6, false, 18, "Low", 19),
        ("900 Duval St", "Key West", "33040", "Monroe", "00019120-000000", 1955, 1100.0, 3500.0, "Single Family", 890000.0, 2, 1, "Metal", 35, 24.5577, -81.8013, "VE", true, true, "Low", 10, false, 82, "High", 20),
    ];

    for (addr, city, zip, county, parcel, yr, sqft, lot, ptype, val, beds, baths, roof, roof_age, lat, lon, fz, is_fz, is_hr_fz, fire, wf_score, is_wf, risk, lead, owner_id) in &properties {
        sqlx::query(
            "INSERT INTO properties (address, city, zip_code, county, state, parcel_id, year_built, square_footage, lot_size, \
             property_type, estimated_value, bedrooms, bathrooms, roof_type, roof_age, latitude, longitude, \
             flood_zone, is_in_flood_zone, is_in_high_risk_flood_zone, fire_risk_level, wildfire_risk_score, is_in_wildfire_zone, \
             overall_risk_score, lead_priority, owner_id) \
             VALUES (?, ?, ?, ?, 'FL', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(addr).bind(city).bind(zip).bind(county).bind(parcel)
        .bind(yr).bind(sqft).bind(lot).bind(ptype).bind(val)
        .bind(beds).bind(baths).bind(roof).bind(roof_age)
        .bind(lat).bind(lon)
        .bind(fz).bind(is_fz).bind(is_hr_fz)
        .bind(fire).bind(wf_score).bind(is_wf)
        .bind(risk).bind(lead).bind(owner_id)
        .execute(pool).await.ok();
    }

    // ── Permits ─────────────────────────────────────────────────────────────
    // (permit_number, permit_type, work_category, description, issued_date, status, contractor_name, contractor_license, estimated_cost, is_roofing, property_id)
    let permits: Vec<(&str, &str, &str, &str, &str, &str, &str, &str, f64, bool, i64)> = vec![
        ("BLD2024-00145", "Roofing", "Roofing", "Complete re-roofing with architectural shingles", "2024-06-15", "Completed", "ABC Roofing Inc", "CCC1330567", 18500.0, true, 1),
        ("BLD2024-00287", "Electrical", "Electrical", "Panel upgrade 200A, whole-house rewiring", "2024-08-20", "Active", "Spark Electric LLC", "EC0001234", 12000.0, false, 1),
        ("BLD2025-00032", "Roofing", "Roofing", "Roof replacement - flat modified bitumen", "2025-01-10", "Active", "Metro Roof Systems", "CCC1331122", 22000.0, true, 2),
        ("BLD2024-00503", "Roofing", "Roofing", "Re-roof tile to tile, hurricane clips", "2024-09-05", "Completed", "Florida Tile Roofing", "CCC1329876", 35000.0, true, 3),
        ("BLD2024-00504", "HVAC", "HVAC", "AC replacement 5-ton split system", "2024-09-10", "Completed", "Cool Air Services", "CAC1816789", 8500.0, false, 3),
        ("BLD2025-00089", "Roofing", "Roofing", "Shingle replacement - wind damage repair", "2025-02-01", "Active", "StormGuard Roofing", "CCC1330890", 15000.0, true, 4),
        ("BLD2024-00612", "Windows/Doors", "Windows/Doors", "Impact window installation - 12 units", "2024-10-15", "Completed", "Hurricane Shield Windows", "GB98765", 24000.0, false, 4),
        ("BLD2025-00101", "Plumbing", "Plumbing", "Water heater replacement - tankless", "2025-01-20", "Active", "Precision Plumbing", "CFC1430123", 4500.0, false, 5),
        ("BLD2024-00789", "Roofing", "Roofing", "Complete re-roof asphalt shingle", "2024-11-01", "Completed", "Central FL Roofing", "CCC1328456", 16000.0, true, 6),
        ("BLD2024-00425", "Solar", "Solar", "10kW solar panel system installation", "2024-08-01", "Completed", "SunPower FL", "ECC003456", 28000.0, false, 6),
        ("BLD2025-00055", "Roofing", "Roofing", "Re-roofing shingle - storm damage", "2025-01-25", "Active", "Bay Area Roofers", "CCC1332211", 19500.0, true, 7),
        ("BLD2024-00666", "Structural", "Structural", "Truss repair and reinforcement", "2024-10-01", "Completed", "Structural Solutions FL", "CGC1523456", 9500.0, false, 7),
        ("BLD2024-00890", "Roofing", "Roofing", "Metal roof installation - standing seam", "2024-12-01", "Completed", "Gulf Coast Metal Roofing", "CCC1329999", 32000.0, true, 8),
        ("BLD2024-00333", "Pool", "Pool", "Screen enclosure rebuild", "2024-07-15", "Completed", "Pool Screens Plus", "SCC131234", 6500.0, false, 9),
        ("BLD2025-00140", "Roofing", "Roofing", "Flat roof membrane replacement", "2025-02-15", "Active", "Ocean Roofing Co", "CCC1331567", 20000.0, true, 10),
        ("BLD2024-00222", "Roofing", "Roofing", "Shingle re-roof and deck repair", "2024-05-20", "Completed", "Reliable Roofing Miami", "CCC1330111", 21000.0, true, 11),
        ("BLD2024-00999", "General Building", "General Building", "New construction - single family home", "2024-12-15", "Active", "Premier Homes FL", "CGC1524567", 450000.0, false, 12),
        ("BLD2025-00045", "Roofing", "Roofing", "Re-roof shingle, new underlayment", "2025-01-15", "Active", "Broward Roofing Pros", "CCC1331890", 17500.0, true, 13),
        ("BLD2024-00777", "Roofing", "Roofing", "Tile roof repair - 500 sqft section", "2024-11-10", "Completed", "Sarasota Tile & Roof", "CCC1329123", 8000.0, true, 14),
        ("BLD2024-00555", "Fence", "Fence", "6ft vinyl privacy fence installation", "2024-09-20", "Completed", "Fence Masters FL", "GB12345", 5200.0, false, 14),
        ("BLD2025-00078", "Roofing", "Roofing", "Complete re-roof - dimensional shingles", "2025-02-05", "Active", "Volusia Roofing Inc", "CCC1332345", 14500.0, true, 15),
        ("BLD2024-00111", "Roofing", "Roofing", "Emergency roof tarp and repair", "2024-03-15", "Completed", "Emergency Roof Repair LLC", "CCC1328900", 5500.0, true, 16),
        ("BLD2025-00198", "Roofing", "Roofing", "Re-roof - concrete tile replacement", "2025-03-01", "Active", "Tampa Bay Tile Roofing", "CCC1330456", 38000.0, true, 17),
        ("BLD2024-00444", "Interior Remodel", "Interior Remodel", "Kitchen renovation - full gut remodel", "2024-08-10", "Completed", "Dream Kitchens Tampa", "CBC1259876", 45000.0, false, 17),
        ("BLD2025-00067", "Roofing", "Roofing", "Shingle roof replacement - 30yr warranty", "2025-01-30", "Active", "Jax Roofing Solutions", "CCC1331456", 16500.0, true, 18),
        ("BLD2024-00888", "Siding/Exterior", "Siding/Exterior", "Stucco repair and exterior painting", "2024-11-20", "Completed", "Exterior Pro FL", "CBC1260123", 11000.0, false, 18),
        ("BLD2025-00120", "Roofing", "Roofing", "Tile re-roof with enhanced underlayment", "2025-02-10", "Active", "Palm Beach Roofing Co", "CCC1331789", 26000.0, true, 19),
        ("BLD2024-00050", "Roofing", "Roofing", "Metal roof restoration - coating system", "2024-04-01", "Completed", "Keys Roofing & Restoration", "CCC1328567", 12000.0, true, 20),
        ("BLD2024-00051", "Plumbing", "Plumbing", "Full re-pipe - copper to PEX", "2024-04-15", "Completed", "Island Plumbing", "CFC1430567", 9500.0, false, 20),
    ];

    for (num, ptype, cat, desc, issued, status, contractor, license, cost, is_roof, prop_id) in &permits {
        sqlx::query(
            "INSERT INTO permits (permit_number, permit_type, work_category, description, issued_date, status, \
             contractor_name, contractor_license, estimated_cost, is_roofing_permit, property_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(num).bind(ptype).bind(cat).bind(desc).bind(issued).bind(status)
        .bind(contractor).bind(license).bind(cost).bind(is_roof).bind(prop_id)
        .execute(pool).await.ok();
    }

    // ── Insurance Claims ────────────────────────────────────────────────────
    let claims: Vec<(&str, &str, &str, &str, &str, &str, f64, f64, &str, bool, bool, bool, bool, i64)> = vec![
        ("CLM-2024-00100", "Wind", "Hurricane wind damage", "2024-10-09", "Open", "Roof shingles torn off during Hurricane Milton", 25000.0, 18000.0, "Citizens Property Insurance", true, false, false, true, 1),
        ("CLM-2024-00101", "Flood", "Storm surge flooding", "2024-10-10", "Open", "First floor flooding from storm surge", 45000.0, 32000.0, "NFIP via Wright Flood", false, true, false, false, 3),
        ("CLM-2024-00102", "Wind", "Roof damage - high winds", "2024-09-26", "Closed", "Partial roof loss during Hurricane Helene", 18000.0, 15500.0, "Universal Insurance", true, false, false, true, 7),
        ("CLM-2024-00103", "Wind", "Hurricane damage - roof and windows", "2024-10-09", "Open", "Complete roof failure and window breach", 65000.0, 0.0, "Heritage Insurance", true, false, false, true, 10),
        ("CLM-2023-00050", "Water", "Pipe burst - water damage", "2023-08-15", "Closed", "Interior water damage from burst pipe", 12000.0, 9800.0, "State Farm", false, false, false, false, 6),
        ("CLM-2024-00104", "Flood", "Hurricane flooding", "2024-10-09", "Open", "Ground floor flooding during Milton", 35000.0, 22000.0, "NFIP via Assurant", false, true, false, false, 11),
        ("CLM-2024-00105", "Wind", "Roof tile displacement", "2024-10-09", "Open", "Multiple tiles displaced by hurricane winds", 15000.0, 12000.0, "FedNat Insurance", true, false, false, true, 13),
        ("CLM-2023-00060", "Fire", "Kitchen fire damage", "2023-12-01", "Closed", "Kitchen fire - smoke and heat damage", 28000.0, 25000.0, "Allstate", false, false, true, false, 14),
        ("CLM-2024-00106", "Flood", "Coastal flooding", "2024-10-10", "Open", "Severe coastal flooding from storm surge", 80000.0, 0.0, "NFIP via Hartford", false, true, false, false, 20),
        ("CLM-2024-00107", "Wind", "Roof and siding damage", "2024-09-27", "Closed", "Shingle and fascia damage from Helene", 22000.0, 19000.0, "Tower Hill Insurance", true, false, false, true, 16),
    ];

    for (num, ctype, cause, date_loss, status, desc, amount, paid, company, is_roof, is_flood, is_fire, is_wind, prop_id) in &claims {
        sqlx::query(
            "INSERT INTO insurance_claims (claim_number, claim_type, cause_of_loss, date_of_loss, status, description, \
             claim_amount, paid_amount, insurance_company, is_roof_claim, is_flood_claim, is_fire_claim, is_wind_claim, property_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(num).bind(ctype).bind(cause).bind(date_loss).bind(status).bind(desc)
        .bind(amount).bind(paid).bind(company)
        .bind(is_roof).bind(is_flood).bind(is_fire).bind(is_wind).bind(prop_id)
        .execute(pool).await.ok();
    }

    // ── Fire Incidents ──────────────────────────────────────────────────────
    let fire_incidents: Vec<(&str, &str, &str, &str, f64, &str, f64, f64, i64, i64)> = vec![
        ("FL-DBS-2024-0045", "Daytona Brush Fire", "Wildfire", "Volusia", 120.0, "Lightning", 29.2150, -81.0300, 2, 15),
        ("FL-HMS-2024-0012", "Homestead Grassland Fire", "Wildfire", "Miami-Dade", 85.5, "Unknown", 25.4500, -80.4600, 0, 16),
        ("FL-HMS-2023-0033", "SW Dade Wildland Fire", "Wildfire", "Miami-Dade", 210.0, "Human", 25.4200, -80.5100, 1, 16),
    ];

    for (iid, name, itype, county, acres, cause, lat, lon, structures, prop_id) in &fire_incidents {
        sqlx::query(
            "INSERT INTO fire_incidents (incident_id, incident_name, incident_type, county, acres_burned, cause, \
             latitude, longitude, structures_damaged, property_id) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(iid).bind(name).bind(itype).bind(county).bind(acres).bind(cause)
        .bind(lat).bind(lon).bind(structures).bind(prop_id)
        .execute(pool).await.ok();
    }

    let prop_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM properties")
        .fetch_one(pool).await.unwrap_or((0,));
    let permit_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM permits")
        .fetch_one(pool).await.unwrap_or((0,));
    let claim_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM insurance_claims")
        .fetch_one(pool).await.unwrap_or((0,));

    info!(
        "Seeded sample data: {} properties, {} permits, {} insurance claims",
        prop_count.0, permit_count.0, claim_count.0
    );
}
