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

    public int PropertyId { get; set; }

    [ForeignKey("PropertyId")]
    public Property? Property { get; set; }
}
