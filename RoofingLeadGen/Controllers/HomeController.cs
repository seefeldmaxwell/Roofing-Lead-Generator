using Microsoft.AspNetCore.Mvc;
using RoofingLeadGen.Services;

namespace RoofingLeadGen.Controllers;

public class HomeController : Controller
{
    private readonly LandingPageService _landingService;

    public HomeController(LandingPageService landingService)
    {
        _landingService = landingService;
    }

    [HttpGet("/")]
    public async Task<IActionResult> Index()
    {
        var stats = await _landingService.GetGlobalStatsAsync();
        ViewData["TotalProperties"] = stats.TotalProperties;
        ViewData["TotalPermits"] = stats.TotalPermits;
        ViewData["TotalDisasters"] = stats.TotalDisasters;
        ViewData["CountiesTracked"] = stats.CountiesTracked;
        return View();
    }

    [HttpGet("/florida/{countySlug}")]
    public async Task<IActionResult> County(string countySlug)
    {
        if (!LandingPageService.FloridaCounties.TryGetValue(countySlug, out var countyName))
            return NotFound();

        var stats = await _landingService.GetCountyStatsAsync(countyName);
        ViewData["CountySlug"] = countySlug;
        ViewData["CountyName"] = countyName;
        ViewData["Stats"] = stats;
        return View("County");
    }

    [HttpGet("/florida/{countySlug}/{citySlug}")]
    public async Task<IActionResult> City(string countySlug, string citySlug)
    {
        if (!LandingPageService.FloridaCounties.TryGetValue(countySlug, out var countyName))
            return NotFound();

        if (!LandingPageService.FloridaCities.TryGetValue(citySlug, out var cityInfo))
            return NotFound();

        var stats = await _landingService.GetCountyStatsAsync(countyName);
        ViewData["CountySlug"] = countySlug;
        ViewData["CountyName"] = countyName;
        ViewData["CitySlug"] = citySlug;
        ViewData["CityName"] = cityInfo.City;
        ViewData["Stats"] = stats;
        return View("City");
    }

    [HttpGet("/counties")]
    public IActionResult Counties()
    {
        return View();
    }
}
