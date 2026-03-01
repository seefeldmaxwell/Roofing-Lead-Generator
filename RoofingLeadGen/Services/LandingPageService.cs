using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Data;

namespace RoofingLeadGen.Services;

public class CountyStats
{
    public string County { get; set; } = string.Empty;
    public int PropertyCount { get; set; }
    public int PermitCount { get; set; }
    public int OldRoofCount { get; set; }
    public int FloodZoneCount { get; set; }
    public int HighFireRiskCount { get; set; }
    public int DisasterCount { get; set; }
}

public class LandingPageService
{
    private readonly AppDbContext _context;
    private readonly FemaApiClient _femaClient;
    private readonly ILogger<LandingPageService> _logger;

    // All 67 Florida counties for location pages
    public static readonly Dictionary<string, string> FloridaCounties = new(StringComparer.OrdinalIgnoreCase)
    {
        ["alachua"] = "Alachua", ["baker"] = "Baker", ["bay"] = "Bay", ["bradford"] = "Bradford",
        ["brevard"] = "Brevard", ["broward"] = "Broward", ["calhoun"] = "Calhoun", ["charlotte"] = "Charlotte",
        ["citrus"] = "Citrus", ["clay"] = "Clay", ["collier"] = "Collier", ["columbia"] = "Columbia",
        ["desoto"] = "DeSoto", ["dixie"] = "Dixie", ["duval"] = "Duval", ["escambia"] = "Escambia",
        ["flagler"] = "Flagler", ["franklin"] = "Franklin", ["gadsden"] = "Gadsden", ["gilchrist"] = "Gilchrist",
        ["glades"] = "Glades", ["gulf"] = "Gulf", ["hamilton"] = "Hamilton", ["hardee"] = "Hardee",
        ["hendry"] = "Hendry", ["hernando"] = "Hernando", ["highlands"] = "Highlands", ["hillsborough"] = "Hillsborough",
        ["holmes"] = "Holmes", ["indian-river"] = "Indian River", ["jackson"] = "Jackson", ["jefferson"] = "Jefferson",
        ["lafayette"] = "Lafayette", ["lake"] = "Lake", ["lee"] = "Lee", ["leon"] = "Leon",
        ["levy"] = "Levy", ["liberty"] = "Liberty", ["madison"] = "Madison", ["manatee"] = "Manatee",
        ["marion"] = "Marion", ["martin"] = "Martin", ["miami-dade"] = "Miami-Dade", ["monroe"] = "Monroe",
        ["nassau"] = "Nassau", ["okaloosa"] = "Okaloosa", ["okeechobee"] = "Okeechobee", ["orange"] = "Orange",
        ["osceola"] = "Osceola", ["palm-beach"] = "Palm Beach", ["pasco"] = "Pasco", ["pinellas"] = "Pinellas",
        ["polk"] = "Polk", ["putnam"] = "Putnam", ["santa-rosa"] = "Santa Rosa", ["sarasota"] = "Sarasota",
        ["seminole"] = "Seminole", ["st-johns"] = "St. Johns", ["st-lucie"] = "St. Lucie", ["sumter"] = "Sumter",
        ["suwannee"] = "Suwannee", ["taylor"] = "Taylor", ["union"] = "Union", ["volusia"] = "Volusia",
        ["wakulla"] = "Wakulla", ["walton"] = "Walton", ["washington"] = "Washington",
    };

    // Major cities mapped to their counties for city-level pages
    public static readonly Dictionary<string, (string City, string County)> FloridaCities = new(StringComparer.OrdinalIgnoreCase)
    {
        ["miami"] = ("Miami", "Miami-Dade"), ["fort-lauderdale"] = ("Fort Lauderdale", "Broward"),
        ["west-palm-beach"] = ("West Palm Beach", "Palm Beach"), ["tampa"] = ("Tampa", "Hillsborough"),
        ["orlando"] = ("Orlando", "Orange"), ["jacksonville"] = ("Jacksonville", "Duval"),
        ["st-petersburg"] = ("St. Petersburg", "Pinellas"), ["hialeah"] = ("Hialeah", "Miami-Dade"),
        ["tallahassee"] = ("Tallahassee", "Leon"), ["cape-coral"] = ("Cape Coral", "Lee"),
        ["fort-myers"] = ("Fort Myers", "Lee"), ["pembroke-pines"] = ("Pembroke Pines", "Broward"),
        ["hollywood"] = ("Hollywood", "Broward"), ["coral-springs"] = ("Coral Springs", "Broward"),
        ["clearwater"] = ("Clearwater", "Pinellas"), ["miami-gardens"] = ("Miami Gardens", "Miami-Dade"),
        ["pompano-beach"] = ("Pompano Beach", "Broward"), ["lakeland"] = ("Lakeland", "Polk"),
        ["davie"] = ("Davie", "Broward"), ["boca-raton"] = ("Boca Raton", "Palm Beach"),
        ["sunrise"] = ("Sunrise", "Broward"), ["plantation"] = ("Plantation", "Broward"),
        ["deerfield-beach"] = ("Deerfield Beach", "Broward"), ["port-st-lucie"] = ("Port St. Lucie", "St. Lucie"),
        ["palm-bay"] = ("Palm Bay", "Brevard"), ["boynton-beach"] = ("Boynton Beach", "Palm Beach"),
        ["kissimmee"] = ("Kissimmee", "Osceola"), ["naples"] = ("Naples", "Collier"),
        ["sarasota"] = ("Sarasota", "Sarasota"), ["ocala"] = ("Ocala", "Marion"),
        ["gainesville"] = ("Gainesville", "Alachua"), ["pensacola"] = ("Pensacola", "Escambia"),
        ["daytona-beach"] = ("Daytona Beach", "Volusia"), ["melbourne"] = ("Melbourne", "Brevard"),
        ["homestead"] = ("Homestead", "Miami-Dade"), ["deltona"] = ("Deltona", "Volusia"),
        ["palm-coast"] = ("Palm Coast", "Flagler"), ["sanford"] = ("Sanford", "Seminole"),
        ["key-west"] = ("Key West", "Monroe"), ["panama-city"] = ("Panama City", "Bay"),
    };

    public LandingPageService(AppDbContext context, FemaApiClient femaClient, ILogger<LandingPageService> logger)
    {
        _context = context;
        _femaClient = femaClient;
        _logger = logger;
    }

    public async Task<CountyStats> GetCountyStatsAsync(string countyName)
    {
        try
        {
            var stats = new CountyStats { County = countyName };

            var properties = await _context.Properties
                .Where(p => p.County.ToLower() == countyName.ToLower() ||
                            p.County.ToLower().Replace("-", " ").Replace("_", " ") == countyName.ToLower().Replace("-", " "))
                .Include(p => p.Permits)
                .ToListAsync();

            stats.PropertyCount = properties.Count;
            stats.PermitCount = properties.Sum(p => p.Permits.Count);
            stats.OldRoofCount = properties.Count(p => (p.CalculatedRoofAge ?? 0) > 15);
            stats.FloodZoneCount = properties.Count(p => p.IsInFloodZone);
            stats.HighFireRiskCount = properties.Count(p => p.WildfireRiskScore > 50);

            stats.DisasterCount = await _context.FemaDisasters
                .CountAsync(d => d.DeclaredCounty != null &&
                    (d.DeclaredCounty.ToLower() == countyName.ToLower() ||
                     d.DeclaredCounty.ToLower().Replace("-", " ") == countyName.ToLower().Replace("-", " ")));

            return stats;
        }
        catch (Exception ex)
        {
            _logger.LogWarning(ex, "Error fetching county stats for {County}", countyName);
            return new CountyStats { County = countyName };
        }
    }

    public async Task<(int TotalProperties, int TotalPermits, int TotalDisasters, int CountiesTracked)> GetGlobalStatsAsync()
    {
        try
        {
            var totalProperties = await _context.Properties.CountAsync();
            var totalPermits = await _context.Permits.CountAsync();
            var totalDisasters = await _context.FemaDisasters.CountAsync();
            var counties = await _context.Properties.Select(p => p.County).Distinct().CountAsync();
            return (totalProperties, totalPermits, totalDisasters, Math.Max(counties, 67));
        }
        catch
        {
            return (0, 0, 0, 67);
        }
    }
}
