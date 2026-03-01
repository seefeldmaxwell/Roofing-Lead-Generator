using Microsoft.AspNetCore.Identity;

namespace RoofingLeadGen.Models;

public class ApplicationUser : IdentityUser
{
    public string? FullName { get; set; }
    public string? Company { get; set; }
    public string? AvatarUrl { get; set; }
    public DateTime CreatedAt { get; set; } = DateTime.UtcNow;
}
