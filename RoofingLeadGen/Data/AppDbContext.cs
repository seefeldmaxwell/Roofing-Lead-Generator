using Microsoft.AspNetCore.Identity.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Data;

public class AppDbContext : IdentityDbContext<ApplicationUser>
{
    public AppDbContext(DbContextOptions<AppDbContext> options) : base(options) { }

    public DbSet<Property> Properties => Set<Property>();
    public DbSet<Owner> Owners => Set<Owner>();
    public DbSet<Permit> Permits => Set<Permit>();
    public DbSet<InsuranceClaim> InsuranceClaims => Set<InsuranceClaim>();
    public DbSet<FireIncident> FireIncidents => Set<FireIncident>();
    public DbSet<FemaDisaster> FemaDisasters => Set<FemaDisaster>();

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        base.OnModelCreating(modelBuilder);

        modelBuilder.Entity<Property>()
            .HasOne(p => p.Owner)
            .WithMany(o => o.Properties)
            .HasForeignKey(p => p.OwnerId);

        modelBuilder.Entity<Permit>()
            .HasOne(p => p.Property)
            .WithMany(pr => pr.Permits)
            .HasForeignKey(p => p.PropertyId);

        modelBuilder.Entity<InsuranceClaim>()
            .HasOne(c => c.Property)
            .WithMany(p => p.InsuranceClaims)
            .HasForeignKey(c => c.PropertyId);

        modelBuilder.Entity<FireIncident>()
            .HasOne(f => f.Property)
            .WithMany(p => p.FireIncidents)
            .HasForeignKey(f => f.PropertyId);

        modelBuilder.Entity<Property>().HasIndex(p => p.Address);
        modelBuilder.Entity<Property>().HasIndex(p => p.City);
        modelBuilder.Entity<Property>().HasIndex(p => p.County);
        modelBuilder.Entity<Property>().HasIndex(p => p.ZipCode);
        modelBuilder.Entity<Property>().HasIndex(p => p.FloodZone);
        modelBuilder.Entity<Property>().HasIndex(p => p.IsInHighRiskFloodZone);
        modelBuilder.Entity<Property>().HasIndex(p => p.FireRiskLevel);
        modelBuilder.Entity<Property>().HasIndex(p => p.OverallRiskScore);
        modelBuilder.Entity<Property>().HasIndex(p => p.LeadPriority);

        modelBuilder.Entity<Owner>().HasIndex(o => new { o.LastName, o.FirstName });
        modelBuilder.Entity<InsuranceClaim>().HasIndex(c => c.ClaimType);
        modelBuilder.Entity<InsuranceClaim>().HasIndex(c => c.DateOfLoss);
        modelBuilder.Entity<FemaDisaster>().HasIndex(d => d.DisasterNumber);
        modelBuilder.Entity<FemaDisaster>().HasIndex(d => d.DeclaredCounty);
    }
}
