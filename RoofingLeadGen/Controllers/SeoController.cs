using System.Text;
using Microsoft.AspNetCore.Mvc;
using RoofingLeadGen.Services;

namespace RoofingLeadGen.Controllers;

public class SeoController : Controller
{
    [HttpGet("/robots.txt")]
    [Produces("text/plain")]
    public IActionResult Robots()
    {
        var sb = new StringBuilder();
        sb.AppendLine("User-agent: *");
        sb.AppendLine("Allow: /");
        sb.AppendLine("Allow: /florida/");
        sb.AppendLine("Allow: /counties");
        sb.AppendLine("Disallow: /dashboard");
        sb.AppendLine("Disallow: /search");
        sb.AppendLine("Disallow: /property/");
        sb.AppendLine("Disallow: /permits");
        sb.AppendLine("Disallow: /risk");
        sb.AppendLine("Disallow: /disasters");
        sb.AppendLine("Disallow: /api-status");
        sb.AppendLine("Disallow: /api/");
        sb.AppendLine("Disallow: /account/");
        sb.AppendLine();
        sb.AppendLine("Sitemap: /sitemap.xml");
        return Content(sb.ToString(), "text/plain");
    }

    [HttpGet("/sitemap.xml")]
    [Produces("application/xml")]
    public IActionResult Sitemap()
    {
        var sb = new StringBuilder();
        sb.AppendLine("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        sb.AppendLine("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">");

        // Home page
        AddUrl(sb, "/", "1.0", "daily");

        // Counties index
        AddUrl(sb, "/counties", "0.9", "weekly");

        // County pages
        foreach (var county in LandingPageService.FloridaCounties)
        {
            AddUrl(sb, $"/florida/{county.Key}", "0.8", "weekly");
        }

        // City pages
        foreach (var city in LandingPageService.FloridaCities)
        {
            var countySlug = LandingPageService.FloridaCounties
                .FirstOrDefault(c => c.Value == city.Value.County).Key;
            if (countySlug != null)
            {
                AddUrl(sb, $"/florida/{countySlug}/{city.Key}", "0.7", "weekly");
            }
        }

        sb.AppendLine("</urlset>");
        return Content(sb.ToString(), "application/xml");
    }

    private static void AddUrl(StringBuilder sb, string path, string priority, string changefreq)
    {
        sb.AppendLine("  <url>");
        sb.AppendLine($"    <loc>{path}</loc>");
        sb.AppendLine($"    <changefreq>{changefreq}</changefreq>");
        sb.AppendLine($"    <priority>{priority}</priority>");
        sb.AppendLine("  </url>");
    }
}
