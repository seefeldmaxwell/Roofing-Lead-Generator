using System.ComponentModel.DataAnnotations;

namespace RoofingLeadGen.Models;

public class Owner
{
    [Key]
    public int Id { get; set; }

    [Required]
    [StringLength(100)]
    public string FirstName { get; set; } = string.Empty;

    [Required]
    [StringLength(100)]
    public string LastName { get; set; } = string.Empty;

    [StringLength(200)]
    public string? MailingAddress { get; set; }

    [StringLength(100)]
    public string? MailingCity { get; set; }

    [StringLength(10)]
    public string? MailingState { get; set; }

    [StringLength(10)]
    public string? MailingZip { get; set; }

    [StringLength(20)]
    public string? PhoneNumber { get; set; }

    [StringLength(150)]
    [EmailAddress]
    public string? Email { get; set; }

    public int? Age { get; set; }

    public DateTime? DateOfBirth { get; set; }

    public string FullName => $"{FirstName} {LastName}";

    public ICollection<Property> Properties { get; set; } = new List<Property>();
}
