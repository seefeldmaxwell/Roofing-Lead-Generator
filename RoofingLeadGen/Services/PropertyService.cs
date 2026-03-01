using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Data;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Services;

public class PropertyService
{
    private readonly AppDbContext _context;

    public PropertyService(AppDbContext context)
    {
        _context = context;
    }

    public async Task<PaginatedResult<SearchResult>> SearchByAddressAsync(SearchRequest request)
    {
        var query = _context.Properties
            .Include(p => p.Owner)
            .Include(p => p.Permits)
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
        var items = await query
            .OrderBy(p => p.Address)
            .Skip((request.Page - 1) * request.PageSize)
            .Take(request.PageSize)
            .Select(p => MapToSearchResult(p))
            .ToListAsync();

        return new PaginatedResult<SearchResult>
        {
            Items = items,
            TotalCount = totalCount,
            Page = request.Page,
            PageSize = request.PageSize
        };
    }

    public async Task<PaginatedResult<SearchResult>> SearchByOwnerAsync(SearchRequest request)
    {
        var query = _context.Properties
            .Include(p => p.Owner)
            .Include(p => p.Permits)
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
        var items = await query
            .OrderBy(p => p.Owner!.LastName)
            .ThenBy(p => p.Owner!.FirstName)
            .Skip((request.Page - 1) * request.PageSize)
            .Take(request.PageSize)
            .Select(p => MapToSearchResult(p))
            .ToListAsync();

        return new PaginatedResult<SearchResult>
        {
            Items = items,
            TotalCount = totalCount,
            Page = request.Page,
            PageSize = request.PageSize
        };
    }

    public async Task<Property?> GetPropertyByIdAsync(int id)
    {
        return await _context.Properties
            .Include(p => p.Owner)
            .Include(p => p.Permits.OrderByDescending(permit => permit.IssuedDate))
            .FirstOrDefaultAsync(p => p.Id == id);
    }

    public async Task<DashboardStats> GetDashboardStatsAsync()
    {
        var properties = await _context.Properties
            .Include(p => p.Permits)
            .ToListAsync();

        var stats = new DashboardStats
        {
            TotalProperties = properties.Count,
            TotalOwners = await _context.Owners.CountAsync(),
            TotalPermits = await _context.Permits.CountAsync(),
            RoofsOver15Years = properties.Count(p => p.RoofAge.HasValue && p.RoofAge > 15),
            RecentPermits30Days = await _context.Permits
                .CountAsync(p => p.IssuedDate.HasValue && p.IssuedDate > DateTime.Now.AddDays(-30)),
        };

        stats.CountyBreakdowns = properties
            .GroupBy(p => p.County)
            .Select(g => new CountyBreakdown
            {
                County = g.Key,
                PropertyCount = g.Count(),
                OldRoofCount = g.Count(p => p.RoofAge.HasValue && p.RoofAge > 15)
            })
            .OrderByDescending(c => c.PropertyCount)
            .ToList();

        stats.RoofAgeDistributions = new List<RoofAgeDistribution>
        {
            new() { Range = "0-5 years", Count = properties.Count(p => p.RoofAge is >= 0 and <= 5) },
            new() { Range = "6-10 years", Count = properties.Count(p => p.RoofAge is >= 6 and <= 10) },
            new() { Range = "11-15 years", Count = properties.Count(p => p.RoofAge is >= 11 and <= 15) },
            new() { Range = "16-20 years", Count = properties.Count(p => p.RoofAge is >= 16 and <= 20) },
            new() { Range = "20+ years", Count = properties.Count(p => p.RoofAge > 20) },
        };

        return stats;
    }

    public async Task<List<string>> GetCountiesAsync()
    {
        return await _context.Properties
            .Select(p => p.County)
            .Where(c => !string.IsNullOrEmpty(c))
            .Distinct()
            .OrderBy(c => c)
            .ToListAsync();
    }

    public async Task<List<string>> GetCitiesAsync(string? county = null)
    {
        var query = _context.Properties.AsQueryable();
        if (!string.IsNullOrEmpty(county))
            query = query.Where(p => p.County == county);

        return await query
            .Select(p => p.City)
            .Where(c => !string.IsNullOrEmpty(c))
            .Distinct()
            .OrderBy(c => c)
            .ToListAsync();
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

        return query;
    }

    private static SearchResult MapToSearchResult(Property p)
    {
        var lastRoofPermit = p.Permits
            .Where(permit => permit.IsRoofingPermit)
            .OrderByDescending(permit => permit.IssuedDate)
            .FirstOrDefault();

        return new SearchResult
        {
            PropertyId = p.Id,
            Address = p.Address,
            City = p.City,
            ZipCode = p.ZipCode,
            County = p.County,
            OwnerName = p.Owner?.FullName ?? "Unknown",
            PhoneNumber = p.Owner?.PhoneNumber,
            Email = p.Owner?.Email,
            OwnerAge = p.Owner?.Age,
            PropertyType = p.PropertyType,
            YearBuilt = p.YearBuilt,
            RoofType = p.RoofType,
            RoofAge = p.RoofAge,
            LastRoofPermitDate = lastRoofPermit?.IssuedDate ?? p.LastRoofPermitDate,
            LastPermitStatus = lastRoofPermit?.Status,
            TotalPermits = p.Permits.Count,
            EstimatedValue = p.EstimatedValue,
            ImageUrl = p.ImageUrl
        };
    }
}
