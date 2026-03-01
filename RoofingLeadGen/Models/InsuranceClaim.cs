using System.ComponentModel.DataAnnotations;
using System.ComponentModel.DataAnnotations.Schema;

namespace RoofingLeadGen.Models;

public class InsuranceClaim
{
    [Key]
    public int Id { get; set; }

    [StringLength(50)]
    public string ClaimNumber { get; set; } = string.Empty;

    [StringLength(100)]
    public string ClaimType { get; set; } = string.Empty;

    [StringLength(100)]
    public string CauseOfLoss { get; set; } = string.Empty;

    public DateTime? DateOfLoss { get; set; }

    public DateTime? DateFiled { get; set; }

    public DateTime? DateClosed { get; set; }

    [StringLength(50)]
    public string Status { get; set; } = string.Empty;

    [Column(TypeName = "decimal(18,2)")]
    public decimal? ClaimAmount { get; set; }

    [Column(TypeName = "decimal(18,2)")]
    public decimal? PaidAmount { get; set; }

    [Column(TypeName = "decimal(18,2)")]
    public decimal? Deductible { get; set; }

    [StringLength(200)]
    public string? InsuranceCompany { get; set; }

    [StringLength(50)]
    public string? PolicyNumber { get; set; }

    [StringLength(500)]
    public string? Description { get; set; }

    public bool IsRoofClaim { get; set; }
    public bool IsFloodClaim { get; set; }
    public bool IsFireClaim { get; set; }
    public bool IsWindClaim { get; set; }

    public bool PublicAdjusterInvolved { get; set; }

    [StringLength(200)]
    public string? PublicAdjusterName { get; set; }

    [StringLength(50)]
    public string? FemaDisasterNumber { get; set; }

    public int PropertyId { get; set; }

    [ForeignKey("PropertyId")]
    public Property? Property { get; set; }
}
