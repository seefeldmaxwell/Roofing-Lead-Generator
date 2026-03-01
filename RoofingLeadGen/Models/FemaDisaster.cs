using System.ComponentModel.DataAnnotations;

namespace RoofingLeadGen.Models;

/// <summary>
/// FEMA disaster declaration data - pulled from real FEMA API
/// </summary>
public class FemaDisaster
{
    [Key]
    public int Id { get; set; }

    [StringLength(20)]
    public string DisasterNumber { get; set; } = string.Empty;

    [StringLength(300)]
    public string Title { get; set; } = string.Empty;

    [StringLength(100)]
    public string DisasterType { get; set; } = string.Empty;

    public DateTime? DeclarationDate { get; set; }

    public DateTime? IncidentBeginDate { get; set; }

    public DateTime? IncidentEndDate { get; set; }

    public DateTime? CloseoutDate { get; set; }

    [StringLength(20)]
    public string State { get; set; } = "FL";

    [StringLength(100)]
    public string? DeclaredCounty { get; set; }

    [StringLength(50)]
    public string? IncidentType { get; set; }

    [StringLength(500)]
    public string? ProgramsAvailable { get; set; }

    public bool IndividualAssistance { get; set; }

    public bool PublicAssistance { get; set; }

    public bool HazardMitigation { get; set; }

    [StringLength(500)]
    public string? FemaUrl { get; set; }
}

/// <summary>
/// FEMA flood zone lookup result
/// </summary>
public class FloodZoneInfo
{
    public string FloodZone { get; set; } = string.Empty;
    public string ZoneDescription { get; set; } = string.Empty;
    public bool IsHighRisk { get; set; }
    public bool RequiresInsurance { get; set; }
    public string? MapPanel { get; set; }
    public string? CommunityName { get; set; }
    public DateTime? EffectiveDate { get; set; }
}

/// <summary>
/// NOAA storm event data
/// </summary>
public class StormEvent
{
    public string EventId { get; set; } = string.Empty;
    public string EventType { get; set; } = string.Empty;
    public DateTime? BeginDate { get; set; }
    public DateTime? EndDate { get; set; }
    public string? County { get; set; }
    public string? State { get; set; }
    public double? BeginLat { get; set; }
    public double? BeginLon { get; set; }
    public string? Source { get; set; }
    public string? Description { get; set; }
    public int? PropertyDamage { get; set; }
    public int? CropDamage { get; set; }
    public int? Injuries { get; set; }
    public int? Deaths { get; set; }
    public string? Magnitude { get; set; }
    public string? TornadoFScale { get; set; }
}
