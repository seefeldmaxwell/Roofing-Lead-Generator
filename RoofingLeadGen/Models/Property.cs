using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace RoofingLeadGen.Models;

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

    public int? RoofAge { get; set; }

    public DateTime? LastRoofPermitDate { get; set; }

    [StringLength(500)]
    public string? ImageUrl { get; set; }

    public double? Latitude { get; set; }

    public double? Longitude { get; set; }

    public int? OwnerId { get; set; }

    [ForeignKey("OwnerId")]
    public Owner? Owner { get; set; }

    public ICollection<Permit> Permits { get; set; } = new List<Permit>();
}
