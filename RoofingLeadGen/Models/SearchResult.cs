namespace RoofingLeadGen.Models;

public class SearchResult
{
    public int PropertyId { get; set; }
    public string Address { get; set; } = string.Empty;
    public string City { get; set; } = string.Empty;
    public string ZipCode { get; set; } = string.Empty;
    public string County { get; set; } = string.Empty;
    public string OwnerName { get; set; } = string.Empty;
    public string? PhoneNumber { get; set; }
    public string? Email { get; set; }
    public int? OwnerAge { get; set; }
    public string PropertyType { get; set; } = string.Empty;
    public int? YearBuilt { get; set; }
    public string RoofType { get; set; } = string.Empty;
    public int? RoofAge { get; set; }
    public DateTime? LastRoofPermitDate { get; set; }
    public string? LastPermitStatus { get; set; }
    public int TotalPermits { get; set; }
    public decimal? EstimatedValue { get; set; }
    public string? ImageUrl { get; set; }

    // Flood data
    public string? FloodZone { get; set; }
    public bool IsInHighRiskFloodZone { get; set; }
    public bool RequiresFloodInsurance { get; set; }

    // Fire risk
    public string? FireRiskLevel { get; set; }
    public int? WildfireRiskScore { get; set; }

    // Insurance
    public int TotalClaims { get; set; }
    public bool HasOpenClaim { get; set; }

    // Risk / Lead scoring
    public int? OverallRiskScore { get; set; }
    public string? LeadPriority { get; set; }

    // Data source
    public string? DataSource { get; set; }

    // Work categories found in permits
    public List<string> WorkCategories { get; set; } = new();
}

public class SearchRequest
{
    public string? Query { get; set; }
    public SearchMode Mode { get; set; } = SearchMode.Address;
    public string? City { get; set; }
    public string? County { get; set; }
    public string? ZipCode { get; set; }
    public int? MinRoofAge { get; set; }
    public int? MaxRoofAge { get; set; }
    public string? PropertyType { get; set; }
    public bool? InFloodZone { get; set; }
    public bool? HighFireRisk { get; set; }
    public string? LeadPriority { get; set; }
    public int Page { get; set; } = 1;
    public int PageSize { get; set; } = 25;
}

public enum SearchMode
{
    Address,
    Owner
}

public class DashboardStats
{
    public int TotalProperties { get; set; }
    public int TotalOwners { get; set; }
    public int TotalPermits { get; set; }
    public int RoofsOver15Years { get; set; }
    public int RecentPermits30Days { get; set; }

    // Flood/fire stats
    public int PropertiesInFloodZone { get; set; }
    public int PropertiesHighRiskFlood { get; set; }
    public int PropertiesHighFireRisk { get; set; }
    public int TotalInsuranceClaims { get; set; }
    public int OpenClaims { get; set; }
    public int FemaDisasters { get; set; }

    public List<CountyBreakdown> CountyBreakdowns { get; set; } = new();
    public List<RoofAgeDistribution> RoofAgeDistributions { get; set; } = new();
}

public class CountyBreakdown
{
    public string County { get; set; } = string.Empty;
    public int PropertyCount { get; set; }
    public int OldRoofCount { get; set; }
    public int FloodZoneCount { get; set; }
    public int HighFireRiskCount { get; set; }
}

public class RoofAgeDistribution
{
    public string Range { get; set; } = string.Empty;
    public int Count { get; set; }
}

public class PaginatedResult<T>
{
    public List<T> Items { get; set; } = new();
    public int TotalCount { get; set; }
    public int Page { get; set; }
    public int PageSize { get; set; }
    public int TotalPages => (int)Math.Ceiling(TotalCount / (double)PageSize);
    public bool HasPreviousPage => Page > 1;
    public bool HasNextPage => Page < TotalPages;
}
