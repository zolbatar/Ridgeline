import geopandas as gpd
from shapely.geometry import mapping, shape
from shapely.ops import unary_union

# Load the GeoJSON file
geojson_file = "/Users/daryl/OSM/AllCountries.geojson"
gdf = gpd.read_file(geojson_file)

# Define function to shift longitudes properly
def fix_russia_longitudes(geometry):
    """ Adds 360° to negative longitude values for Russia """
    if geometry.is_empty:
        return geometry

    geojson_geom = mapping(geometry)  # Convert to GeoJSON format
    coords_key = "coordinates" if "coordinates" in geojson_geom else None

    if coords_key:
        def adjust_coords(coords):
            return [(lon + 360 if lon < 0 else lon, lat) for lon, lat in coords]

        def process_geometry(geom):
            if isinstance(geom[0], (list, tuple)):  # MultiPolygon or Polygon
                return [process_geometry(part) for part in geom] if isinstance(geom[0][0], (list, tuple)) else adjust_coords(geom)
            return geom

        geojson_geom["coordinates"] = process_geometry(geojson_geom["coordinates"])

    return shape(geojson_geom)  # Convert back to Shapely geometry

# Fix Russia's longitudes first
gdf.loc[gdf["ADM0_A3"] == "RUS", "geometry"] = gdf.loc[gdf["ADM0_A3"] == "RUS", "geometry"].apply(fix_russia_longitudes)

# Merge all Russia's polygons into one contiguous shape
russia_gdf = gdf[gdf["ADM0_A3"] == "RUS"]
merged_russia = unary_union(russia_gdf.geometry)

# Update Russia in the GeoDataFrame
gdf.loc[gdf["ADM0_A3"] == "RUS", "geometry"] = merged_russia

# Simplify each country's polygon while keeping properties
tolerance = 0.1  # Adjust for more or less simplification
gdf["geometry"] = gdf["geometry"].simplify(tolerance, preserve_topology=True)

# Save the new simplified GeoJSON with properties retained
gdf.to_file("simplified_countries.geojson", driver="GeoJSON")

print("Simplified GeoJSON saved as 'simplified_countries.geojson'")
