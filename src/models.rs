use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// ============================================================================
// 1. Property
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Property {
    pub id: i64,
    pub address: String,
    pub city: String,
    pub zip_code: String,
    pub county: String,
    pub state: String,
    pub parcel_id: String,
    pub year_built: Option<i64>,
    pub square_footage: Option<f64>,
    pub lot_size: Option<f64>,
    pub property_type: String,
    pub estimated_value: Option<f64>,
    pub bedrooms: Option<i64>,
    pub bathrooms: Option<i64>,
    pub roof_type: String,
    pub roof_age: Option<i64>,
    pub last_roof_permit_date: Option<NaiveDateTime>,
    pub image_url: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    // Flood zone fields
    pub flood_zone: Option<String>,
    pub flood_zone_description: Option<String>,
    pub is_in_flood_zone: bool,
    pub is_in_high_risk_flood_zone: bool,
    pub fema_map_panel: Option<String>,
    pub flood_zone_last_updated: Option<NaiveDateTime>,
    pub requires_flood_insurance: bool,
    // Fire risk fields
    pub fire_risk_level: Option<String>,
    pub wildfire_risk_score: Option<i64>,
    pub distance_to_fire_station: Option<f64>,
    pub fire_protection_class: Option<String>,
    pub is_in_wildfire_zone: bool,
    // Insurance fields
    pub has_homeowners_insurance: bool,
    pub has_flood_insurance: bool,
    pub has_wind_insurance: bool,
    pub insured_value: Option<f64>,
    pub insurance_carrier: Option<String>,
    // Scoring / lead fields
    pub overall_risk_score: Option<i64>,
    pub lead_priority: Option<String>,
    pub data_source: Option<String>,
    pub last_data_refresh: Option<NaiveDateTime>,
    pub owner_id: Option<i64>,
}

// ============================================================================
// 2. Owner
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Owner {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub mailing_address: Option<String>,
    pub mailing_city: Option<String>,
    pub mailing_state: Option<String>,
    pub mailing_zip: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub age: Option<i64>,
    pub date_of_birth: Option<NaiveDateTime>,
}

impl Owner {
    /// Returns the owner's full name.  Falls back to `"Unknown"` when both
    /// first and last name are blank.
    pub fn full_name(&self) -> String {
        let first = self.first_name.trim();
        let last = self.last_name.trim();
        if first.is_empty() && last.is_empty() {
            "Unknown".to_string()
        } else if first.is_empty() {
            last.to_string()
        } else if last.is_empty() {
            first.to_string()
        } else {
            format!("{} {}", first, last)
        }
    }
}

// ============================================================================
// 3. Permit  +  classify_permit()
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Permit {
    pub id: i64,
    pub permit_number: String,
    pub permit_type: String,
    pub work_category: String,
    pub description: Option<String>,
    pub issued_date: Option<NaiveDateTime>,
    pub completed_date: Option<NaiveDateTime>,
    pub expiration_date: Option<NaiveDateTime>,
    pub status: String,
    pub contractor_name: Option<String>,
    pub contractor_license: Option<String>,
    pub estimated_cost: Option<f64>,
    pub is_roofing_permit: bool,
    pub data_source: Option<String>,
    pub property_id: i64,
}

/// Classify a permit based on its type and description strings.
///
/// Returns `(work_category, is_roofing)` covering all 17 categories ported
/// from the original C# implementation:
///
/// 1. Roofing
/// 2. Electrical
/// 3. Plumbing
/// 4. HVAC
/// 5. Windows/Doors
/// 6. Structural
/// 7. Siding/Exterior
/// 8. Foundation
/// 9. Solar
/// 10. Pool
/// 11. Fence
/// 12. Fire Alarm
/// 13. Demolition
/// 14. Interior Remodel
/// 15. General Building
/// 16. Other (fallback)
///
/// The first matching category wins (order matters -- more specific patterns
/// are checked before broader ones).
pub fn classify_permit(permit_type: &str, description: &str) -> (String, bool) {
    let ptype = permit_type.to_lowercase();
    let desc = description.to_lowercase();

    // (keywords, category_label, is_roofing)
    let rules: &[(&[&str], &str, bool)] = &[
        // 1 - Roofing
        (
            &["roof", "reroofing", "re-roof", "shingle", "roofing"],
            "Roofing",
            true,
        ),
        // 2 - Electrical
        (
            &["electr", "wiring", "panel upgrade", "circuit", "outlet"],
            "Electrical",
            false,
        ),
        // 3 - Plumbing
        (
            &[
                "plumb", "sewer", "water heater", "water line", "drain",
                "pipe", "backflow",
            ],
            "Plumbing",
            false,
        ),
        // 4 - HVAC
        (
            &[
                "hvac", "air condition", "a/c", "ac unit", "heating",
                "furnace", "duct", "heat pump", "mechanical",
            ],
            "HVAC",
            false,
        ),
        // 5 - Windows/Doors
        (
            &[
                "window", "door", "glass", "glazing", "impact window",
                "impact door", "shutter",
            ],
            "Windows/Doors",
            false,
        ),
        // 6 - Structural
        (
            &["structur", "load bearing", "beam", "column", "truss"],
            "Structural",
            false,
        ),
        // 7 - Siding/Exterior
        (
            &[
                "siding", "stucco", "exterior", "facade", "cladding",
                "paint", "soffit", "fascia",
            ],
            "Siding/Exterior",
            false,
        ),
        // 8 - Foundation
        (
            &["foundation", "slab", "footing", "pier", "piling"],
            "Foundation",
            false,
        ),
        // 9 - Solar
        (
            &["solar", "photovoltaic", "pv system", "pv panel"],
            "Solar",
            false,
        ),
        // 10 - Pool
        (
            &["pool", "spa", "hot tub", "screen enclosure", "pool deck"],
            "Pool",
            false,
        ),
        // 11 - Fence
        (
            &["fence", "fencing", "gate"],
            "Fence",
            false,
        ),
        // 12 - Fire Alarm
        (
            &[
                "fire alarm", "fire sprinkler", "fire suppression",
                "fire protection", "smoke detector",
            ],
            "Fire Alarm",
            false,
        ),
        // 13 - Demolition
        (
            &["demo", "demolition", "tear down", "removal"],
            "Demolition",
            false,
        ),
        // 14 - Interior Remodel
        (
            &[
                "interior", "remodel", "renovation", "kitchen", "bathroom",
                "flooring", "drywall", "cabinet", "tile",
            ],
            "Interior Remodel",
            false,
        ),
        // 15 - General Building
        (
            &[
                "building", "construct", "addition", "alteration",
                "new construction", "commercial", "residential",
            ],
            "General Building",
            false,
        ),
    ];

    for (keywords, category, is_roofing) in rules {
        for kw in *keywords {
            if ptype.contains(kw) || desc.contains(kw) {
                return (category.to_string(), *is_roofing);
            }
        }
    }

    // 16 - Other (fallback)
    ("Other".to_string(), false)
}

// ============================================================================
// 4. InsuranceClaim
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct InsuranceClaim {
    pub id: i64,
    pub claim_number: String,
    pub claim_type: String,
    pub cause_of_loss: String,
    pub date_of_loss: Option<NaiveDateTime>,
    pub date_filed: Option<NaiveDateTime>,
    pub date_closed: Option<NaiveDateTime>,
    pub status: String,
    pub claim_amount: Option<f64>,
    pub paid_amount: Option<f64>,
    pub deductible: Option<f64>,
    pub insurance_company: Option<String>,
    pub policy_number: Option<String>,
    pub description: Option<String>,
    pub is_roof_claim: bool,
    pub is_flood_claim: bool,
    pub is_fire_claim: bool,
    pub is_wind_claim: bool,
    pub public_adjuster_involved: bool,
    pub public_adjuster_name: Option<String>,
    pub fema_disaster_number: Option<String>,
    pub property_id: i64,
}

// ============================================================================
// 5. FireIncident
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FireIncident {
    pub id: i64,
    pub incident_id: String,
    pub incident_name: String,
    pub incident_type: String,
    pub discovery_date: Option<NaiveDateTime>,
    pub contained_date: Option<NaiveDateTime>,
    pub acres_burned: Option<f64>,
    pub county: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub distance_to_property: Option<f64>,
    pub cause: Option<String>,
    pub structures_damaged: Option<i64>,
    pub structures_destroyed: Option<i64>,
    pub description: Option<String>,
    pub source: Option<String>,
    pub property_id: i64,
}

// ============================================================================
// 6. FemaDisaster
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct FemaDisaster {
    pub id: i64,
    pub disaster_number: String,
    pub title: String,
    pub disaster_type: String,
    pub declaration_date: Option<NaiveDateTime>,
    pub incident_begin_date: Option<NaiveDateTime>,
    pub incident_end_date: Option<NaiveDateTime>,
    pub closeout_date: Option<NaiveDateTime>,
    pub state: String,
    pub declared_county: Option<String>,
    pub incident_type: Option<String>,
    pub programs_available: Option<String>,
    pub individual_assistance: bool,
    pub public_assistance: bool,
    pub hazard_mitigation: bool,
    pub fema_url: Option<String>,
}

// ============================================================================
// 7. FloodZoneInfo (value object -- not a database row)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FloodZoneInfo {
    pub flood_zone: String,
    pub zone_description: String,
    pub is_high_risk: bool,
    pub requires_insurance: bool,
    pub map_panel: Option<String>,
}

// ============================================================================
// 8. SearchResult
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SearchResult {
    pub property_id: i64,
    pub address: String,
    pub city: String,
    pub county: String,
    pub zip_code: String,
    pub parcel_id: String,
    pub property_type: String,
    pub year_built: Option<i64>,
    pub roof_type: String,
    pub roof_age: Option<i64>,
    pub estimated_value: Option<f64>,
    pub flood_zone: Option<String>,
    pub is_in_flood_zone: bool,
    pub is_in_high_risk_flood_zone: bool,
    pub fire_risk_level: Option<String>,
    pub overall_risk_score: Option<i64>,
    pub lead_priority: Option<String>,
    pub owner_name: Option<String>,
    pub permit_count: Option<i64>,
    pub last_permit_date: Option<NaiveDateTime>,
    pub claim_count: Option<i64>,
}

// ============================================================================
// 9. SearchRequest  +  SearchMode enum
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SearchRequest {
    pub query: Option<String>,
    pub mode: SearchMode,
    pub city: Option<String>,
    pub county: Option<String>,
    pub zip_code: Option<String>,
    pub min_roof_age: Option<i64>,
    pub max_roof_age: Option<i64>,
    pub in_flood_zone: Option<bool>,
    pub high_fire_risk: Option<bool>,
    pub lead_priority: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum SearchMode {
    #[default]
    Address,
    Owner,
}

// ============================================================================
// 10. DashboardStats / CountyBreakdown / RoofAgeDistribution
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DashboardStats {
    pub total_properties: i64,
    pub total_permits: i64,
    pub total_claims: i64,
    pub total_disasters: i64,
    pub properties_in_flood_zone: i64,
    pub properties_high_fire_risk: i64,
    pub average_roof_age: f64,
    pub high_priority_leads: i64,
    pub medium_priority_leads: i64,
    pub low_priority_leads: i64,
    pub county_breakdown: Vec<CountyBreakdown>,
    pub roof_age_distribution: Vec<RoofAgeDistribution>,
}

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CountyBreakdown {
    pub county: String,
    pub property_count: i64,
    pub permit_count: i64,
    pub average_roof_age: Option<f64>,
    pub flood_zone_count: i64,
    pub high_fire_risk_count: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoofAgeDistribution {
    pub range: String,
    pub count: i64,
}

// ============================================================================
// 11. PaginatedResult<T>
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i64,
    pub page_size: i64,
}

impl<T> PaginatedResult<T> {
    /// Total number of pages given the current page_size.
    pub fn total_pages(&self) -> i64 {
        if self.page_size <= 0 {
            return 0;
        }
        (self.total_count + self.page_size - 1) / self.page_size
    }

    /// Whether there is a page before the current one.
    pub fn has_previous_page(&self) -> bool {
        self.page > 1
    }

    /// Whether there is a page after the current one.
    pub fn has_next_page(&self) -> bool {
        self.page < self.total_pages()
    }
}

// ============================================================================
// 12. WorkHistoryEntry
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct WorkHistoryEntry {
    pub work_category: String,
    pub total_permits: i64,
    pub first_permit_date: Option<NaiveDateTime>,
    pub last_permit_date: Option<NaiveDateTime>,
    pub most_recent_status: Option<String>,
    pub total_estimated_cost: Option<f64>,
    pub years_since_last_work: Option<f64>,
}

// ============================================================================
// 13. CountyStats
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct CountyStats {
    pub county: String,
    pub property_count: i64,
    pub permit_count: i64,
    pub old_roof_count: i64,
    pub flood_zone_count: i64,
    pub high_fire_risk_count: i64,
    pub disaster_count: i64,
}

// ============================================================================
// 14. User
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: String,
    pub email: String,
    pub full_name: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    pub created_at: NaiveDateTime,
}

// ============================================================================
// 15. PropertyRecord (DTO from external API)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropertyRecord {
    pub address: Option<String>,
    pub city: Option<String>,
    pub county: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub parcel_id: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub data_source: Option<String>,
    pub year_built: Option<i64>,
    pub square_footage: Option<f64>,
    pub lot_size: Option<f64>,
    pub property_type: Option<String>,
    pub just_value: Option<f64>,
    pub assessed_value: Option<f64>,
    pub taxable_value: Option<f64>,
    pub bedrooms: Option<i64>,
    pub bathrooms: Option<i64>,
    pub owner_name: Option<String>,
    pub owner_address: Option<String>,
    pub owner_city: Option<String>,
    pub owner_state: Option<String>,
    pub owner_zip: Option<String>,
}

// ============================================================================
// 16. PermitRecord (DTO from external API)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PermitRecord {
    pub permit_number: Option<String>,
    pub permit_type: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub issued_date: Option<NaiveDateTime>,
    pub contractor_name: Option<String>,
    pub contractor_license: Option<String>,
    pub estimated_cost: Option<f64>,
    pub address: Option<String>,
}

// ============================================================================
// 17. WildfireIncident (DTO from NIFC IRWIN ArcGIS API)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WildfireIncident {
    pub incident_name: String,
    pub irwin_id: String,
    pub discovery_date: Option<NaiveDateTime>,
    pub contained_date: Option<NaiveDateTime>,
    pub acres_burned: f64,
    pub percent_contained: f64,
    pub county: String,
    pub cause: String,
    pub latitude: f64,
    pub longitude: f64,
    pub residences_destroyed: i64,
    pub residences_threatened: i64,
    pub is_active: bool,
}

// ============================================================================
// 18. WeatherAlert (DTO from NOAA NWS API)
// ============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WeatherAlert {
    pub id: String,
    pub event: String,
    pub severity: String,
    pub urgency: String,
    pub headline: String,
    pub description: String,
    pub effective: Option<String>,
    pub expires: Option<String>,
}
