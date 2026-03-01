# FL Property & Permit Tracker

A C# Blazor Server web application for tracking Florida property owner information, roofing permits, and generating leads for roofing contractors.

## Features

### Property Search
- **Search by Address** - Find properties by street address, city, or zip code
- **Search by Owner** - Look up properties owned by a specific person
- **Advanced Filters** - Filter by county, roof age range, and property type

### Property Details
- Complete property information (year built, square footage, bedrooms, bathrooms)
- Owner contact information (phone, email, age, mailing address)
- Roof type and roof age tracking
- Estimated property value
- Parcel ID and county data

### Permit Tracking
- Full permit history for every property
- Roofing permit filtering
- Contractor name and license tracking
- Permit status, dates, and estimated costs
- Recent permits feed across all properties

### Dashboard
- Total properties, owners, and permits overview
- Properties with roofs 15+ years old (high-value leads)
- County-by-county breakdown with lead potential scoring
- Roof age distribution chart

### API
- REST API endpoints for property search, details, and permits
- **Coming Soon** features:
  - Real-Time Permit Webhooks (Q3 2026)
  - Batch Property Lookup (Q3 2026)
  - AI Lead Scoring (Q4 2026)
  - Storm Damage Assessment (Q4 2026)
  - Automated Outreach Campaigns (Q1 2027)
  - Satellite Roof Analysis (Q1 2027)

## Tech Stack

- **Framework**: ASP.NET Core 8.0 / Blazor Server
- **Database**: SQLite via Entity Framework Core
- **Frontend**: Blazor Components + Custom CSS (dark teal/blue theme)
- **API**: ASP.NET Core Web API Controllers
- **Language**: C# (entirely)

## Getting Started

### Prerequisites
- [.NET 8 SDK](https://dotnet.microsoft.com/download/dotnet/8.0)

### Run the Application

```bash
cd RoofingLeadGen
dotnet restore
dotnet run
```

Open your browser to `https://localhost:7142`

The database is automatically created and seeded with 20 Florida properties, owners, and permit data on first run.

### API Endpoints

| Endpoint | Description |
|---|---|
| `GET /api/properties/search?mode=address&q=miami` | Search by address |
| `GET /api/properties/search?mode=owner&q=rodriguez` | Search by owner name |
| `GET /api/properties/{id}` | Property details |
| `GET /api/properties/{id}/permits` | Property permit history |
| `GET /api/properties/stats` | Dashboard statistics |
| `GET /api/properties/counties` | List all counties |
| `GET /api/v2/features` | API feature list & roadmap |

## Project Structure

```
RoofingLeadGen/
├── Models/           # Data models (Property, Owner, Permit, SearchResult)
├── Data/             # EF Core DbContext and seed data
├── Services/         # Business logic (PropertyService, PermitService)
├── Controllers/      # REST API controllers
├── Pages/            # Blazor pages
│   ├── Shared/       # Layout components
│   ├── Dashboard     # Main dashboard
│   ├── Search        # Property search (address & owner)
│   ├── PropertyDetail # Individual property view
│   ├── RecentPermits # Recent permit feed
│   └── ApiStatus     # API docs & Coming Soon features
└── wwwroot/css/      # Dark theme stylesheet
```
