CREATE TABLE IF NOT EXISTS owners (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    first_name TEXT NOT NULL DEFAULT '',
    last_name TEXT NOT NULL DEFAULT '',
    mailing_address TEXT,
    mailing_city TEXT,
    mailing_state TEXT,
    mailing_zip TEXT,
    phone_number TEXT,
    email TEXT,
    age INTEGER,
    date_of_birth TEXT
);

CREATE TABLE IF NOT EXISTS properties (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    address TEXT NOT NULL DEFAULT '',
    city TEXT NOT NULL DEFAULT '',
    zip_code TEXT NOT NULL DEFAULT '',
    county TEXT NOT NULL DEFAULT '',
    state TEXT NOT NULL DEFAULT 'FL',
    parcel_id TEXT NOT NULL DEFAULT '',
    year_built INTEGER,
    square_footage REAL,
    lot_size REAL,
    property_type TEXT NOT NULL DEFAULT '',
    estimated_value REAL,
    bedrooms INTEGER,
    bathrooms INTEGER,
    roof_type TEXT NOT NULL DEFAULT '',
    roof_age INTEGER,
    last_roof_permit_date TEXT,
    image_url TEXT,
    latitude REAL,
    longitude REAL,
    flood_zone TEXT,
    flood_zone_description TEXT,
    is_in_flood_zone INTEGER NOT NULL DEFAULT 0,
    is_in_high_risk_flood_zone INTEGER NOT NULL DEFAULT 0,
    fema_map_panel TEXT,
    flood_zone_last_updated TEXT,
    requires_flood_insurance INTEGER NOT NULL DEFAULT 0,
    fire_risk_level TEXT,
    wildfire_risk_score INTEGER,
    distance_to_fire_station REAL,
    fire_protection_class TEXT,
    is_in_wildfire_zone INTEGER NOT NULL DEFAULT 0,
    has_homeowners_insurance INTEGER NOT NULL DEFAULT 0,
    has_flood_insurance INTEGER NOT NULL DEFAULT 0,
    has_wind_insurance INTEGER NOT NULL DEFAULT 0,
    insured_value REAL,
    insurance_carrier TEXT,
    overall_risk_score INTEGER,
    lead_priority TEXT,
    data_source TEXT,
    last_data_refresh TEXT,
    owner_id INTEGER REFERENCES owners(id)
);

CREATE TABLE IF NOT EXISTS permits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    permit_number TEXT NOT NULL DEFAULT '',
    permit_type TEXT NOT NULL DEFAULT '',
    work_category TEXT NOT NULL DEFAULT '',
    description TEXT,
    issued_date TEXT,
    completed_date TEXT,
    expiration_date TEXT,
    status TEXT NOT NULL DEFAULT '',
    contractor_name TEXT,
    contractor_license TEXT,
    estimated_cost REAL,
    is_roofing_permit INTEGER NOT NULL DEFAULT 0,
    data_source TEXT,
    property_id INTEGER NOT NULL REFERENCES properties(id)
);

CREATE TABLE IF NOT EXISTS insurance_claims (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    claim_number TEXT NOT NULL DEFAULT '',
    claim_type TEXT NOT NULL DEFAULT '',
    cause_of_loss TEXT NOT NULL DEFAULT '',
    date_of_loss TEXT,
    date_filed TEXT,
    date_closed TEXT,
    status TEXT NOT NULL DEFAULT '',
    claim_amount REAL,
    paid_amount REAL,
    deductible REAL,
    insurance_company TEXT,
    policy_number TEXT,
    description TEXT,
    is_roof_claim INTEGER NOT NULL DEFAULT 0,
    is_flood_claim INTEGER NOT NULL DEFAULT 0,
    is_fire_claim INTEGER NOT NULL DEFAULT 0,
    is_wind_claim INTEGER NOT NULL DEFAULT 0,
    public_adjuster_involved INTEGER NOT NULL DEFAULT 0,
    public_adjuster_name TEXT,
    fema_disaster_number TEXT,
    property_id INTEGER NOT NULL REFERENCES properties(id)
);

CREATE TABLE IF NOT EXISTS fire_incidents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    incident_id TEXT NOT NULL DEFAULT '',
    incident_name TEXT NOT NULL DEFAULT '',
    incident_type TEXT NOT NULL DEFAULT '',
    discovery_date TEXT,
    contained_date TEXT,
    acres_burned REAL,
    county TEXT,
    latitude REAL,
    longitude REAL,
    distance_to_property REAL,
    cause TEXT,
    structures_damaged INTEGER,
    structures_destroyed INTEGER,
    description TEXT,
    source TEXT,
    property_id INTEGER NOT NULL REFERENCES properties(id)
);

CREATE TABLE IF NOT EXISTS fema_disasters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    disaster_number TEXT NOT NULL DEFAULT '',
    title TEXT NOT NULL DEFAULT '',
    disaster_type TEXT NOT NULL DEFAULT '',
    declaration_date TEXT,
    incident_begin_date TEXT,
    incident_end_date TEXT,
    closeout_date TEXT,
    state TEXT NOT NULL DEFAULT 'FL',
    declared_county TEXT,
    incident_type TEXT,
    programs_available TEXT,
    individual_assistance INTEGER NOT NULL DEFAULT 0,
    public_assistance INTEGER NOT NULL DEFAULT 0,
    hazard_mitigation INTEGER NOT NULL DEFAULT 0,
    fema_url TEXT
);

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    email TEXT NOT NULL UNIQUE,
    full_name TEXT,
    avatar_url TEXT,
    provider TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id),
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_properties_address ON properties(address);
CREATE INDEX IF NOT EXISTS idx_properties_city ON properties(city);
CREATE INDEX IF NOT EXISTS idx_properties_county ON properties(county);
CREATE INDEX IF NOT EXISTS idx_properties_zip ON properties(zip_code);
CREATE INDEX IF NOT EXISTS idx_properties_flood ON properties(flood_zone);
CREATE INDEX IF NOT EXISTS idx_properties_high_risk ON properties(is_in_high_risk_flood_zone);
CREATE INDEX IF NOT EXISTS idx_properties_fire_risk ON properties(fire_risk_level);
CREATE INDEX IF NOT EXISTS idx_properties_risk_score ON properties(overall_risk_score);
CREATE INDEX IF NOT EXISTS idx_properties_lead ON properties(lead_priority);
CREATE INDEX IF NOT EXISTS idx_owners_name ON owners(last_name, first_name);
CREATE INDEX IF NOT EXISTS idx_claims_type ON insurance_claims(claim_type);
CREATE INDEX IF NOT EXISTS idx_claims_date ON insurance_claims(date_of_loss);
CREATE INDEX IF NOT EXISTS idx_disasters_number ON fema_disasters(disaster_number);
CREATE INDEX IF NOT EXISTS idx_disasters_county ON fema_disasters(declared_county);
