using System.Text.Json;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Services;

/// <summary>
/// Client for real Florida public property records from county/state APIs.
///
/// Primary data sources (all free, public, no auth required):
/// 1. FDOT Statewide Parcels FeatureServer - all 67 counties
///    https://gis.fdot.gov/arcgis/rest/services/Parcels/FeatureServer
/// 2. Florida Statewide Cadastral (FL GIO / Dept of Revenue)
///    https://services9.arcgis.com/Gh9awoU677aKree0/arcgis/rest/services/Florida_Statewide_Cadastral/FeatureServer
/// 3. County-specific ArcGIS endpoints for detailed data
/// 4. US Census Bureau Geocoder for address resolution
/// 5. Miami-Dade Building Permits Open Data Hub
/// </summary>
public class FloridaPropertyDataClient
{
    private readonly HttpClient _httpClient;
    private readonly ILogger<FloridaPropertyDataClient> _logger;

    // FDOT county layer IDs (real data - maps county name to FDOT FeatureServer layer number)
    private static readonly Dictionary<string, int> FdotCountyLayers = new(StringComparer.OrdinalIgnoreCase)
    {
        ["ALACHUA"] = 0, ["BAKER"] = 1, ["BAY"] = 2, ["BRADFORD"] = 3, ["BREVARD"] = 4,
        ["BROWARD"] = 6, ["CALHOUN"] = 7, ["CHARLOTTE"] = 8, ["CITRUS"] = 9, ["CLAY"] = 10,
        ["COLLIER"] = 11, ["COLUMBIA"] = 12, ["MIAMI-DADE"] = 13, ["DESOTO"] = 14, ["DIXIE"] = 15,
        ["DUVAL"] = 16, ["ESCAMBIA"] = 17, ["FLAGLER"] = 18, ["FRANKLIN"] = 19, ["GADSDEN"] = 20,
        ["GILCHRIST"] = 21, ["GLADES"] = 22, ["GULF"] = 23, ["HAMILTON"] = 24, ["HARDEE"] = 25,
        ["HENDRY"] = 26, ["HERNANDO"] = 27, ["HIGHLANDS"] = 28, ["HILLSBOROUGH"] = 29,
        ["HOLMES"] = 30, ["INDIAN RIVER"] = 31, ["JACKSON"] = 32, ["JEFFERSON"] = 33,
        ["LAFAYETTE"] = 34, ["LAKE"] = 35, ["LEE"] = 36, ["LEON"] = 37, ["LEVY"] = 38,
        ["LIBERTY"] = 39, ["MADISON"] = 40, ["MANATEE"] = 41, ["MARION"] = 42, ["MARTIN"] = 43,
        ["MONROE"] = 44, ["NASSAU"] = 45, ["OKALOOSA"] = 46, ["OKEECHOBEE"] = 47,
        ["ORANGE"] = 48, ["OSCEOLA"] = 49, ["PALM BEACH"] = 50, ["PASCO"] = 51,
        ["PINELLAS"] = 52, ["POLK"] = 53, ["PUTNAM"] = 54, ["SANTA ROSA"] = 55,
        ["SARASOTA"] = 56, ["SEMINOLE"] = 57, ["ST. JOHNS"] = 58, ["ST. LUCIE"] = 59,
        ["SUMTER"] = 60, ["SUWANNEE"] = 61, ["TAYLOR"] = 62, ["UNION"] = 63,
        ["VOLUSIA"] = 64, ["WAKULLA"] = 65, ["WALTON"] = 66, ["WASHINGTON"] = 67,
    };

    // Florida DOR County codes
    private static readonly Dictionary<string, int> DorCountyCodes = new(StringComparer.OrdinalIgnoreCase)
    {
        ["ALACHUA"] = 1, ["BAKER"] = 2, ["BAY"] = 3, ["BRADFORD"] = 4, ["BREVARD"] = 5,
        ["BROWARD"] = 6, ["CALHOUN"] = 7, ["CHARLOTTE"] = 8, ["CITRUS"] = 9, ["CLAY"] = 10,
        ["COLLIER"] = 11, ["COLUMBIA"] = 12, ["MIAMI-DADE"] = 13, ["DESOTO"] = 14, ["DIXIE"] = 15,
        ["DUVAL"] = 16, ["ESCAMBIA"] = 17, ["FLAGLER"] = 18, ["FRANKLIN"] = 19, ["GADSDEN"] = 20,
        ["GILCHRIST"] = 21, ["GLADES"] = 22, ["GULF"] = 23, ["HAMILTON"] = 24, ["HARDEE"] = 25,
        ["HENDRY"] = 26, ["HERNANDO"] = 27, ["HIGHLANDS"] = 28, ["HILLSBOROUGH"] = 29,
        ["HOLMES"] = 30, ["INDIAN RIVER"] = 31, ["JACKSON"] = 32, ["JEFFERSON"] = 33,
        ["LAFAYETTE"] = 34, ["LAKE"] = 35, ["LEE"] = 36, ["LEON"] = 37, ["LEVY"] = 38,
        ["LIBERTY"] = 39, ["MADISON"] = 40, ["MANATEE"] = 41, ["MARION"] = 42, ["MARTIN"] = 43,
        ["MONROE"] = 44, ["NASSAU"] = 45, ["OKALOOSA"] = 46, ["OKEECHOBEE"] = 47,
        ["ORANGE"] = 48, ["OSCEOLA"] = 49, ["PALM BEACH"] = 50, ["PASCO"] = 51,
        ["PINELLAS"] = 52, ["POLK"] = 53, ["PUTNAM"] = 54, ["SANTA ROSA"] = 55,
        ["SARASOTA"] = 56, ["SEMINOLE"] = 57, ["ST. JOHNS"] = 58, ["ST. LUCIE"] = 59,
        ["SUMTER"] = 60, ["SUWANNEE"] = 61, ["TAYLOR"] = 62, ["UNION"] = 63,
        ["VOLUSIA"] = 64, ["WAKULLA"] = 65, ["WALTON"] = 66, ["WASHINGTON"] = 67,
    };

    public FloridaPropertyDataClient(HttpClient httpClient, ILogger<FloridaPropertyDataClient> logger)
    {
        _httpClient = httpClient;
        _logger = logger;
        _httpClient.Timeout = TimeSpan.FromSeconds(30);
    }

    /// <summary>
    /// Search FDOT Statewide Parcels by owner name.
    /// Real endpoint: https://gis.fdot.gov/arcgis/rest/services/Parcels/FeatureServer/{layer}/query
    /// Fields: OWN_NAME, OWN_ADDR1, SITUS_ADDR, JV, AV_SD, TV_SD, LND_VAL, SALE_PRC1, etc.
    /// </summary>
    public async Task<List<PropertyRecord>> SearchByOwnerNameAsync(string ownerName, string county, int maxResults = 50)
    {
        try
        {
            if (!FdotCountyLayers.TryGetValue(county.ToUpper(), out var layerId))
            {
                _logger.LogWarning("Unknown county: {County}", county);
                return new List<PropertyRecord>();
            }

            var escapedName = ownerName.Replace("'", "''").ToUpper();
            var url = $"https://gis.fdot.gov/arcgis/rest/services/Parcels/FeatureServer/{layerId}/query" +
                      $"?where=OWN_NAME+LIKE+'%25{Uri.EscapeDataString(escapedName)}%25'" +
                      $"&outFields=PARCELNO,OWN_NAME,OWN_ADDR1,OWN_ADDR2,OWN_CITY,OWN_STATE,OWN_ZIPCD,SITUS_ADDR,SITUS_CITY,SITUS_ZIP,JV,AV_SD,TV_SD,LND_VAL,SALE_PRC1,ACT_YR_BLT,TOT_LVG_AR,NO_BULDNG,NO_RES_UNT,DOR_UC" +
                      $"&f=json&resultRecordCount={maxResults}&returnGeometry=true&outSR=4326";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            return ParseFdotResponse(await response.Content.ReadAsStringAsync(), county);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to search FDOT parcels by owner for {County}", county);
            return new List<PropertyRecord>();
        }
    }

    /// <summary>
    /// Search FDOT Statewide Parcels by situs (property) address.
    /// </summary>
    public async Task<List<PropertyRecord>> SearchByAddressAsync(string address, string county, int maxResults = 50)
    {
        try
        {
            if (!FdotCountyLayers.TryGetValue(county.ToUpper(), out var layerId))
            {
                _logger.LogWarning("Unknown county: {County}", county);
                return new List<PropertyRecord>();
            }

            var escapedAddr = address.Replace("'", "''").ToUpper();
            var url = $"https://gis.fdot.gov/arcgis/rest/services/Parcels/FeatureServer/{layerId}/query" +
                      $"?where=SITUS_ADDR+LIKE+'%25{Uri.EscapeDataString(escapedAddr)}%25'" +
                      $"&outFields=PARCELNO,OWN_NAME,OWN_ADDR1,OWN_ADDR2,OWN_CITY,OWN_STATE,OWN_ZIPCD,SITUS_ADDR,SITUS_CITY,SITUS_ZIP,JV,AV_SD,TV_SD,LND_VAL,SALE_PRC1,ACT_YR_BLT,TOT_LVG_AR,NO_BULDNG,NO_RES_UNT,DOR_UC" +
                      $"&f=json&resultRecordCount={maxResults}&returnGeometry=true&outSR=4326";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            return ParseFdotResponse(await response.Content.ReadAsStringAsync(), county);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to search FDOT parcels by address for {County}", county);
            return new List<PropertyRecord>();
        }
    }

    /// <summary>
    /// Search Florida Statewide Cadastral dataset by county code.
    /// Backup data source with max 2000 records per request.
    /// Real endpoint: https://services9.arcgis.com/Gh9awoU677aKree0/arcgis/rest/services/Florida_Statewide_Cadastral/FeatureServer/0
    /// </summary>
    public async Task<List<PropertyRecord>> SearchStatewideCadastralAsync(string county, string? addressFilter = null, int maxResults = 50)
    {
        try
        {
            if (!DorCountyCodes.TryGetValue(county.ToUpper(), out var countyCode))
                return new List<PropertyRecord>();

            var where = $"CO_NO={countyCode}";
            if (!string.IsNullOrEmpty(addressFilter))
            {
                var escaped = addressFilter.Replace("'", "''").ToUpper();
                where += $" AND SITUS_ADDR LIKE '%25{escaped}%25'";
            }

            var url = $"https://services9.arcgis.com/Gh9awoU677aKree0/arcgis/rest/services/Florida_Statewide_Cadastral/FeatureServer/0/query" +
                      $"?where={Uri.EscapeDataString(where)}" +
                      $"&outFields=*&f=json&resultRecordCount={maxResults}&returnGeometry=true&outSR=4326";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            return ParseFdotResponse(await response.Content.ReadAsStringAsync(), county);
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to search statewide cadastral for {County}", county);
            return new List<PropertyRecord>();
        }
    }

    /// <summary>
    /// Search all supported Florida counties for a property.
    /// Tries FDOT first, falls back to state cadastral, then Census geocoder.
    /// </summary>
    public async Task<List<PropertyRecord>> SearchFloridaPropertiesAsync(string query, string? county = null)
    {
        if (!string.IsNullOrEmpty(county))
        {
            // Try FDOT first
            var results = await SearchByAddressAsync(query, county);
            if (results.Count > 0) return results;

            // Fallback to statewide cadastral
            results = await SearchStatewideCadastralAsync(county, query);
            if (results.Count > 0) return results;
        }

        // Fallback to Census geocoder for any FL address
        return await SearchViaCensusGeocoderAsync(query);
    }

    /// <summary>
    /// US Census Bureau Geocoder - works for any Florida address.
    /// Real endpoint: https://geocoding.geo.census.gov/geocoder/locations/onelineaddress
    /// </summary>
    public async Task<List<PropertyRecord>> SearchViaCensusGeocoderAsync(string query)
    {
        try
        {
            var encoded = Uri.EscapeDataString(query.Contains("FL") ? query : query + ", FL");
            var url = $"https://geocoding.geo.census.gov/geocoder/locations/onelineaddress?address={encoded}&benchmark=Public_AR_Current&format=json";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            using var doc = JsonDocument.Parse(json);

            var records = new List<PropertyRecord>();
            var result = doc.RootElement.GetProperty("result");
            if (result.TryGetProperty("addressMatches", out var matches))
            {
                foreach (var match in matches.EnumerateArray())
                {
                    var coords = match.GetProperty("coordinates");
                    var addrComponents = match.GetProperty("addressComponents");
                    var state = addrComponents.TryGetProperty("state", out var st) ? st.GetString() : "";
                    if (state != "FL") continue;

                    records.Add(new PropertyRecord
                    {
                        Address = match.TryGetProperty("matchedAddress", out var addr) ? addr.GetString() ?? "" : "",
                        Latitude = coords.GetProperty("y").GetDouble(),
                        Longitude = coords.GetProperty("x").GetDouble(),
                        City = addrComponents.TryGetProperty("city", out var city) ? city.GetString() ?? "" : "",
                        ZipCode = addrComponents.TryGetProperty("zip", out var zip) ? zip.GetString() ?? "" : "",
                        State = "FL",
                        DataSource = "US Census Bureau Geocoder",
                    });
                }
            }
            return records;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to geocode via Census Bureau");
            return new List<PropertyRecord>();
        }
    }

    /// <summary>
    /// Get Miami-Dade building permits from their Open Data Hub.
    /// Real endpoint via ArcGIS Open Data Hub - updated weekly.
    /// </summary>
    public async Task<List<PermitRecord>> GetMiamiDadePermitsAsync(string address, int maxResults = 50)
    {
        try
        {
            var escaped = address.Replace("'", "''").ToUpper();
            // Miami-Dade Building Permit dataset
            var url = $"https://gisweb.miamidade.gov/arcgis/rest/services/MDC_OpenData/MD_BuildingPermit/FeatureServer/0/query" +
                      $"?where=ProcessAddress+LIKE+'%25{Uri.EscapeDataString(escaped)}%25'" +
                      $"&outFields=*&f=json&resultRecordCount={maxResults}&orderByFields=IssuedDate DESC";

            var response = await _httpClient.GetAsync(url);
            response.EnsureSuccessStatusCode();

            var json = await response.Content.ReadAsStringAsync();
            using var doc = JsonDocument.Parse(json);

            var permits = new List<PermitRecord>();
            if (doc.RootElement.TryGetProperty("features", out var features))
            {
                foreach (var feature in features.EnumerateArray())
                {
                    var attrs = feature.GetProperty("attributes");
                    permits.Add(new PermitRecord
                    {
                        PermitNumber = GetString(attrs, "PermitNumber"),
                        PermitType = GetString(attrs, "PermitType"),
                        Description = GetString(attrs, "ScopeOfWork"),
                        Status = GetString(attrs, "StatusCurrent"),
                        IssuedDate = GetTimestamp(attrs, "IssuedDate"),
                        ContractorName = GetString(attrs, "ContractorName"),
                        ContractorLicense = GetString(attrs, "ContractorLicenseNumber"),
                        EstimatedCost = GetDouble(attrs, "ApplicationValue"),
                        Address = GetString(attrs, "ProcessAddress"),
                    });
                }
            }
            return permits;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Failed to fetch Miami-Dade permits");
            return new List<PermitRecord>();
        }
    }

    /// <summary>
    /// Get all supported county names
    /// </summary>
    public List<string> GetSupportedCounties()
    {
        return FdotCountyLayers.Keys.OrderBy(k => k).ToList();
    }

    private List<PropertyRecord> ParseFdotResponse(string json, string county)
    {
        var records = new List<PropertyRecord>();
        using var doc = JsonDocument.Parse(json);

        if (!doc.RootElement.TryGetProperty("features", out var features)) return records;

        foreach (var feature in features.EnumerateArray())
        {
            var attrs = feature.GetProperty("attributes");

            double lat = 0, lon = 0;
            if (feature.TryGetProperty("geometry", out var geom))
            {
                if (geom.TryGetProperty("y", out var y)) lat = y.GetDouble();
                else if (geom.TryGetProperty("rings", out var rings) && rings.GetArrayLength() > 0)
                {
                    // Polygon centroid approximation from first ring
                    var ring = rings[0];
                    double sumX = 0, sumY = 0;
                    int n = ring.GetArrayLength();
                    foreach (var pt in ring.EnumerateArray())
                    {
                        sumX += pt[0].GetDouble();
                        sumY += pt[1].GetDouble();
                    }
                    lon = n > 0 ? sumX / n : 0;
                    lat = n > 0 ? sumY / n : 0;
                }
                if (geom.TryGetProperty("x", out var x)) lon = x.GetDouble();
            }

            records.Add(new PropertyRecord
            {
                ParcelId = GetString(attrs, "PARCELNO"),
                Address = GetString(attrs, "SITUS_ADDR"),
                City = GetString(attrs, "SITUS_CITY"),
                ZipCode = GetString(attrs, "SITUS_ZIP"),
                County = county,
                State = "FL",
                Latitude = lat,
                Longitude = lon,
                OwnerName = GetString(attrs, "OWN_NAME"),
                OwnerAddress = GetString(attrs, "OWN_ADDR1"),
                OwnerCity = GetString(attrs, "OWN_CITY"),
                OwnerState = GetString(attrs, "OWN_STATE"),
                OwnerZip = GetString(attrs, "OWN_ZIPCD"),
                YearBuilt = GetInt(attrs, "ACT_YR_BLT"),
                SquareFootage = GetDouble(attrs, "TOT_LVG_AR"),
                JustValue = GetDecimal(attrs, "JV"),
                AssessedValue = GetDecimal(attrs, "AV_SD"),
                TaxableValue = GetDecimal(attrs, "TV_SD"),
                DataSource = "FDOT Statewide Parcels / FL Dept of Revenue",
            });
        }

        return records;
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

    private static decimal? GetDecimal(JsonElement el, string prop)
    {
        if (!el.TryGetProperty(prop, out var v) || v.ValueKind == JsonValueKind.Null) return null;
        return v.ValueKind == JsonValueKind.Number ? v.GetDecimal() : null;
    }

    private static DateTime? GetTimestamp(JsonElement el, string prop)
    {
        if (!el.TryGetProperty(prop, out var v) || v.ValueKind == JsonValueKind.Null) return null;
        if (v.ValueKind == JsonValueKind.Number)
            return DateTimeOffset.FromUnixTimeMilliseconds(v.GetInt64()).DateTime;
        return null;
    }
}

public class PropertyRecord
{
    public string Address { get; set; } = string.Empty;
    public string City { get; set; } = string.Empty;
    public string County { get; set; } = string.Empty;
    public string State { get; set; } = "FL";
    public string ZipCode { get; set; } = string.Empty;
    public string ParcelId { get; set; } = string.Empty;
    public double Latitude { get; set; }
    public double Longitude { get; set; }
    public string DataSource { get; set; } = string.Empty;
    public int? YearBuilt { get; set; }
    public double? SquareFootage { get; set; }
    public double? LotSize { get; set; }
    public string? PropertyType { get; set; }
    public decimal? JustValue { get; set; }
    public decimal? AssessedValue { get; set; }
    public decimal? TaxableValue { get; set; }
    public int? Bedrooms { get; set; }
    public int? Bathrooms { get; set; }
    public string? OwnerName { get; set; }
    public string? OwnerAddress { get; set; }
    public string? OwnerCity { get; set; }
    public string? OwnerState { get; set; }
    public string? OwnerZip { get; set; }
}

public class PermitRecord
{
    public string PermitNumber { get; set; } = string.Empty;
    public string PermitType { get; set; } = string.Empty;
    public string? Description { get; set; }
    public string Status { get; set; } = string.Empty;
    public DateTime? IssuedDate { get; set; }
    public string? ContractorName { get; set; }
    public string? ContractorLicense { get; set; }
    public double? EstimatedCost { get; set; }
    public string Address { get; set; } = string.Empty;
}
