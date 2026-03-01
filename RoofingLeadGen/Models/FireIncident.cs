using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace RoofingLeadGen.Models;

public class FireIncident
{
    [Key]
    public int Id { get; set; }

    [StringLength(50)]
    public string IncidentId { get; set; } = string.Empty;

    [StringLength(200)]
    public string IncidentName { get; set; } = string.Empty;

    [StringLength(100)]
    public string IncidentType { get; set; } = string.Empty;

    public DateTime? DiscoveryDate { get; set; }

    public DateTime? ContainedDate { get; set; }

    public double? AcresBurned { get; set; }

    [StringLength(50)]
    public string? County { get; set; }

    public double? Latitude { get; set; }

    public double? Longitude { get; set; }

    public double? DistanceToProperty { get; set; }

    [StringLength(50)]
    public string? Cause { get; set; }

    public int? StructuresDamaged { get; set; }

    public int? StructuresDestroyed { get; set; }

    [StringLength(500)]
    public string? Description { get; set; }

    [StringLength(100)]
    public string? Source { get; set; }

    public int PropertyId { get; set; }

    [ForeignKey("PropertyId")]
    public Property? Property { get; set; }
}
