import csv
import math

import cbor2
import geopandas as gpd
from shapely.geometry.geo import mapping, shape
from shapely.lib import unary_union

geojson_file = "/Users/daryl/OSM/AllCountries.geojson"
gdf = gpd.read_file(geojson_file)


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


# Filter for only the United Kingdom proper
gdf = gdf[gdf["ISO_A3"] == "GBR"]

# Define the bounding box for the mainland UK
uk_bbox = {
    "minx": -13.2275390621,  # West extent
    "maxx": 8.3056640621,  # East extent
    "miny": 47.6357835912,  # South extent
    "maxy": 60.8449105734  # North extent
}


# Function to check if a geometry is within the UK bounding box
def within_uk_bbox(geometry):
    bounds = geometry.bounds  # Get (minx, miny, maxx, maxy)
    return (
            bounds[0] >= uk_bbox["minx"] and bounds[2] <= uk_bbox["maxx"] and
            bounds[1] >= uk_bbox["miny"] and bounds[3] <= uk_bbox["maxy"]
    )


# Ensure the GeoDataFrame is in OSGB (EPSG:27700)
# print("Converting to EPSG:27700")
# gdf = gdf.to_crs("EPSG:27700")

# Apply the bounding box filter
# gdf = gdf[gdf["geometry"].apply(within_uk_bbox)]

# Check if any geometries remain
if gdf.empty:
    print("Warning: No geometries remain after filtering. Check CRS and bounding box.")

# Convert to Web Mercator projection
gdf = gdf.to_crs("EPSG:3857")
# gdf = gdf.to_crs("EPSG:6933")  # Equal-area

# Apply rounding to all geometries
# gdf["geometry"] = gdf["geometry"].apply(lambda geom: round_coordinates(geom, precision=4))  # Reduce decimal places

# Apply geometry fixes (removes spurs)
# gdf["geometry"] = gdf["geometry"].apply(fix_geometries)

# Simplify each country's polygon while keeping properties
# tolerance = 1.0  # Adjust for more or less simplification
# gdf["geometry"] = gdf["geometry"].simplify(tolerance, preserve_topology=True)

output_file = "merged_by_region.geojson"

# Apply `unary_union` to truly merge touching geometries (removes internal borders)
gdf["geometry"] = gdf["geometry"].apply(lambda x: unary_union(x) if x else None)

# Save as a new GeoJSON file
gdf.to_file(output_file, driver="GeoJSON")

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

cities_out = []
with open("/Users/daryl/OSM/allCountries.txt") as f:
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

            if feature_class == "P" and feature_code in valid_feature_codes and country_code == "GB":
                latitude = float(latitude)
                longitude = float(longitude)
                if uk_bbox["minx"] <= longitude <= uk_bbox["maxx"] and uk_bbox["miny"] <= latitude <= uk_bbox["maxy"]:
                    x, y = mercator_projection(latitude, longitude)
                    # print("Match: ", name, x, y, latitude, longitude, population)#, countries[country_code])
                    cities_out.append([name, float(x) / 1000.0, float(y) / 1000.0, int(population)])  # , 
                    pop = pop + int(population)
                #        else:
                #            print(name, feature_class, feature_code, population)

# Sort by size descending
cities_out.sort(key=lambda x: x[3], reverse=True)

print("There are " + str(len(cities_out)))
cbor_data = cbor2.dumps(cities_out)
with open("Cities.cbor", 'wb') as o:
    o.write(cbor_data)
