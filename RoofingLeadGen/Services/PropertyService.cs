using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Data;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Services;

public class PropertyService
{
    private readonly AppDbContext _context;
    private readonly FloridaPropertyDataClient _propertyClient;
    private readonly FemaApiClient _femaClient;
    private readonly FloridaFireDataClient _fireClient;
    private readonly ILogger<PropertyService> _logger;

    public PropertyService(
        AppDbContext context,
        FloridaPropertyDataClient propertyClient,
        FemaApiClient femaClient,
        FloridaFireDataClient fireClient,
        ILogger<PropertyService> logger)
    {
        _context = context;
        _propertyClient = propertyClient;
        _femaClient = femaClient;
        _fireClient = fireClient;
        _logger = logger;
    }

    public async Task<PaginatedResult<SearchResult>> SearchByAddressAsync(SearchRequest request)
    {
        var query = _context.Properties
            .Include(p => p.Owner)
            .Include(p => p.Permits)
            .Include(p => p.InsuranceClaims)
            .AsQueryable();

        if (!string.IsNullOrWhiteSpace(request.Query))
        {
            var searchTerm = request.Query.ToLower();
            query = query.Where(p =>
                p.Address.ToLower().Contains(searchTerm) ||
                p.City.ToLower().Contains(searchTerm) ||
                p.ZipCode.Contains(searchTerm) ||
                p.ParcelId.ToLower().Contains(searchTerm));
        }

        query = ApplyFilters(query, request);
        var totalCount = await query.CountAsync();

        // If no local results, fetch from live FL property APIs
        if (totalCount == 0 && !string.IsNullOrWhiteSpace(request.Query))
        {
            await FetchAndCacheFromLiveApiAsync(request.Query, request.County);

            query = _context.Properties
                .Include(p => p.Owner).Include(p => p.Permits).Include(p => p.InsuranceClaims)
                .AsQueryable();

            var searchTerm = request.Query.ToLower();
            query = query.Where(p =>
                p.Address.ToLower().Contains(searchTerm) ||
                p.City.ToLower().Contains(searchTerm) ||
                p.ZipCode.Contains(searchTerm));

            query = ApplyFilters(query, request);
            totalCount = await query.CountAsync();
        }

        var items = await query
            .OrderBy(p => p.Address)
            .Skip((request.Page - 1) * request.PageSize)
            .Take(request.PageSize)
            .Select(p => MapToSearchResult(p))
            .ToListAsync();

        return new PaginatedResult<SearchResult>
        {
            Items = items, TotalCount = totalCount,
            Page = request.Page, PageSize = request.PageSize
        };
    }

    public async Task<PaginatedResult<SearchResult>> SearchByOwnerAsync(SearchRequest request)
    {
        var query = _context.Properties
            .Include(p => p.Owner).Include(p => p.Permits).Include(p => p.InsuranceClaims)
            .Where(p => p.Owner != null)
            .AsQueryable();

        if (!string.IsNullOrWhiteSpace(request.Query))
        {
            var searchTerm = request.Query.ToLower();
            query = query.Where(p =>
                (p.Owner!.FirstName + " " + p.Owner.LastName).ToLower().Contains(searchTerm) ||
                p.Owner.LastName.ToLower().Contains(searchTerm) ||
                p.Owner.FirstName.ToLower().Contains(searchTerm));
        }

        query = ApplyFilters(query, request);
        var totalCount = await query.CountAsync();

        // If no local results and county specified, try live FDOT owner search
        if (totalCount == 0 && !string.IsNullOrWhiteSpace(request.Query) && !string.IsNullOrWhiteSpace(request.County))
        {
            await FetchAndCacheOwnerFromLiveApiAsync(request.Query, request.County);

            query = _context.Properties.Include(p => p.Owner).Include(p => p.Permits).Include(p => p.InsuranceClaims)
                .Where(p => p.Owner != null).AsQueryable();

            var searchTerm = request.Query.ToLower();
            query = query.Where(p =>
                (p.Owner!.FirstName + " " + p.Owner.LastName).ToLower().Contains(searchTerm) ||
                p.Owner.LastName.ToLower().Contains(searchTerm));

            query = ApplyFilters(query, request);
            totalCount = await query.CountAsync();
        }

        var items = await query
            .OrderBy(p => p.Owner!.LastName).ThenBy(p => p.Owner!.FirstName)
            .Skip((request.Page - 1) * request.PageSize)
            .Take(request.PageSize)
            .Select(p => MapToSearchResult(p))
            .ToListAsync();

        return new PaginatedResult<SearchResult>
        {
            Items = items, TotalCount = totalCount,
            Page = request.Page, PageSize = request.PageSize
        };
    }

    public async Task<Property?> GetPropertyByIdAsync(int id)
    {
        var property = await _context.Properties
            .Include(p => p.Owner)
            .Include(p => p.Permits.OrderByDescending(permit => permit.IssuedDate))
            .Include(p => p.InsuranceClaims.OrderByDescending(c => c.DateOfLoss))
            .Include(p => p.FireIncidents.OrderByDescending(f => f.DiscoveryDate))
            .FirstOrDefaultAsync(p => p.Id == id);

        if (property != null && string.IsNullOrEmpty(property.FloodZone) &&
            property.Latitude.HasValue && property.Longitude.HasValue)
        {
            await EnrichWithFloodDataAsync(property);
        }

        return property;
    }

    public async Task<DashboardStats> GetDashboardStatsAsync()
    {
        var properties = await _context.Properties
            .Include(p => p.Permits).Include(p => p.InsuranceClaims).ToListAsync();

        return new DashboardStats
        {
            TotalProperties = properties.Count,
            TotalOwners = await _context.Owners.CountAsync(),
            TotalPermits = await _context.Permits.CountAsync(),
            RoofsOver15Years = properties.Count(p => p.RoofAge.HasValue && p.RoofAge > 15),
            RecentPermits30Days = await _context.Permits
                .CountAsync(p => p.IssuedDate.HasValue && p.IssuedDate > DateTime.Now.AddDays(-30)),
            PropertiesInFloodZone = properties.Count(p => p.IsInFloodZone),
            PropertiesHighRiskFlood = properties.Count(p => p.IsInHighRiskFloodZone),
            PropertiesHighFireRisk = properties.Count(p => p.WildfireRiskScore > 50),
            TotalInsuranceClaims = await _context.InsuranceClaims.CountAsync(),
            OpenClaims = await _context.InsuranceClaims.CountAsync(c => c.Status == "Open" || c.Status == "In Progress"),
            FemaDisasters = await _context.FemaDisasters.CountAsync(),
            CountyBreakdowns = properties.GroupBy(p => p.County).Select(g => new CountyBreakdown
            {
                County = g.Key, PropertyCount = g.Count(),
                OldRoofCount = g.Count(p => p.RoofAge.HasValue && p.RoofAge > 15),
                FloodZoneCount = g.Count(p => p.IsInFloodZone),
                HighFireRiskCount = g.Count(p => p.WildfireRiskScore > 50),
            }).OrderByDescending(c => c.PropertyCount).ToList(),
            RoofAgeDistributions = new List<RoofAgeDistribution>
            {
                new() { Range = "0-5 years", Count = properties.Count(p => p.RoofAge is >= 0 and <= 5) },
                new() { Range = "6-10 years", Count = properties.Count(p => p.RoofAge is >= 6 and <= 10) },
                new() { Range = "11-15 years", Count = properties.Count(p => p.RoofAge is >= 11 and <= 15) },
                new() { Range = "16-20 years", Count = properties.Count(p => p.RoofAge is >= 16 and <= 20) },
                new() { Range = "20+ years", Count = properties.Count(p => p.RoofAge > 20) },
            },
        };
    }

    public async Task<List<string>> GetCountiesAsync()
    {
        var dbCounties = await _context.Properties.Select(p => p.County)
            .Where(c => !string.IsNullOrEmpty(c)).Distinct().ToListAsync();
        return dbCounties.Union(_propertyClient.GetSupportedCounties(), StringComparer.OrdinalIgnoreCase)
            .OrderBy(c => c).ToList();
    }

    public async Task<List<string>> GetCitiesAsync(string? county = null)
    {
        var query = _context.Properties.AsQueryable();
        if (!string.IsNullOrEmpty(county)) query = query.Where(p => p.County == county);
        return await query.Select(p => p.City).Where(c => !string.IsNullOrEmpty(c))
            .Distinct().OrderBy(c => c).ToListAsync();
    }

    public async Task<List<FemaDisaster>> GetRecentDisastersAsync(int count = 20)
    {
        return await _context.FemaDisasters.OrderByDescending(d => d.DeclarationDate).Take(count).ToListAsync();
    }

    private async Task FetchAndCacheFromLiveApiAsync(string query, string? county)
    {
        try
        {
            var records = await _propertyClient.SearchFloridaPropertiesAsync(query, county);
            foreach (var record in records.Take(50))
            {
                if (!string.IsNullOrEmpty(record.ParcelId) &&
                    await _context.Properties.AnyAsync(p => p.ParcelId == record.ParcelId)) continue;
                if (await _context.Properties.AnyAsync(p => p.Address == record.Address && p.City == record.City)) continue;

                var owner = await CreateOrGetOwnerAsync(record);
                var riskScore = _fireClient.CalculateWildfireRiskScore(record.Latitude, record.Longitude, record.County);

                _context.Properties.Add(new Property
                {
                    Address = record.Address, City = record.City, ZipCode = record.ZipCode,
                    County = record.County, ParcelId = record.ParcelId, YearBuilt = record.YearBuilt,
                    SquareFootage = record.SquareFootage, EstimatedValue = record.JustValue ?? record.AssessedValue,
                    Latitude = record.Latitude, Longitude = record.Longitude,
                    DataSource = record.DataSource, LastDataRefresh = DateTime.UtcNow,
                    OwnerId = owner?.Id, WildfireRiskScore = riskScore,
                    FireRiskLevel = riskScore <= 20 ? "Low" : riskScore <= 40 ? "Moderate" : riskScore <= 60 ? "High" : "Very High",
                });
            }
            await _context.SaveChangesAsync();
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch/cache live property data for '{Query}'", query);
        }
    }

    private async Task FetchAndCacheOwnerFromLiveApiAsync(string ownerName, string county)
    {
        try
        {
            var records = await _propertyClient.SearchByOwnerNameAsync(ownerName, county);
            foreach (var record in records.Take(50))
            {
                if (!string.IsNullOrEmpty(record.ParcelId) &&
                    await _context.Properties.AnyAsync(p => p.ParcelId == record.ParcelId)) continue;

                var owner = await CreateOrGetOwnerAsync(record);
                var riskScore = _fireClient.CalculateWildfireRiskScore(record.Latitude, record.Longitude, record.County);

                _context.Properties.Add(new Property
                {
                    Address = record.Address, City = record.City, ZipCode = record.ZipCode,
                    County = record.County, ParcelId = record.ParcelId, YearBuilt = record.YearBuilt,
                    SquareFootage = record.SquareFootage, EstimatedValue = record.JustValue ?? record.AssessedValue,
                    Latitude = record.Latitude, Longitude = record.Longitude,
                    DataSource = record.DataSource, LastDataRefresh = DateTime.UtcNow,
                    OwnerId = owner?.Id, WildfireRiskScore = riskScore,
                    FireRiskLevel = riskScore <= 20 ? "Low" : riskScore <= 40 ? "Moderate" : riskScore <= 60 ? "High" : "Very High",
                });
            }
            await _context.SaveChangesAsync();
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch/cache owner data for '{Owner}'", ownerName);
        }
    }

    private async Task<Owner?> CreateOrGetOwnerAsync(PropertyRecord record)
    {
        if (string.IsNullOrEmpty(record.OwnerName)) return null;
        var nameParts = record.OwnerName.Split(new[] { ' ', ',' }, 2, StringSplitOptions.RemoveEmptyEntries);
        var lastName = nameParts[0].Trim();
        var firstName = nameParts.Length > 1 ? nameParts[1].Trim() : "";

        var existing = await _context.Owners.FirstOrDefaultAsync(o => o.LastName == lastName && o.FirstName == firstName);
        if (existing != null) return existing;

        var owner = new Owner
        {
            FirstName = firstName, LastName = lastName,
            MailingAddress = record.OwnerAddress, MailingCity = record.OwnerCity,
            MailingState = record.OwnerState ?? "FL", MailingZip = record.OwnerZip,
        };
        _context.Owners.Add(owner);
        await _context.SaveChangesAsync();
        return owner;
    }

    private async Task EnrichWithFloodDataAsync(Property property)
    {
        try
        {
            var floodInfo = await _femaClient.GetFloodZoneByCoordinatesAsync(property.Latitude!.Value, property.Longitude!.Value);
            property.FloodZone = floodInfo.FloodZone;
            property.FloodZoneDescription = floodInfo.ZoneDescription;
            property.IsInFloodZone = floodInfo.IsHighRisk || floodInfo.FloodZone != "X";
            property.IsInHighRiskFloodZone = floodInfo.IsHighRisk;
            property.RequiresFloodInsurance = floodInfo.RequiresInsurance;
            property.FemaMapPanel = floodInfo.MapPanel;
            property.FloodZoneLastUpdated = DateTime.UtcNow;
            await _context.SaveChangesAsync();
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to enrich property {Id} with flood data", property.Id);
        }
    }

    private static IQueryable<Property> ApplyFilters(IQueryable<Property> query, SearchRequest request)
    {
        if (!string.IsNullOrWhiteSpace(request.City))
            query = query.Where(p => p.City.ToLower() == request.City.ToLower());
        if (!string.IsNullOrWhiteSpace(request.County))
            query = query.Where(p => p.County.ToLower() == request.County.ToLower());
        if (!string.IsNullOrWhiteSpace(request.ZipCode))
            query = query.Where(p => p.ZipCode == request.ZipCode);
        if (request.MinRoofAge.HasValue)
            query = query.Where(p => p.RoofAge >= request.MinRoofAge.Value);
        if (request.MaxRoofAge.HasValue)
            query = query.Where(p => p.RoofAge <= request.MaxRoofAge.Value);
        if (!string.IsNullOrWhiteSpace(request.PropertyType))
            query = query.Where(p => p.PropertyType == request.PropertyType);
        if (request.InFloodZone == true)
            query = query.Where(p => p.IsInFloodZone);
        if (request.HighFireRisk == true)
            query = query.Where(p => p.WildfireRiskScore > 50);
        return query;
    }

    private static SearchResult MapToSearchResult(Property p)
    {
        var lastRoofPermit = p.Permits.Where(permit => permit.IsRoofingPermit)
            .OrderByDescending(permit => permit.IssuedDate).FirstOrDefault();

        return new SearchResult
        {
            PropertyId = p.Id, Address = p.Address, City = p.City, ZipCode = p.ZipCode,
            County = p.County, OwnerName = p.Owner?.FullName ?? "Unknown",
            PhoneNumber = p.Owner?.PhoneNumber, Email = p.Owner?.Email, OwnerAge = p.Owner?.Age,
            PropertyType = p.PropertyType, YearBuilt = p.YearBuilt, RoofType = p.RoofType,
            RoofAge = p.RoofAge, LastRoofPermitDate = lastRoofPermit?.IssuedDate ?? p.LastRoofPermitDate,
            LastPermitStatus = lastRoofPermit?.Status, TotalPermits = p.Permits.Count,
            EstimatedValue = p.EstimatedValue, FloodZone = p.FloodZone,
            IsInHighRiskFloodZone = p.IsInHighRiskFloodZone, RequiresFloodInsurance = p.RequiresFloodInsurance,
            FireRiskLevel = p.FireRiskLevel, WildfireRiskScore = p.WildfireRiskScore,
            TotalClaims = p.InsuranceClaims.Count,
            HasOpenClaim = p.InsuranceClaims.Any(c => c.Status == "Open" || c.Status == "In Progress"),
            OverallRiskScore = p.OverallRiskScore, LeadPriority = p.LeadPriority,
            DataSource = p.DataSource,
        };
    }
}
