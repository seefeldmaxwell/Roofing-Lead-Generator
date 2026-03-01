using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace RoofingLeadGen.Models;

/// <summary>Summary of a category of work done on a property, derived from permits.</summary>
public class WorkHistoryEntry
{
    public string WorkCategory { get; set; } = string.Empty;
    public int TotalPermits { get; set; }
    public DateTime? FirstPermitDate { get; set; }
    public DateTime? LastPermitDate { get; set; }
    public string? MostRecentStatus { get; set; }
    public decimal TotalEstimatedCost { get; set; }
    public int? YearsSinceLastWork { get; set; }
}

public class Property
{
    [Key]
    public int Id { get; set; }

    [Required]
    [StringLength(200)]
    public string Address { get; set; } = string.Empty;

    [StringLength(100)]
    public string City { get; set; } = string.Empty;

    [StringLength(10)]
    public string ZipCode { get; set; } = string.Empty;

    [StringLength(50)]
    public string County { get; set; } = string.Empty;

    [StringLength(20)]
    public string State { get; set; } = "FL";

    [StringLength(50)]
    public string ParcelId { get; set; } = string.Empty;

    public int? YearBuilt { get; set; }

    public double? SquareFootage { get; set; }

    public double? LotSize { get; set; }

    [StringLength(50)]
    public string PropertyType { get; set; } = string.Empty;

    [Column(TypeName = "decimal(18,2)")]
    public decimal? EstimatedValue { get; set; }

    public int? Bedrooms { get; set; }

    public int? Bathrooms { get; set; }

    [StringLength(50)]
    public string RoofType { get; set; } = string.Empty;

    /// <summary>
    /// Stored roof age value. Use CalculatedRoofAge for dynamic computation from permits.
    /// </summary>
    public int? RoofAge { get; set; }

    public DateTime? LastRoofPermitDate { get; set; }

    /// <summary>
    /// Dynamically calculates roof age based on the most recent roofing permit.
    /// Falls back to stored RoofAge, then to YearBuilt if no permit data exists.
    /// </summary>
    [NotMapped]
    public int? CalculatedRoofAge
    {
        get
        {
            // Priority 1: Most recent roofing permit date
            var lastRoofPermit = Permits?
                .Where(p => p.IsRoofingPermit && p.IssuedDate.HasValue)
                .OrderByDescending(p => p.IssuedDate)
                .FirstOrDefault();

            if (lastRoofPermit?.IssuedDate != null)
                return (int)((DateTime.Now - lastRoofPermit.IssuedDate.Value).TotalDays / 365.25);

            // Priority 2: Stored LastRoofPermitDate
            if (LastRoofPermitDate.HasValue)
                return (int)((DateTime.Now - LastRoofPermitDate.Value).TotalDays / 365.25);

            // Priority 3: Stored RoofAge (may be from external source)
            if (RoofAge.HasValue)
                return RoofAge.Value;

            // Priority 4: YearBuilt (assume original roof if no permits)
            if (YearBuilt.HasValue)
                return DateTime.Now.Year - YearBuilt.Value;

            return null;
        }
    }

    /// <summary>
    /// Gets the effective last roof permit date from actual permits or stored value.
    /// </summary>
    [NotMapped]
    public DateTime? EffectiveLastRoofPermitDate
    {
        get
        {
            var lastRoofPermit = Permits?
                .Where(p => p.IsRoofingPermit && p.IssuedDate.HasValue)
                .OrderByDescending(p => p.IssuedDate)
                .FirstOrDefault();

            return lastRoofPermit?.IssuedDate ?? LastRoofPermitDate;
        }
    }

    /// <summary>
    /// Gets a summary of all work done on this property from permits, grouped by category.
    /// </summary>
    [NotMapped]
    public List<WorkHistoryEntry> WorkHistory
    {
        get
        {
            if (Permits == null || !Permits.Any()) return new List<WorkHistoryEntry>();

            return Permits
                .Where(p => !string.IsNullOrEmpty(p.WorkCategory) && p.WorkCategory != "Other")
                .GroupBy(p => p.WorkCategory)
                .Select(g => new WorkHistoryEntry
                {
                    WorkCategory = g.Key,
                    TotalPermits = g.Count(),
                    FirstPermitDate = g.Min(p => p.IssuedDate),
                    LastPermitDate = g.Max(p => p.IssuedDate),
                    MostRecentStatus = g.OrderByDescending(p => p.IssuedDate).First().Status,
                    TotalEstimatedCost = g.Sum(p => p.EstimatedCost ?? 0),
                    YearsSinceLastWork = g.Max(p => p.IssuedDate).HasValue
                        ? (int)((DateTime.Now - g.Max(p => p.IssuedDate)!.Value).TotalDays / 365.25)
                        : (int?)null,
                })
                .OrderByDescending(w => w.LastPermitDate)
                .ToList();
        }
    }

    [StringLength(500)]
    public string? ImageUrl { get; set; }

    public double? Latitude { get; set; }

    public double? Longitude { get; set; }

    // Flood data (from FEMA NFHL)
    [StringLength(10)]
    public string? FloodZone { get; set; }

    [StringLength(100)]
    public string? FloodZoneDescription { get; set; }

    public bool IsInFloodZone { get; set; }

    public bool IsInHighRiskFloodZone { get; set; }

    [StringLength(50)]
    public string? FemaMapPanel { get; set; }

    public DateTime? FloodZoneLastUpdated { get; set; }

    public bool RequiresFloodInsurance { get; set; }

    // Fire risk data
    [StringLength(20)]
    public string? FireRiskLevel { get; set; }

    public int? WildfireRiskScore { get; set; }

    public double? DistanceToFireStation { get; set; }

    [StringLength(20)]
    public string? FireProtectionClass { get; set; }

    public bool IsInWildfireZone { get; set; }

    // Insurance data
    public bool HasHomeownersInsurance { get; set; }

    public bool HasFloodInsurance { get; set; }

    public bool HasWindInsurance { get; set; }

    [Column(TypeName = "decimal(18,2)")]
    public decimal? InsuredValue { get; set; }

    [StringLength(100)]
    public string? InsuranceCarrier { get; set; }

    // Risk scoring
    public int? OverallRiskScore { get; set; }

    [StringLength(20)]
    public string? LeadPriority { get; set; }

    // Data source tracking
    [StringLength(200)]
    public string? DataSource { get; set; }

    public DateTime? LastDataRefresh { get; set; }

    public int? OwnerId { get; set; }

    [ForeignKey("OwnerId")]
    public Owner? Owner { get; set; }

    public ICollection<Permit> Permits { get; set; } = new List<Permit>();
    public ICollection<InsuranceClaim> InsuranceClaims { get; set; } = new List<InsuranceClaim>();
    public ICollection<FireIncident> FireIncidents { get; set; } = new List<FireIncident>();
}
