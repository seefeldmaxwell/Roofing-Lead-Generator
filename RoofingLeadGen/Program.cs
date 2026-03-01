using Microsoft.EntityFrameworkCore;
using RoofingLeadGen.Data;
using RoofingLeadGen.Services;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddRazorPages();
builder.Services.AddServerSideBlazor();
builder.Services.AddControllers()
    .AddJsonOptions(opts =>
    {
        opts.JsonSerializerOptions.ReferenceHandler = System.Text.Json.Serialization.ReferenceHandler.IgnoreCycles;
        opts.JsonSerializerOptions.DefaultIgnoreCondition = System.Text.Json.Serialization.JsonIgnoreCondition.WhenWritingNull;
    });

builder.Services.AddDbContext<AppDbContext>(options =>
    options.UseSqlite(builder.Configuration.GetConnectionString("DefaultConnection")
        ?? "Data Source=RoofingLeadGen.db"));

// Real government API clients (FEMA, NOAA, FDOT, NIFC - all free, no auth required)
builder.Services.AddHttpClient<FemaApiClient>();
builder.Services.AddHttpClient<NoaaStormClient>();
builder.Services.AddHttpClient<FloridaFireDataClient>();
builder.Services.AddHttpClient<FloridaPropertyDataClient>();

builder.Services.AddScoped<PropertyService>();
builder.Services.AddScoped<PermitService>();

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

app.MapControllers();
app.MapBlazorHub();
app.MapFallbackToPage("/_Host");

app.Run();
