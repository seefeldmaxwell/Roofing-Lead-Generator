using RoofingLeadGen.Models;

namespace RoofingLeadGen.Data;

public static class SeedData
{
    public static void Initialize(AppDbContext context)
    {
        context.Database.EnsureCreated();

        if (context.Properties.Any())
            return;

        var owners = new List<Owner>
        {
            new() { FirstName = "James", LastName = "Rodriguez", MailingAddress = "1420 SW 3rd Ave", MailingCity = "Miami", MailingState = "FL", MailingZip = "33130", PhoneNumber = "(305) 555-0142", Email = "j.rodriguez@email.com", Age = 54, DateOfBirth = new DateTime(1971, 6, 15) },
            new() { FirstName = "Maria", LastName = "Gonzalez", MailingAddress = "2850 NW 72nd Ave", MailingCity = "Miami", MailingState = "FL", MailingZip = "33122", PhoneNumber = "(305) 555-0287", Email = "mgonzalez@email.com", Age = 47, DateOfBirth = new DateTime(1978, 3, 22) },
            new() { FirstName = "Robert", LastName = "Thompson", MailingAddress = "540 E Las Olas Blvd", MailingCity = "Fort Lauderdale", MailingState = "FL", MailingZip = "33301", PhoneNumber = "(954) 555-0193", Email = "rthompson@email.com", Age = 62, DateOfBirth = new DateTime(1963, 11, 8) },
            new() { FirstName = "Patricia", LastName = "Williams", MailingAddress = "1200 S Pine Island Rd", MailingCity = "Plantation", MailingState = "FL", MailingZip = "33324", PhoneNumber = "(954) 555-0341", Email = "pwilliams@email.com", Age = 58, DateOfBirth = new DateTime(1967, 9, 3) },
            new() { FirstName = "Michael", LastName = "Chen", MailingAddress = "3400 Coral Way", MailingCity = "Miami", MailingState = "FL", MailingZip = "33145", PhoneNumber = "(305) 555-0456", Email = "mchen@email.com", Age = 41, DateOfBirth = new DateTime(1984, 7, 19) },
            new() { FirstName = "Jennifer", LastName = "Davis", MailingAddress = "8900 W Commercial Blvd", MailingCity = "Sunrise", MailingState = "FL", MailingZip = "33351", PhoneNumber = "(954) 555-0512", Email = "jdavis@email.com", Age = 36, DateOfBirth = new DateTime(1989, 12, 1) },
            new() { FirstName = "David", LastName = "Martinez", MailingAddress = "1500 N Congress Ave", MailingCity = "West Palm Beach", MailingState = "FL", MailingZip = "33401", PhoneNumber = "(561) 555-0678", Email = "dmartinez@email.com", Age = 51, DateOfBirth = new DateTime(1974, 4, 25) },
            new() { FirstName = "Linda", LastName = "Johnson", MailingAddress = "7200 N Federal Hwy", MailingCity = "Boca Raton", MailingState = "FL", MailingZip = "33487", PhoneNumber = "(561) 555-0789", Email = "ljohnson@email.com", Age = 67, DateOfBirth = new DateTime(1958, 8, 14) },
            new() { FirstName = "William", LastName = "Anderson", MailingAddress = "4500 PGA Blvd", MailingCity = "Palm Beach Gardens", MailingState = "FL", MailingZip = "33410", PhoneNumber = "(561) 555-0834", Email = "wanderson@email.com", Age = 73, DateOfBirth = new DateTime(1952, 2, 28) },
            new() { FirstName = "Barbara", LastName = "Taylor", MailingAddress = "920 N Orlando Ave", MailingCity = "Winter Park", MailingState = "FL", MailingZip = "32789", PhoneNumber = "(407) 555-0921", Email = "btaylor@email.com", Age = 44, DateOfBirth = new DateTime(1981, 10, 7) },
            new() { FirstName = "Richard", LastName = "Moore", MailingAddress = "3100 S Orange Ave", MailingCity = "Orlando", MailingState = "FL", MailingZip = "32806", PhoneNumber = "(407) 555-0135", Email = "rmoore@email.com", Age = 59, DateOfBirth = new DateTime(1966, 5, 30) },
            new() { FirstName = "Susan", LastName = "Clark", MailingAddress = "6700 N Dale Mabry Hwy", MailingCity = "Tampa", MailingState = "FL", MailingZip = "33614", PhoneNumber = "(813) 555-0246", Email = "sclark@email.com", Age = 52, DateOfBirth = new DateTime(1973, 1, 16) },
            new() { FirstName = "Joseph", LastName = "Lewis", MailingAddress = "2100 Gulf to Bay Blvd", MailingCity = "Clearwater", MailingState = "FL", MailingZip = "33765", PhoneNumber = "(727) 555-0357", Email = "jlewis@email.com", Age = 45, DateOfBirth = new DateTime(1980, 6, 22) },
            new() { FirstName = "Karen", LastName = "Walker", MailingAddress = "1800 Main St", MailingCity = "Sarasota", MailingState = "FL", MailingZip = "34236", PhoneNumber = "(941) 555-0468", Email = "kwalker@email.com", Age = 71, DateOfBirth = new DateTime(1954, 3, 11) },
            new() { FirstName = "Thomas", LastName = "Hall", MailingAddress = "550 S Tamiami Trail", MailingCity = "Naples", MailingState = "FL", MailingZip = "34102", PhoneNumber = "(239) 555-0579", Email = "thall@email.com", Age = 68, DateOfBirth = new DateTime(1957, 9, 27) },
            new() { FirstName = "Nancy", LastName = "Allen", MailingAddress = "3200 Atlantic Blvd", MailingCity = "Jacksonville", MailingState = "FL", MailingZip = "32207", PhoneNumber = "(904) 555-0681", Email = "nallen@email.com", Age = 39, DateOfBirth = new DateTime(1986, 11, 4) },
            new() { FirstName = "Daniel", LastName = "Young", MailingAddress = "1100 Thomasville Rd", MailingCity = "Tallahassee", MailingState = "FL", MailingZip = "32303", PhoneNumber = "(850) 555-0792", Email = "dyoung@email.com", Age = 33, DateOfBirth = new DateTime(1992, 7, 18) },
            new() { FirstName = "Margaret", LastName = "King", MailingAddress = "4800 Bayshore Blvd", MailingCity = "Tampa", MailingState = "FL", MailingZip = "33611", PhoneNumber = "(813) 555-0813", Email = "mking@email.com", Age = 56, DateOfBirth = new DateTime(1969, 4, 9) },
            new() { FirstName = "Christopher", LastName = "Wright", MailingAddress = "2500 Weston Rd", MailingCity = "Weston", MailingState = "FL", MailingZip = "33331", PhoneNumber = "(954) 555-0924", Email = "cwright@email.com", Age = 48, DateOfBirth = new DateTime(1977, 8, 21) },
            new() { FirstName = "Dorothy", LastName = "Lopez", MailingAddress = "1900 SW 8th St", MailingCity = "Miami", MailingState = "FL", MailingZip = "33135", PhoneNumber = "(305) 555-0135", Email = "dlopez@email.com", Age = 63, DateOfBirth = new DateTime(1962, 12, 30) },
        };

        context.Owners.AddRange(owners);
        context.SaveChanges();

        var properties = new List<Property>
        {
            new() { Address = "1420 SW 3rd Ave", City = "Miami", ZipCode = "33130", County = "Miami-Dade", ParcelId = "01-3120-045-0010", YearBuilt = 1985, SquareFootage = 2100, LotSize = 0.18, PropertyType = "Single Family", EstimatedValue = 485000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Shingle", RoofAge = 18, LastRoofPermitDate = new DateTime(2007, 8, 15), Latitude = 25.7617, Longitude = -80.1918, OwnerId = owners[0].Id },
            new() { Address = "2850 NW 72nd Ave", City = "Miami", ZipCode = "33122", County = "Miami-Dade", ParcelId = "01-3126-012-0030", YearBuilt = 1992, SquareFootage = 1800, LotSize = 0.14, PropertyType = "Single Family", EstimatedValue = 395000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Tile", RoofAge = 12, LastRoofPermitDate = new DateTime(2013, 5, 20), Latitude = 25.7796, Longitude = -80.3180, OwnerId = owners[1].Id },
            new() { Address = "540 E Las Olas Blvd", City = "Fort Lauderdale", ZipCode = "33301", County = "Broward", ParcelId = "50-42-16-01-0010", YearBuilt = 2001, SquareFootage = 3200, LotSize = 0.25, PropertyType = "Single Family", EstimatedValue = 875000m, Bedrooms = 4, Bathrooms = 3, RoofType = "Metal", RoofAge = 8, LastRoofPermitDate = new DateTime(2017, 11, 3), Latitude = 26.1185, Longitude = -80.1290, OwnerId = owners[2].Id },
            new() { Address = "1200 S Pine Island Rd", City = "Plantation", ZipCode = "33324", County = "Broward", ParcelId = "50-41-12-03-0050", YearBuilt = 1978, SquareFootage = 2400, LotSize = 0.22, PropertyType = "Single Family", EstimatedValue = 420000m, Bedrooms = 4, Bathrooms = 2, RoofType = "Shingle", RoofAge = 22, LastRoofPermitDate = new DateTime(2003, 6, 10), Latitude = 26.1224, Longitude = -80.2331, OwnerId = owners[3].Id },
            new() { Address = "3400 Coral Way", City = "Miami", ZipCode = "33145", County = "Miami-Dade", ParcelId = "01-4130-008-0020", YearBuilt = 2010, SquareFootage = 1600, LotSize = 0.12, PropertyType = "Townhouse", EstimatedValue = 520000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Flat/Built-Up", RoofAge = 5, LastRoofPermitDate = new DateTime(2020, 9, 1), Latitude = 25.7489, Longitude = -80.2456, OwnerId = owners[4].Id },
            new() { Address = "8900 W Commercial Blvd", City = "Sunrise", ZipCode = "33351", County = "Broward", ParcelId = "50-42-09-02-0080", YearBuilt = 1988, SquareFootage = 1950, LotSize = 0.16, PropertyType = "Single Family", EstimatedValue = 365000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Shingle", RoofAge = 19, LastRoofPermitDate = new DateTime(2006, 4, 22), Latitude = 26.1884, Longitude = -80.2556, OwnerId = owners[5].Id },
            new() { Address = "1500 N Congress Ave", City = "West Palm Beach", ZipCode = "33401", County = "Palm Beach", ParcelId = "74-43-44-10-01-000", YearBuilt = 1995, SquareFootage = 2800, LotSize = 0.30, PropertyType = "Single Family", EstimatedValue = 550000m, Bedrooms = 4, Bathrooms = 3, RoofType = "Tile", RoofAge = 15, LastRoofPermitDate = new DateTime(2010, 12, 8), Latitude = 26.7153, Longitude = -80.0664, OwnerId = owners[6].Id },
            new() { Address = "7200 N Federal Hwy", City = "Boca Raton", ZipCode = "33487", County = "Palm Beach", ParcelId = "74-43-47-20-03-000", YearBuilt = 1972, SquareFootage = 3500, LotSize = 0.35, PropertyType = "Single Family", EstimatedValue = 780000m, Bedrooms = 5, Bathrooms = 3, RoofType = "Tile", RoofAge = 25, LastRoofPermitDate = new DateTime(2000, 7, 14), Latitude = 26.3683, Longitude = -80.0729, OwnerId = owners[7].Id },
            new() { Address = "4500 PGA Blvd", City = "Palm Beach Gardens", ZipCode = "33410", County = "Palm Beach", ParcelId = "74-42-41-08-05-000", YearBuilt = 1998, SquareFootage = 4200, LotSize = 0.45, PropertyType = "Single Family", EstimatedValue = 1250000m, Bedrooms = 5, Bathrooms = 4, RoofType = "Tile", RoofAge = 14, LastRoofPermitDate = new DateTime(2011, 3, 19), Latitude = 26.8434, Longitude = -80.0942, OwnerId = owners[8].Id },
            new() { Address = "920 N Orlando Ave", City = "Winter Park", ZipCode = "32789", County = "Orange", ParcelId = "25-22-29-01-0010", YearBuilt = 2005, SquareFootage = 2600, LotSize = 0.20, PropertyType = "Single Family", EstimatedValue = 610000m, Bedrooms = 4, Bathrooms = 3, RoofType = "Shingle", RoofAge = 10, LastRoofPermitDate = new DateTime(2015, 6, 25), Latitude = 28.5999, Longitude = -81.3479, OwnerId = owners[9].Id },
            new() { Address = "3100 S Orange Ave", City = "Orlando", ZipCode = "32806", County = "Orange", ParcelId = "25-23-30-02-0040", YearBuilt = 1982, SquareFootage = 2200, LotSize = 0.19, PropertyType = "Single Family", EstimatedValue = 380000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Shingle", RoofAge = 20, LastRoofPermitDate = new DateTime(2005, 10, 12), Latitude = 28.5105, Longitude = -81.3789, OwnerId = owners[10].Id },
            new() { Address = "6700 N Dale Mabry Hwy", City = "Tampa", ZipCode = "33614", County = "Hillsborough", ParcelId = "A-28-29-18-4FR-000001", YearBuilt = 1990, SquareFootage = 2000, LotSize = 0.17, PropertyType = "Single Family", EstimatedValue = 430000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Shingle", RoofAge = 16, LastRoofPermitDate = new DateTime(2009, 2, 28), Latitude = 28.0137, Longitude = -82.5074, OwnerId = owners[11].Id },
            new() { Address = "2100 Gulf to Bay Blvd", City = "Clearwater", ZipCode = "33765", County = "Pinellas", ParcelId = "15-29-16-00000-230-0100", YearBuilt = 2003, SquareFootage = 1700, LotSize = 0.13, PropertyType = "Single Family", EstimatedValue = 390000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Metal", RoofAge = 7, LastRoofPermitDate = new DateTime(2018, 8, 5), Latitude = 27.9576, Longitude = -82.7746, OwnerId = owners[12].Id },
            new() { Address = "1800 Main St", City = "Sarasota", ZipCode = "34236", County = "Sarasota", ParcelId = "2019-0-0003", YearBuilt = 1968, SquareFootage = 2900, LotSize = 0.28, PropertyType = "Single Family", EstimatedValue = 650000m, Bedrooms = 4, Bathrooms = 3, RoofType = "Tile", RoofAge = 28, LastRoofPermitDate = new DateTime(1997, 11, 20), Latitude = 27.3364, Longitude = -82.5307, OwnerId = owners[13].Id },
            new() { Address = "550 S Tamiami Trail", City = "Naples", ZipCode = "34102", County = "Collier", ParcelId = "61844240008", YearBuilt = 1975, SquareFootage = 3800, LotSize = 0.40, PropertyType = "Single Family", EstimatedValue = 920000m, Bedrooms = 5, Bathrooms = 4, RoofType = "Tile", RoofAge = 24, LastRoofPermitDate = new DateTime(2001, 4, 3), Latitude = 26.1420, Longitude = -81.7948, OwnerId = owners[14].Id },
            new() { Address = "3200 Atlantic Blvd", City = "Jacksonville", ZipCode = "32207", County = "Duval", ParcelId = "073572-0000", YearBuilt = 2008, SquareFootage = 1900, LotSize = 0.15, PropertyType = "Single Family", EstimatedValue = 340000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Shingle", RoofAge = 9, LastRoofPermitDate = new DateTime(2016, 7, 30), Latitude = 30.3004, Longitude = -81.6240, OwnerId = owners[15].Id },
            new() { Address = "1100 Thomasville Rd", City = "Tallahassee", ZipCode = "32303", County = "Leon", ParcelId = "21-20-3-000-0001-0", YearBuilt = 2015, SquareFootage = 1500, LotSize = 0.11, PropertyType = "Townhouse", EstimatedValue = 285000m, Bedrooms = 3, Bathrooms = 2, RoofType = "Shingle", RoofAge = 4, LastRoofPermitDate = new DateTime(2021, 1, 15), Latitude = 30.4586, Longitude = -84.2807, OwnerId = owners[16].Id },
            new() { Address = "4800 Bayshore Blvd", City = "Tampa", ZipCode = "33611", County = "Hillsborough", ParcelId = "A-15-30-18-2Z7-000002", YearBuilt = 1994, SquareFootage = 3100, LotSize = 0.32, PropertyType = "Single Family", EstimatedValue = 720000m, Bedrooms = 4, Bathrooms = 3, RoofType = "Metal", RoofAge = 11, LastRoofPermitDate = new DateTime(2014, 5, 18), Latitude = 27.8868, Longitude = -82.4876, OwnerId = owners[17].Id },
            new() { Address = "2500 Weston Rd", City = "Weston", ZipCode = "33331", County = "Broward", ParcelId = "50-41-35-01-0200", YearBuilt = 2000, SquareFootage = 2700, LotSize = 0.24, PropertyType = "Single Family", EstimatedValue = 595000m, Bedrooms = 4, Bathrooms = 3, RoofType = "Tile", RoofAge = 13, LastRoofPermitDate = new DateTime(2012, 10, 9), Latitude = 26.1004, Longitude = -80.3998, OwnerId = owners[18].Id },
            new() { Address = "1900 SW 8th St", City = "Miami", ZipCode = "33135", County = "Miami-Dade", ParcelId = "01-4128-033-0060", YearBuilt = 1965, SquareFootage = 1400, LotSize = 0.10, PropertyType = "Single Family", EstimatedValue = 350000m, Bedrooms = 2, Bathrooms = 1, RoofType = "Shingle", RoofAge = 30, LastRoofPermitDate = new DateTime(1995, 3, 7), Latitude = 25.7655, Longitude = -80.2270, OwnerId = owners[19].Id },
        };

        context.Properties.AddRange(properties);
        context.SaveChanges();

        var permits = new List<Permit>();
        var random = new Random(42);
        var permitTypes = new[] { "Roofing", "Roofing - Re-Roof", "Roofing - Repair", "Building", "Electrical", "Plumbing", "HVAC", "Window Replacement", "Impact Windows", "Solar Panel Installation" };
        var statuses = new[] { "Completed", "Final", "Approved", "Expired", "In Progress" };
        var contractors = new[] { "ABC Roofing Inc.", "Sunshine Roof & Repair", "Florida Pro Roofing", "Guardian Roofing LLC", "All-Weather Roofing Co.", "Coastal Roofing Solutions", "Palm State Contractors", "Elite Home Services", "Bay Area Roofing", "First Choice Roofing" };

        int permitNum = 1000;
        foreach (var property in properties)
        {
            int numPermits = random.Next(2, 7);
            for (int i = 0; i < numPermits; i++)
            {
                var isRoofing = i == 0 || random.NextDouble() > 0.6;
                var permitType = isRoofing ? permitTypes[random.Next(0, 3)] : permitTypes[random.Next(3, permitTypes.Length)];
                var yearOffset = random.Next(0, 25);
                var issuedDate = DateTime.Now.AddYears(-yearOffset).AddDays(-random.Next(0, 365));
                var contractor = contractors[random.Next(contractors.Length)];

                permits.Add(new Permit
                {
                    PermitNumber = $"FLP-{DateTime.Now.Year - yearOffset}-{permitNum++:D6}",
                    PermitType = permitType,
                    Description = isRoofing ? $"Complete {property.RoofType?.ToLower()} roof replacement - {property.SquareFootage} sq ft" : $"{permitType} work at {property.Address}",
                    IssuedDate = issuedDate,
                    CompletedDate = issuedDate.AddDays(random.Next(14, 120)),
                    ExpirationDate = issuedDate.AddYears(1),
                    Status = statuses[random.Next(statuses.Length)],
                    ContractorName = contractor,
                    ContractorLicense = $"CCC{random.Next(100000, 999999)}",
                    EstimatedCost = isRoofing ? random.Next(8000, 35000) : random.Next(2000, 15000),
                    IsRoofingPermit = isRoofing,
                    PropertyId = property.Id
                });
            }
        }

        context.Permits.AddRange(permits);
        context.SaveChanges();
    }
}
