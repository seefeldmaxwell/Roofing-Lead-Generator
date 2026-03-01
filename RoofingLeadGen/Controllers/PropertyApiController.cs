using Microsoft.AspNetCore.Mvc;
using RoofingLeadGen.Models;
using RoofingLeadGen.Services;

namespace RoofingLeadGen.Controllers;

[ApiController]
[Route("api/[controller]")]
public class PropertiesController : ControllerBase
{
    private readonly PropertyService _propertyService;
    private readonly PermitService _permitService;

    public PropertiesController(PropertyService propertyService, PermitService permitService)
    {
        _propertyService = propertyService;
        _permitService = permitService;
    }

    [HttpGet("search")]
    public async Task<ActionResult<PaginatedResult<SearchResult>>> Search(
        [FromQuery] string? q,
        [FromQuery] string mode = "address",
        [FromQuery] string? city = null,
        [FromQuery] string? county = null,
        [FromQuery] string? zip = null,
        [FromQuery] int? minRoofAge = null,
        [FromQuery] int? maxRoofAge = null,
        [FromQuery] int page = 1,
        [FromQuery] int pageSize = 25)
    {
        var request = new SearchRequest
        {
            Query = q,
            Mode = mode.ToLower() == "owner" ? SearchMode.Owner : SearchMode.Address,
            City = city,
            County = county,
            ZipCode = zip,
            MinRoofAge = minRoofAge,
            MaxRoofAge = maxRoofAge,
            Page = page,
            PageSize = Math.Min(pageSize, 100)
        };

        var result = request.Mode == SearchMode.Owner
            ? await _propertyService.SearchByOwnerAsync(request)
            : await _propertyService.SearchByAddressAsync(request);

        return Ok(result);
    }

    [HttpGet("{id}")]
    public async Task<ActionResult<Property>> GetProperty(int id)
    {
        var property = await _propertyService.GetPropertyByIdAsync(id);
        if (property == null) return NotFound();
        return Ok(property);
    }

    [HttpGet("{id}/permits")]
    public async Task<ActionResult<List<Permit>>> GetPermits(int id)
    {
        var permits = await _permitService.GetPermitsByPropertyIdAsync(id);
        return Ok(permits);
    }

    [HttpGet("stats")]
    public async Task<ActionResult<DashboardStats>> GetStats()
    {
        var stats = await _propertyService.GetDashboardStatsAsync();
        return Ok(stats);
    }

    [HttpGet("counties")]
    public async Task<ActionResult<List<string>>> GetCounties()
    {
        var counties = await _propertyService.GetCountiesAsync();
        return Ok(counties);
    }

    [HttpGet("cities")]
    public async Task<ActionResult<List<string>>> GetCities([FromQuery] string? county = null)
    {
        var cities = await _propertyService.GetCitiesAsync(county);
        return Ok(cities);
    }
}
