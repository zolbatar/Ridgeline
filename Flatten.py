import geopandas as gpd

# Load the GeoJSON file
input_file = "simplified_countries.geojson"
gdf = gpd.read_file(input_file)

# 🔹 Choose a Projection That Reduces North-South Squishing
# Option 1: Robinson Projection (Good for world maps)
flattened_crs = "ESRI:54030"  # Robinson projection

# Option 2: Winkel Tripel (Another good compromise)
#flattened_crs = "ESRI:54042"

# Option 3: Eckert IV (More evenly distributed distortion)
#flattened_crs = "ESRI:54012"

# Convert the projection
gdf = gdf.to_crs(flattened_crs)

# Save the new GeoJSON
output_file = "flattened_countries.geojson"
gdf.to_file(output_file, driver="GeoJSON")

print(f"✅ Flattened projection saved as '{output_file}'")