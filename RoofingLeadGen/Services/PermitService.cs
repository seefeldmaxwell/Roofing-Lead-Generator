using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Data;
using RoofingLeadGen.Models;

namespace RoofingLeadGen.Services;

public class PermitService
{
    private readonly AppDbContext _context;

    public PermitService(AppDbContext context)
    {
        _context = context;
    }

    public async Task<List<Permit>> GetPermitsByPropertyIdAsync(int propertyId)
    {
        return await _context.Permits
            .Where(p => p.PropertyId == propertyId)
            .OrderByDescending(p => p.IssuedDate)
            .ToListAsync();
    }

    public async Task<List<Permit>> GetRoofingPermitsByPropertyIdAsync(int propertyId)
    {
        return await _context.Permits
            .Where(p => p.PropertyId == propertyId && p.IsRoofingPermit)
            .OrderByDescending(p => p.IssuedDate)
            .ToListAsync();
    }

    public async Task<Permit?> GetPermitByIdAsync(int id)
    {
        return await _context.Permits
            .Include(p => p.Property)
            .FirstOrDefaultAsync(p => p.Id == id);
    }

    public async Task<List<Permit>> GetRecentPermitsAsync(int days = 30)
    {
        var cutoff = DateTime.Now.AddDays(-days);
        return await _context.Permits
            .Include(p => p.Property)
            .Where(p => p.IssuedDate >= cutoff)
            .OrderByDescending(p => p.IssuedDate)
            .Take(50)
            .ToListAsync();
    }
}
