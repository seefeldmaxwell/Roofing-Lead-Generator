using Microsoft.AspNetCore.Identity;
using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Data;
using RoofingLeadGen.Models;
using RoofingLeadGen.Services;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddRazorPages();
builder.Services.AddServerSideBlazor();
builder.Services.AddControllersWithViews()
    .AddJsonOptions(opts =>
    {
        opts.JsonSerializerOptions.ReferenceHandler = System.Text.Json.Serialization.ReferenceHandler.IgnoreCycles;
        opts.JsonSerializerOptions.DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull;
    });

builder.Services.AddDbContext<AppDbContext>(options =>
    options.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")
        ?? "Data Source=RoofingLeadGen.db"));

// ASP.NET Core Identity (OAuth-only, no passwords)
builder.Services.AddIdentity<ApplicationUser, IdentityRole>(options =>
{
    options.User.RequireUniqueEmail = true;
})
.AddEntityFrameworkStores<AppDbContext>()
.AddDefaultTokenProviders();

builder.Services.ConfigureApplicationCookie(options =>
{
    options.LoginPath = "/account/login";
    options.LogoutPath = "/account/logout";
    options.AccessDeniedPath = "/account/login";
});

// External OAuth providers (Microsoft + Google)
builder.Services.AddAuthentication()
    .AddMicrosoftAccount(options =>
    {
        options.ClientId = builder.Configuration["Authentication:Microsoft:ClientId"] ?? "";
        options.ClientSecret = builder.Configuration["Authentication:Microsoft:ClientSecret"] ?? "";
    })
    .AddGoogle(options =>
    {
        options.ClientId = builder.Configuration["Authentication:Google:ClientId"] ?? "";
        options.ClientSecret = builder.Configuration["Authentication:Google:ClientSecret"] ?? "";
    });

// Real government API clients (FEMA, NOAA, FDOT, NIFC - all free, no auth required)
builder.Services.AddHttpClient<FemaApiClient>();
builder.Services.AddHttpClient<NoaaStormClient>();
builder.Services.AddHttpClient<FloridaFireDataClient>();
builder.Services.AddHttpClient<FloridaPropertyDataClient>();

builder.Services.AddScoped<PropertyService>();
builder.Services.AddScoped<PermitService>();
builder.Services.AddScoped<LandingPageService>();

var app = builder.Build();

// Seed with real FEMA disaster declarations for Florida
using (var scope = app.Services.CreateScope())
{
    var context = scope.ServiceProvider.GetRequiredService<AppDbContext>();
    await SeedData.InitializeAsync(context, scope.ServiceProvider);
}

if (!app.Environment.IsDevelopment())
{
    app.UseExceptionHandler("/Error");
    app.UseHsts();
}

app.UseHttpsRedirection();
app.UseStaticFiles();
app.UseRouting();
app.UseAuthentication();
app.UseAuthorization();

app.MapControllers();
app.MapControllerRoute(name: "default", pattern: "{controller}/{action}/{id?}");
app.MapBlazorHub();

// Blazor app pages (authenticated) - route these to the Blazor host
app.MapFallbackToPage("/dashboard", "/_Host");
app.MapFallbackToPage("/dashboard/{**path}", "/_Host");
app.MapFallbackToPage("/search", "/_Host");
app.MapFallbackToPage("/search/{**path}", "/_Host");
app.MapFallbackToPage("/property/{**path}", "/_Host");
app.MapFallbackToPage("/permits", "/_Host");
app.MapFallbackToPage("/permits/{**path}", "/_Host");
app.MapFallbackToPage("/risk", "/_Host");
app.MapFallbackToPage("/risk/{**path}", "/_Host");
app.MapFallbackToPage("/disasters", "/_Host");
app.MapFallbackToPage("/disasters/{**path}", "/_Host");
app.MapFallbackToPage("/api-status", "/_Host");
app.MapFallbackToPage("/api-status/{**path}", "/_Host");

app.Run();
