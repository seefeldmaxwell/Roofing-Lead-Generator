using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace RoofingLeadGen.Models;

public class Permit
{
    [Key]
    public int Id { get; set; }

    [Required]
    [StringLength(50)]
    public string PermitNumber { get; set; } = string.Empty;

    [Required]
    [StringLength(100)]
    public string PermitType { get; set; } = string.Empty;

    /// <summary>
    /// Normalized work category derived from PermitType + Description.
    /// Values: Roofing, Electrical, Plumbing, HVAC, Structural, Windows/Doors,
    ///         Siding/Exterior, Flooring, General Building, Demolition, Solar,
    ///         Pool, Fence, Fire Alarm, Mechanical, Foundation, Other
    /// </summary>
    [StringLength(50)]
    public string WorkCategory { get; set; } = string.Empty;

    [StringLength(500)]
    public string? Description { get; set; }

    public DateTime? IssuedDate { get; set; }

    public DateTime? CompletedDate { get; set; }

    public DateTime? ExpirationDate { get; set; }

    [StringLength(50)]
    public string Status { get; set; } = string.Empty;

    [StringLength(200)]
    public string? ContractorName { get; set; }

    [StringLength(50)]
    public string? ContractorLicense { get; set; }

    [Column(TypeName = "decimal(18,2)")]
    public decimal? EstimatedCost { get; set; }

    public bool IsRoofingPermit { get; set; }

    /// <summary>Data source this permit was pulled from (e.g., "Miami-Dade Open Data")</summary>
    [StringLength(100)]
    public string? DataSource { get; set; }

    public int PropertyId { get; set; }

    [ForeignKey("PropertyId")]
    public Property? Property { get; set; }

    /// <summary>
    /// Classify the permit into a normalized WorkCategory based on permit type and description text.
    /// Also sets IsRoofingPermit flag.
    /// </summary>
    public static (string WorkCategory, bool IsRoofing) ClassifyPermit(string? permitType, string? description)
    {
        var type = (permitType ?? "").ToUpperInvariant();
        var desc = (description ?? "").ToUpperInvariant();
        var combined = type + " " + desc;

        // Roofing
        if (ContainsAny(combined, "ROOF", "RE-ROOF", "REROOF", "SHINGLE", "TILE ROOF", "METAL ROOF", "FLAT ROOF"))
            return ("Roofing", true);

        // Electrical
        if (ContainsAny(combined, "ELECTR", "WIRING", "PANEL UPGRADE", "CIRCUIT", "ELECTRICAL SERVICE", "METER"))
            return ("Electrical", false);

        // Plumbing
        if (ContainsAny(combined, "PLUMB", "WATER HEATER", "SEWER", "DRAIN", "PIPE", "BACKFLOW", "IRRIGATION"))
            return ("Plumbing", false);

        // HVAC / Air Conditioning
        if (ContainsAny(combined, "HVAC", "AIR COND", "A/C", "AC UNIT", "HEATING", "FURNACE", "DUCT", "HEAT PUMP", "MECHANICAL"))
            return ("HVAC", false);

        // Windows & Doors
        if (ContainsAny(combined, "WINDOW", "DOOR", "IMPACT WINDOW", "IMPACT DOOR", "SHUTTER", "GLASS", "GLAZING", "STOREFRONT"))
            return ("Windows/Doors", false);

        // Structural
        if (ContainsAny(combined, "STRUCTUR", "LOAD BEARING", "BEAM", "COLUMN", "TRUSS", "FRAMING", "ADDITION"))
            return ("Structural", false);

        // Siding / Exterior
        if (ContainsAny(combined, "SIDING", "STUCCO", "EXTERIOR", "PAINT", "FASCIA", "SOFFIT", "GUTTER"))
            return ("Siding/Exterior", false);

        // Foundation
        if (ContainsAny(combined, "FOUNDATION", "SLAB", "FOOTING", "CONCRETE", "PILING"))
            return ("Foundation", false);

        // Solar
        if (ContainsAny(combined, "SOLAR", "PHOTOVOLTAIC", "PV SYSTEM"))
            return ("Solar", false);

        // Pool
        if (ContainsAny(combined, "POOL", "SPA", "HOT TUB"))
            return ("Pool", false);

        // Fence
        if (ContainsAny(combined, "FENCE", "FENCING", "GATE"))
            return ("Fence", false);

        // Fire Alarm / Sprinkler
        if (ContainsAny(combined, "FIRE ALARM", "SPRINKLER", "FIRE SUPPRESS", "FIRE PROTECT"))
            return ("Fire Alarm", false);

        // Demolition
        if (ContainsAny(combined, "DEMOL", "TEAR DOWN", "RAZE"))
            return ("Demolition", false);

        // Flooring / Interior
        if (ContainsAny(combined, "FLOOR", "TILE", "CARPET", "INTERIOR", "REMODEL", "RENOVAT", "KITCHEN", "BATH"))
            return ("Interior Remodel", false);

        // General Building
        if (ContainsAny(combined, "BUILD", "CONSTRUCT", "NEW CONSTRUCT", "ALTERATION"))
            return ("General Building", false);

        return ("Other", false);
    }

    private static bool ContainsAny(string text, params string[] keywords)
    {
        foreach (var keyword in keywords)
        {
            if (text.Contains(keyword, StringComparison.OrdinalIgnoreCase))
                return true;
        }
        return false;
    }
}
