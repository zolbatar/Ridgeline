import geopandas as gpd
from shapely.geometry import box

# Load the GeoJSON file
geojson_file = "/Users/daryl/OSM/AllCountries.geojson"
gdf = gpd.read_file(geojson_file)

# Define latitude crop range (adjust as needed)
min_lat = -60  # 5th percentile latitude (adjust if necessary)
max_lat = 80  # 95th percentile latitude
min_lon = -140  # 5th percentile latitude (adjust if necessary)
max_lon = 175  # 95th percentile latitude

# Create a bounding box (min_x, min_y, max_x, max_y)
bbox = box(min_lon, min_lat, max_lon, max_lat)

# Clip the GeoDataFrame
gdf_clipped = gpd.clip(gdf, bbox)  # gdf[gdf.intersects(bbox)]

# Save the cropped GeoJSON
output_file = "cropped_file.geojson"
gdf_clipped.to_file(output_file, driver="GeoJSON")

print(f"Saved cropped GeoJSON to {output_file}")
