using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Data;

public class AppDbContext : DbContext
{
    public AppDbContext(DbContextOptions<AppDbContext> options) : base(options) { }

    public DbSet<Property> Properties => Set<Property>();
    public DbSet<Owner> Owners => Set<Owner>();
    public DbSet<Permit> Permits => Set<Permit>();

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

        modelBuilder.Entity<Property>()
            .HasIndex(p => p.Address);

        modelBuilder.Entity<Property>()
            .HasIndex(p => p.City);

        modelBuilder.Entity<Property>()
            .HasIndex(p => p.County);

        modelBuilder.Entity<Property>()
            .HasIndex(p => p.ZipCode);

        modelBuilder.Entity<Owner>()
            .HasIndex(o => new { o.LastName, o.FirstName });
    }
}
