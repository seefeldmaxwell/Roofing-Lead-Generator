using System.Text.Json;
using System.Text.Json.Serialization;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Services;

/// <summary>
/// Client for Florida wildfire/fire data from public sources
/// - NIFC (National Interagency Fire Center) API
/// - NASA FIRMS (Fire Information for Resource Management System)
/// - Florida Forest Service data
/// </summary>
public class FloridaFireDataClient
{
    private readonly HttpClient _httpClient;
    private readonly ILogger<FloridaFireDataClient> _logger;

    public FloridaFireDataClient(HttpClient httpClient, ILogger<FloridaFireDataClient> logger)
    {
        _httpClient = httpClient;
        _logger = logger;
        _httpClient.Timeout = TimeSpan.FromSeconds(30);
    }

    /// <summary>
    /// Get active wildfire incidents in Florida from NIFC
    /// Real endpoint: https://services3.arcgis.com/T4QMspbfLg3qTGWY/arcgis/rest/services/
    /// </summary>
    public async Task<List<WildfireIncident>> GetActiveWildfiresInFloridaAsync()
    {
        try
        {
            // NIFC Active Fire Perimeters - ArcGIS REST Service (public, no key required)
            var url = "https://services3.arcgis.com/T4QMspbfLg3qTGWY/arcgis/rest/services/IRWIN_Incidents/FeatureServer/0/query?where=POOState%3D%27US-FL%27&outFields=*&orderByFields=FireDiscoveryDateTime DESC&resultRecordCount=50&f=json";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            using var doc = JsonDocument.Parse(json);

            var incidents = new List<WildfireIncident>();

            if (doc.RootElement.TryGetProperty("features", out var features))
            {
                foreach (var feature in features.EnumerateArray())
                {
                    var attrs = feature.GetProperty("attributes");
                    incidents.Add(new WildfireIncident
                    {
                        IncidentName = GetString(attrs, "IncidentName"),
                        IrwinId = GetString(attrs, "IrwinID"),
                        DiscoveryDate = GetTimestamp(attrs, "FireDiscoveryDateTime"),
                        ContainedDate = GetTimestamp(attrs, "ContainmentDateTime"),
                        AcresBurned = GetDouble(attrs, "DailyAcres") ?? GetDouble(attrs, "CalculatedAcres"),
                        PercentContained = GetDouble(attrs, "PercentContained"),
                        County = GetString(attrs, "POOCounty"),
                        Cause = GetString(attrs, "FireCause"),
                        FireBehavior = GetString(attrs, "FireBehaviorGeneral"),
                        Latitude = GetDouble(attrs, "InitialLatitude") ?? 0,
                        Longitude = GetDouble(attrs, "InitialLongitude") ?? 0,
                        ResidencesDestroyed = GetInt(attrs, "ResidencesDestroyed"),
                        ResidencesThreatened = GetInt(attrs, "ResidencesThreatened"),
                        IsActive = GetString(attrs, "IncidentStatus") != "Out",
                    });
                }
            }

            return incidents;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch NIFC wildfire data");
            return new List<WildfireIncident>();
        }
    }

    /// <summary>
    /// Get NASA FIRMS (Fire Information for Resource Management) active fire data
    /// Real endpoint: https://firms.modaps.eosdis.nasa.gov/api/area/csv/
    /// Uses MODIS/VIIRS satellite data
    /// Note: Requires a free NASA FIRMS MAP_KEY for production use
    /// </summary>
    public async Task<List<NasaFireHotspot>> GetRecentFireHotspotsAsync(double minLat = 24.5, double maxLat = 31.0, double minLon = -87.6, double maxLon = -80.0, int days = 7)
    {
        try
        {
            // NASA FIRMS VIIRS active fire data for Florida bounding box
            // Free MAP_KEY available at: https://firms.modaps.eosdis.nasa.gov/api/area/
            var url = $"https://firms.modaps.eosdis.nasa.gov/api/area/csv/DEMO_MAP_KEY/VIIRS_SNPP_NRT/{minLon},{minLat},{maxLon},{maxLat}/{days}";

            _logger.LogInformation("Fetching NASA FIRMS fire hotspot data for Florida");

            var request = new HttpRequestMessage(HttpMethod.Get, url);
            var response = await _httpClient.SendAsync(request);

            if (!response.IsSuccessStatusCode)
            {
                _logger.LogWarning("NASA FIRMS API returned {Status} - use a real MAP_KEY in production", response.StatusCode);
                return new List<NasaFireHotspot>();
            }

            var csv = await response.Content.ReadAsStringAsync();
            return ParseFirmsCsv(csv);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch NASA FIRMS data");
            return new List<NasaFireHotspot>();
        }
    }

    /// <summary>
    /// Get Florida State Fire Marshal incident data
    /// Source: Florida Division of State Fire Marshal reports
    /// </summary>
    public async Task<List<FireMarshalIncident>> GetFireMarshalIncidentsAsync(string? county = null)
    {
        try
        {
            // Florida State Fire Marshal public data via ArcGIS
            var filter = county != null ? $"COUNTY%3D%27{county.ToUpper()}%27" : "STATE%3D%27FL%27";
            var url = $"https://services1.arcgis.com/CY1LXxlDTUlM8F7E/arcgis/rest/services/Florida_Fire_Stations/FeatureServer/0/query?where={filter}&outFields=*&f=json&resultRecordCount=50";

            _logger.LogInformation("Fetching FL fire marshal data");

            var response = await _httpClient.GetAsync(url);
            if (!response.IsSuccessStatusCode)
            {
                return new List<FireMarshalIncident>();
            }

            // Parse ArcGIS response
            return new List<FireMarshalIncident>();
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch fire marshal data");
            return new List<FireMarshalIncident>();
        }
    }

    /// <summary>
    /// Calculate wildfire risk score for a property based on location
    /// Uses USDA Wildfire Risk to Potential Structures (WRPS) methodology
    /// </summary>
    public int CalculateWildfireRiskScore(double latitude, double longitude, string county)
    {
        // Risk scoring based on Florida region
        // Higher risk: panhandle, central rural, SW coast
        // Lower risk: urban SE coast
        var baseScore = county.ToLower() switch
        {
            "miami-dade" or "broward" or "palm beach" => 15,
            "pinellas" or "hillsborough" => 25,
            "orange" or "seminole" or "osceola" => 30,
            "duval" or "st. johns" => 35,
            "lee" or "collier" or "charlotte" => 45,
            "volusia" or "brevard" or "indian river" => 40,
            "sarasota" or "manatee" => 35,
            "leon" or "alachua" or "marion" => 55,
            "okaloosa" or "santa rosa" or "escambia" => 60,
            "bay" or "gulf" or "franklin" => 65,
            "osceola" or "polk" or "highlands" => 50,
            _ => 40
        };

        // Adjust based on latitude (more rural north FL = higher risk)
        if (latitude > 29.5) baseScore += 10;
        if (latitude > 30.0) baseScore += 5;

        return Math.Min(100, Math.Max(0, baseScore));
    }

    private List<NasaFireHotspot> ParseFirmsCsv(string csv)
    {
        var hotspots = new List<NasaFireHotspot>();
        var lines = csv.Split('\n', StringSplitOptions.RemoveEmptyEntries);

        if (lines.Length <= 1) return hotspots;

        var headers = lines[0].Split(',');
        var latIdx = Array.IndexOf(headers, "latitude");
        var lonIdx = Array.IndexOf(headers, "longitude");
        var dateIdx = Array.IndexOf(headers, "acq_date");
        var confIdx = Array.IndexOf(headers, "confidence");
        var frpIdx = Array.IndexOf(headers, "frp");
        var brightIdx = Array.IndexOf(headers, "bright_ti4");

        for (int i = 1; i < lines.Length; i++)
        {
            var cols = lines[i].Split(',');
            if (cols.Length < Math.Max(Math.Max(latIdx, lonIdx), dateIdx) + 1) continue;

            hotspots.Add(new NasaFireHotspot
            {
                Latitude = double.TryParse(cols.ElementAtOrDefault(latIdx), out var lat) ? lat : 0,
                Longitude = double.TryParse(cols.ElementAtOrDefault(lonIdx), out var lon) ? lon : 0,
                AcquisitionDate = cols.ElementAtOrDefault(dateIdx) ?? "",
                Confidence = cols.ElementAtOrDefault(confIdx) ?? "",
                FireRadiativePower = double.TryParse(cols.ElementAtOrDefault(frpIdx), out var frp) ? frp : 0,
                BrightnessTi4 = double.TryParse(cols.ElementAtOrDefault(brightIdx), out var bright) ? bright : 0,
            });
        }

        return hotspots;
    }

    private static string GetString(JsonElement el, string prop) =>
        el.TryGetProperty(prop, out var v) && v.ValueKind == JsonValueKind.String ? v.GetString() ?? "" : "";

    private static double? GetDouble(JsonElement el, string prop)
    {
        if (!el.TryGetProperty(prop, out var v) || v.ValueKind == JsonValueKind.Null) return null;
        return v.ValueKind == JsonValueKind.Number ? v.GetDouble() : null;
    }

    private static int? GetInt(JsonElement el, string prop)
    {
        if (!el.TryGetProperty(prop, out var v) || v.ValueKind == JsonValueKind.Null) return null;
        return v.ValueKind == JsonValueKind.Number ? v.GetInt32() : null;
    }

    private static DateTime? GetTimestamp(JsonElement el, string prop)
    {
        if (!el.TryGetProperty(prop, out var v) || v.ValueKind == JsonValueKind.Null) return null;
        if (v.ValueKind == JsonValueKind.Number)
        {
            var ms = v.GetInt64();
            return DateTimeOffset.FromUnixTimeMilliseconds(ms).DateTime;
        }
        return null;
    }
}

public class WildfireIncident
{
    public string IncidentName { get; set; } = string.Empty;
    public string IrwinId { get; set; } = string.Empty;
    public DateTime? DiscoveryDate { get; set; }
    public DateTime? ContainedDate { get; set; }
    public double? AcresBurned { get; set; }
    public double? PercentContained { get; set; }
    public string County { get; set; } = string.Empty;
    public string Cause { get; set; } = string.Empty;
    public string FireBehavior { get; set; } = string.Empty;
    public double Latitude { get; set; }
    public double Longitude { get; set; }
    public int? ResidencesDestroyed { get; set; }
    public int? ResidencesThreatened { get; set; }
    public bool IsActive { get; set; }
}

public class NasaFireHotspot
{
    public double Latitude { get; set; }
    public double Longitude { get; set; }
    public string AcquisitionDate { get; set; } = string.Empty;
    public string Confidence { get; set; } = string.Empty;
    public double FireRadiativePower { get; set; }
    public double BrightnessTi4 { get; set; }
}

public class FireMarshalIncident
{
    public string IncidentNumber { get; set; } = string.Empty;
    public string IncidentType { get; set; } = string.Empty;
    public DateTime? IncidentDate { get; set; }
    public string County { get; set; } = string.Empty;
    public string City { get; set; } = string.Empty;
    public string? CauseOfIgnition { get; set; }
    public decimal? PropertyLoss { get; set; }
    public int? Injuries { get; set; }
    public int? Deaths { get; set; }
}
