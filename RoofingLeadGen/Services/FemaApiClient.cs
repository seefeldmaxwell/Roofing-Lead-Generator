using System.Text.Json;
using System.Text.Json.Serialization;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Services;

/// <summary>
/// Client for FEMA public APIs - real government data
/// https://www.fema.gov/about/openfema/api
/// </summary>
public class FemaApiClient
{
    private readonly HttpClient _httpClient;
    private readonly ILogger<FemaApiClient> _logger;
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull
    };

    // Real FEMA OpenFEMA API base URL
    private const string BaseUrl = "https://www.fema.gov/api/open/v2";

    public FemaApiClient(HttpClient httpClient, ILogger<FemaApiClient> logger)
    {
        _httpClient = httpClient;
        _logger = logger;
        _httpClient.BaseAddress = new Uri(BaseUrl);
        _httpClient.Timeout = TimeSpan.FromSeconds(30);
    }

    /// <summary>
    /// Get FEMA disaster declarations for Florida
    /// Real endpoint: https://www.fema.gov/api/open/v2/DisasterDeclarationsSummaries
    /// </summary>
    public async Task<List<FemaDisaster>> GetFloridaDisastersAsync(int limit = 100)
    {
        try
        {
            var url = $"/DisasterDeclarationsSummaries?$filter=state eq 'Florida'&$orderby=declarationDate desc&$top={limit}";
            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<FemaApiResponse<FemaDisasterDto>>(json, JsonOptions);

            return result?.DisasterDeclarationsSummaries?.Select(d => new FemaDisaster
            {
                DisasterNumber = d.DisasterNumber?.ToString() ?? "",
                Title = d.DeclarationTitle ?? "",
                DisasterType = d.DeclarationType ?? "",
                DeclarationDate = ParseDate(d.DeclarationDate),
                IncidentBeginDate = ParseDate(d.IncidentBeginDate),
                IncidentEndDate = ParseDate(d.IncidentEndDate),
                CloseoutDate = ParseDate(d.DisasterCloseoutDate),
                State = "FL",
                DeclaredCounty = d.DesignatedArea,
                IncidentType = d.IncidentType,
                IndividualAssistance = d.IhProgramDeclared ?? false,
                PublicAssistance = d.PaProgramDeclared ?? false,
                HazardMitigation = d.HmProgramDeclared ?? false,
            }).ToList() ?? new List<FemaDisaster>();
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch FEMA disaster data");
            return new List<FemaDisaster>();
        }
    }

    /// <summary>
    /// Get FEMA disaster declarations by county
    /// </summary>
    public async Task<List<FemaDisaster>> GetDisastersByCountyAsync(string county, int limit = 50)
    {
        try
        {
            var url = $"/DisasterDeclarationsSummaries?$filter=state eq 'Florida' and contains(designatedArea,'{county}')&$orderby=declarationDate desc&$top={limit}";
            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<FemaApiResponse<FemaDisasterDto>>(json, JsonOptions);

            return result?.DisasterDeclarationsSummaries?.Select(d => new FemaDisaster
            {
                DisasterNumber = d.DisasterNumber?.ToString() ?? "",
                Title = d.DeclarationTitle ?? "",
                DisasterType = d.DeclarationType ?? "",
                DeclarationDate = ParseDate(d.DeclarationDate),
                IncidentBeginDate = ParseDate(d.IncidentBeginDate),
                IncidentEndDate = ParseDate(d.IncidentEndDate),
                State = "FL",
                DeclaredCounty = d.DesignatedArea,
                IncidentType = d.IncidentType,
                IndividualAssistance = d.IhProgramDeclared ?? false,
                PublicAssistance = d.PaProgramDeclared ?? false,
                HazardMitigation = d.HmProgramDeclared ?? false,
            }).ToList() ?? new List<FemaDisaster>();
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch FEMA data for county {County}", county);
            return new List<FemaDisaster>();
        }
    }

    /// <summary>
    /// Get NFIP (National Flood Insurance Program) claims data by zip code
    /// Real endpoint: https://www.fema.gov/api/open/v2/FimaNfipClaims
    /// </summary>
    public async Task<FloodClaimsSummary> GetFloodClaimsByZipAsync(string zipCode)
    {
        try
        {
            var url = $"/FimaNfipClaims?$filter=reportedZipCode eq '{zipCode}'&$top=200";
            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            var result = JsonSerializer.Deserialize<FemaClaimsResponse>(json, JsonOptions);

            var claims = result?.FimaNfipClaims ?? new List<NfipClaimDto>();

            return new FloodClaimsSummary
            {
                ZipCode = zipCode,
                TotalClaims = claims.Count,
                TotalPaid = claims.Sum(c => c.AmountPaidOnBuildingClaim ?? 0) + claims.Sum(c => c.AmountPaidOnContentsClaim ?? 0),
                AverageClaimAmount = claims.Count > 0 ? (claims.Sum(c => c.AmountPaidOnBuildingClaim ?? 0) + claims.Sum(c => c.AmountPaidOnContentsClaim ?? 0)) / claims.Count : 0,
                MostRecentClaim = claims.Max(c => ParseDate(c.DateOfLoss)),
                RepeatLossCount = claims.Count(c => c.ReportedCity != null),
                FloodZones = claims.Select(c => c.FloodZone).Where(z => z != null).Distinct().Cast<string>().ToList(),
            };
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch flood claims for zip {Zip}", zipCode);
            return new FloodClaimsSummary { ZipCode = zipCode };
        }
    }

    /// <summary>
    /// Get NFIP policies in force by state/county
    /// Real endpoint: https://www.fema.gov/api/open/v2/FimaNfipPolicies
    /// </summary>
    public async Task<int> GetActivePolicyCountByCountyAsync(string county)
    {
        try
        {
            var url = $"/FimaNfipPolicies?$filter=propertyState eq 'FL' and contains(countyCode,'{county}')&$inlinecount=allpages&$top=1";
            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            using var doc = JsonDocument.Parse(json);
            if (doc.RootElement.TryGetProperty("metadata", out var meta) &&
                meta.TryGetProperty("count", out var count))
            {
                return count.GetInt32();
            }
            return 0;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch NFIP policy count for {County}", county);
            return 0;
        }
    }

    /// <summary>
    /// Determine flood zone from lat/long via FEMA NFHL service
    /// Real endpoint: https://hazards.fema.gov/gis/nfhl/rest/services
    /// </summary>
    public async Task<FloodZoneInfo> GetFloodZoneByCoordinatesAsync(double latitude, double longitude)
    {
        try
        {
            var url = $"https://hazards.fema.gov/gis/nfhl/rest/services/public/NFHL/MapServer/28/query?where=1%3D1&geometry={longitude}%2C{latitude}&geometryType=esriGeometryPoint&spatialRel=esriSpatialRelIntersects&outFields=FLD_ZONE%2CZONE_SUBTY%2CSFHA_TF%2CDFIRM_ID&returnGeometry=false&f=json";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            using var doc = JsonDocument.Parse(json);

            if (doc.RootElement.TryGetProperty("features", out var features) && features.GetArrayLength() > 0)
            {
                var attrs = features[0].GetProperty("attributes");
                var zone = attrs.TryGetProperty("FLD_ZONE", out var z) ? z.GetString() ?? "" : "";
                var subType = attrs.TryGetProperty("ZONE_SUBTY", out var s) ? s.GetString() : null;
                var sfha = attrs.TryGetProperty("SFHA_TF", out var sf) ? sf.GetString() : null;

                return new FloodZoneInfo
                {
                    FloodZone = zone,
                    ZoneDescription = GetFloodZoneDescription(zone),
                    IsHighRisk = sfha == "T" || zone == "A" || zone == "AE" || zone == "AH" || zone == "AO" || zone == "V" || zone == "VE",
                    RequiresInsurance = sfha == "T",
                    MapPanel = attrs.TryGetProperty("DFIRM_ID", out var d) ? d.GetString() : null,
                };
            }

            return new FloodZoneInfo { FloodZone = "X", ZoneDescription = "Minimal flood risk", IsHighRisk = false };
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch flood zone for {Lat},{Lon}", latitude, longitude);
            return new FloodZoneInfo { FloodZone = "Unknown", ZoneDescription = "Could not determine" };
        }
    }

    private static string GetFloodZoneDescription(string zone) => zone switch
    {
        "A" => "High Risk - 1% annual chance flood (100-year)",
        "AE" => "High Risk - 1% annual chance with base flood elevations",
        "AH" => "High Risk - 1% annual chance shallow flooding (1-3 ft)",
        "AO" => "High Risk - 1% annual chance sheet flow flooding",
        "AR" => "High Risk - Temporary increased risk due to levee restoration",
        "A99" => "High Risk - Federal flood protection system under construction",
        "V" => "High Risk - Coastal flood with velocity (wave action)",
        "VE" => "High Risk - Coastal flood with velocity and base elevations",
        "X" => "Minimal Risk - Outside 500-year floodplain",
        "B" or "X500" => "Moderate Risk - 500-year floodplain (0.2% annual chance)",
        "C" => "Minimal Risk - Outside floodplain",
        "D" => "Undetermined Risk - No analysis performed",
        _ => $"Zone {zone}"
    };

    private static DateTime? ParseDate(string? dateStr)
    {
        if (string.IsNullOrEmpty(dateStr)) return null;
        return DateTime.TryParse(dateStr, out var d) ? d : null;
    }

    // DTO classes for FEMA API JSON responses
    private class FemaApiResponse<T>
    {
        public List<T>? DisasterDeclarationsSummaries { get; set; }
    }

    private class FemaClaimsResponse
    {
        public List<NfipClaimDto>? FimaNfipClaims { get; set; }
    }

    private class FemaDisasterDto
    {
        public int? DisasterNumber { get; set; }
        public string? DeclarationTitle { get; set; }
        public string? DeclarationType { get; set; }
        public string? DeclarationDate { get; set; }
        public string? IncidentBeginDate { get; set; }
        public string? IncidentEndDate { get; set; }
        public string? DisasterCloseoutDate { get; set; }
        public string? DesignatedArea { get; set; }
        public string? IncidentType { get; set; }
        public bool? IhProgramDeclared { get; set; }
        public bool? PaProgramDeclared { get; set; }
        public bool? HmProgramDeclared { get; set; }
    }

    private class NfipClaimDto
    {
        public double? AmountPaidOnBuildingClaim { get; set; }
        public double? AmountPaidOnContentsClaim { get; set; }
        public string? DateOfLoss { get; set; }
        public string? FloodZone { get; set; }
        public string? ReportedCity { get; set; }
        public string? ReportedZipCode { get; set; }
    }
}

public class FloodClaimsSummary
{
    public string ZipCode { get; set; } = string.Empty;
    public int TotalClaims { get; set; }
    public double TotalPaid { get; set; }
    public double AverageClaimAmount { get; set; }
    public DateTime? MostRecentClaim { get; set; }
    public int RepeatLossCount { get; set; }
    public List<string> FloodZones { get; set; } = new();
}
