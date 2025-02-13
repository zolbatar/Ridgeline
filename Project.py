import csv
import math
import cbor2
import geopandas as gpd
import pandas as pd
from shapely.affinity import scale
from shapely.geometry.geo import mapping, shape, box
from shapely.lib import unary_union

geojson_file = "/Users/daryl/OSM/AllCountries.geojson"
gdf = gpd.read_file(geojson_file)

# Extract ISO_A2 and SUBREGION as a dictionary
country_subregion_dict = dict(zip(gdf["ISO_A2"], gdf["SUBREGION"]))

# Define regions to exclude (example: Micronesia)
excluded_regions = ["Micronesia", "Polynesia", "Melanesia"]

# Filter out the excluded regions
filtered_gdf = gdf[~gdf["SUBREGION"].isin(excluded_regions)]


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


def round_remove_digits(num, digits):
    factor = 10 ** digits  # Compute the factor to remove precision
    return round(num / factor) * factor  # Round and multiply back


# Define a function to round coordinates
def round_coordinates(geometry, precision=4):
    """ Recursively rounds coordinates in a geometry object. """

    def round_point(point):
        return [round_remove_digits(coord, precision) for coord in point]

    def process_coords(coords):
        if isinstance(coords[0], (list, tuple)):  # Multi-point structures
            return [process_coords(subcoords) for subcoords in coords]
        return round_point(coords)  # Single coordinate pair

    geojson_geom = mapping(geometry)  # Convert to GeoJSON-like dictionary
    geojson_geom["coordinates"] = process_coords(geojson_geom["coordinates"])
    return shape(geojson_geom)  # Convert back to Shapely geometry


# Function to enforce valid polygons and remove spurs
def fix_geometries(geometry, min_area=1e-6):
    """ Removes spurs, fixes self-intersections, and enforces polygon structures. """
    geometry = geometry.buffer(0)  # Fix self-intersections

    # Remove small spurs or degenerate polygons
    if geometry.geom_type in ["Polygon", "MultiPolygon"]:
        if geometry.area < min_area:  # Remove tiny polygons
            return None
        return geometry

    return None  # Remove any remaining invalid geometry


# Fix Russia's longitudes first
gdf.loc[gdf["ADM0_A3"] == "RUS", "geometry"] = gdf.loc[gdf["ADM0_A3"] == "RUS", "geometry"].apply(fix_russia_longitudes)

# Merge all Russia's polygons into one contiguous shape
russia_gdf = gdf[gdf["ADM0_A3"] == "RUS"]
merged_russia = unary_union(russia_gdf.geometry)

# Update Russia in the GeoDataFrame
gdf.loc[gdf["ADM0_A3"] == "RUS", "geometry"] = merged_russia

# Define latitude crop range (adjust as needed)
min_lat = -80  # 5th percentile latitude (adjust if necessary)
max_lat = 60  # 95th percentile latitude
min_lon = -200  # 5th percentile latitude (adjust if necessary)
max_lon = 200  # 95th percentile latitude

# Create a bounding box (min_x, min_y, max_x, max_y)
# bbox = box(min_lon, min_lat, max_lon, max_lat)

# Clip the GeoDataFrame
# gdf = gpd.clip(gdf, bbox)  # gdf[gdf.intersects(bbox)]

gdf["geometry"] = gdf["geometry"].apply(lambda geom: scale(geom, xfact=0.92, yfact=1.0, origin=(0, 0)))

# Convert to Web Mercator projection
gdf = gdf.to_crs("EPSG:3857")
# gdf = gdf.to_crs("EPSG:6933")  # Equal-area

# Apply rounding to all geometries
gdf["geometry"] = gdf["geometry"].apply(lambda geom: round_coordinates(geom, precision=1))  # Reduce decimal places

# Apply geometry fixes (removes spurs)
gdf["geometry"] = gdf["geometry"].apply(fix_geometries)

# Simplify each country's polygon while keeping properties
tolerance = 1.0  # Adjust for more or less simplification
gdf["geometry"] = gdf["geometry"].simplify(tolerance, preserve_topology=True)

csv_mapping_file = "all.csv"
output_file = "merged_by_region.geojson"

# Load the CSV mapping (ADM3 -> region_id)
csv_df = pd.read_csv(csv_mapping_file)

# Ensure column names match (GeoJSON uses `ADM0_A3` for country codes)
csv_df.rename(columns={"alpha-3": "ADM0_A3", "sub-region-code": "region_id"}, inplace=True)

# Merge `region_id` into the GeoDataFrame based on `alpha-3` country codes
gdf = gdf.merge(csv_df[["ADM0_A3", "region_id"]], on="ADM0_A3", how="left")

# Drop rows where region_id is missing
gdf = gdf[gdf["region_id"].notnull()]

# Group by `region_id` and fully dissolve all geometries
merged_gdf = gdf.dissolve(by="region_id", aggfunc="first")

# Apply `unary_union` to truly merge touching geometries (removes internal borders)
merged_gdf["geometry"] = merged_gdf["geometry"].apply(lambda x: unary_union(x) if x else None)

# Remove unnecessary columns, keep only `region_id`
merged_gdf = merged_gdf.reset_index()[["region_id", "geometry"]]

# Save as a new GeoJSON file
merged_gdf.to_file(output_file, driver="GeoJSON")

print(f"Merged and dissolved GeoJSON saved as '{output_file}' with `region_id` as the only property.")


def mercator_projection(lat, lon):
    """
    Converts latitude and longitude into Mercator projection coordinates.

    :param lat: Latitude in decimal degrees
    :param lon: Longitude in decimal degrees
    :return: (x, y) Mercator projection coordinates
    """
    # Convert degrees to radians
    lat_rad = math.radians(lat)
    lon_rad = math.radians(lon)

    # Mercator projection formula
    x = EARTH_RADIUS * lon_rad
    y = EARTH_RADIUS * math.log(math.tan(math.pi / 4 + lat_rad / 2))

    return x, y


# Constants
EARTH_RADIUS = 6378137  # WGS84 radius in meters

# Define valid feature codes for populated places
valid_feature_codes = {"PPLA", "PPLA2", "PPLA3", "PPLA4", "PPLL", "PPLC", "PPLS", "PPL"}

# Define excluded country codes
excluded_countries = {"FM", "PW", "MH", "PF", "AS", "CK", "KI", "TO"}  # Example: Micronesia, Palau, Marshall Islands

# Convert country mapping to a dictionary for fast lookup
country_to_region = csv_df.set_index("alpha-2")["region_id"].to_dict()

cities_out = []
with open("/Users/daryl/OSM//allCountries.txt") as f:
    reader = csv.reader(f, delimiter="\t")
    pop = 0
    for row in reader:
        if len(row) != 19:
            print("Invalid number of columns: " + str(len(row)))
            print('|| '.join(row))
        else:
            geonameid = row[0]
            name = row[1]
            asciiname = row[2]
            alternames = row[3]
            latitude = row[4]
            longitude = row[5]
            feature_class = row[6]
            feature_code = row[7]
            country_code = row[8]
            cc2 = row[9]
            admin1_code = row[10]
            admin2_code = row[11]
            admin3_code = row[12]
            admin4_code = row[13]
            population = row[14]
            elevation = row[15]
            dem = row[16]
            timezone = row[17]
            mod_date = row[18]

            # Get region_id using country_code lookup, default to None if not found
            region_id = country_to_region.get(country_code, None)
            
            if region_id is not None and feature_class == "P" and feature_code in valid_feature_codes and country_code not in excluded_countries and int(population) > 25000:
                x, y = mercator_projection(float(latitude), float(longitude))
                # print("Match: ", name, x, y, latitude, longitude, population)#, countries[country_code])
                cities_out.append([int(region_id), name, float(x) * 0.92 / 1000.0, float(y) / 1000.0, int(population)])  # , countries[country_code]])
                pop = pop + int(population)
                #        else:
                #            print(name, feature_class, feature_code, population)

# Sort by size descending
cities_out.sort(key=lambda x: x[4], reverse=True)

print("There are " + str(len(cities_out)))
cbor_data = cbor2.dumps(cities_out)
with open("Cities.cbor", 'wb') as o:
    o.write(cbor_data)
