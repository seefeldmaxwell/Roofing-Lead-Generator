using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Models;
using RoofingLeadGen.Services;

namespace RoofingLeadGen.Data;

/// <summary>
/// Seeds the database with real data fetched from public government APIs.
/// On first run, pulls FEMA disaster declarations for Florida.
/// Property data is fetched on-demand from FDOT/state parcel APIs when users search.
/// </summary>
public static class SeedData
{
    public static async Task InitializeAsync(AppDbContext context, IServiceProvider services)
    {
        context.Database.EnsureCreated();

        // Seed FEMA disaster declarations for Florida (real data from OpenFEMA API)
        if (!await context.FemaDisasters.AnyAsync())
        {
            var femaClient = services.GetService<FemaApiClient>();
            if (femaClient != null)
            {
                try
                {
                    var disasters = await femaClient.GetFloridaDisastersAsync(200);
                    if (disasters.Count > 0)
                    {
                        context.FemaDisasters.AddRange(disasters);
                        await context.SaveChangesAsync();
                    }
                }
                catch (Exception ex)
                {
                    var logger = services.GetService<ILogger<AppDbContext>>();
                    logger?.LogWarning(ex, "Failed to seed FEMA disaster data - will retry on demand");
                }
            }
        }
    }
}
