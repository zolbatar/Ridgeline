import csv
import math

import cbor2
import geopandas as gpd
from pyproj import Transformer
from shapely.geometry.geo import mapping, shape
from shapely.lib import unary_union
import matplotlib.pyplot as plt
from shapely.ops import transform

geojson_file = "/Users/daryl/OSM/AllCountries.geojson"
gdf = gpd.read_file(geojson_file)

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


# Apply the bounding box filter
# gdf = gdf[gdf["geometry"].apply(within_uk_bbox)]

# Check if any geometries remain
if gdf.empty:
    print("Warning: No geometries remain after filtering. Check CRS and bounding box.")

#fig, ax = plt.subplots(1, 2, figsize=(12, 6))

# Plot original (WGS84)
gdf.plot(ax=ax[0], color="blue", edgecolor="black")
#ax[0].set_title("Before Reprojection (WGS84)")

# Define the transformer from WGS84 to British National Grid
transformer = Transformer.from_crs("EPSG:4326", "EPSG:27700", always_xy=True)


# Function to apply transformation
def reproject_geom(geom):
    return transform(transformer.transform, geom)


# Apply the transformation manually
gdf["geometry"] = gdf["geometry"].apply(reproject_geom)
gdf = gdf.set_crs("EPSG:27700", allow_override=True)

# Plot transformed (British National Grid)
#gdf.plot(ax=ax[1], color="red", edgecolor="black")
#ax[1].set_title("After Reprojection (British National Grid)")

# plt.show()

output_file = "merged_by_region.geojson"

# Apply `unary_union` to truly merge touching geometries (removes internal borders)
# gdf["geometry"] = gdf["geometry"].apply(lambda x: unary_union(x) if x else None)

# Save as a new GeoJSON file
gdf.to_file(output_file, driver="GeoJSON")

print(f"Merged and dissolved GeoJSON saved as '{output_file}' with `region_id` as the only property.")

# Define valid feature codes for populated places
valid_feature_codes = {"PPLA", "PPLA2", "PPLA3", "PPLA4", "PPLL", "PPLC", "PPLS", "PPL"}


def latlon_to_osgb(lat, lon):
    """
    Converts WGS84 (latitude, longitude) to OSGB36 (easting, northing).
    
    Args:
        lat (float): Latitude in decimal degrees.
        lon (float): Longitude in decimal degrees.

    Returns:
        tuple: (easting, northing) in OSGB36 coordinates.
    """
    # Define transformation from EPSG:4326 (WGS84) to EPSG:27700 (OSGB36)
    transformer = Transformer.from_crs("EPSG:4326", "EPSG:27700", always_xy=True)

    # Perform the transformation
    easting, northing = transformer.transform(lon, lat)
    return easting, northing


cities_out = []

with open("/Users/daryl/OSM/allCountries.txt") as f:
    reader = csv.reader(f, delimiter="\t")
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
                if int(population) > 500 and uk_bbox["minx"] <= longitude <= uk_bbox["maxx"] and uk_bbox[
                    "miny"] <= latitude <= uk_bbox["maxy"]:
                    x, y = latlon_to_osgb(latitude, longitude)
                    # print("Match: ", name, x, y, latitude, longitude, population)  # , countries[country_code])
                    cities_out.append([name, float(x), float(y), int(population)])
            #        else:
            #            print(name, feature_class, feature_code, population)

# Sort by size descending
# cities_out.sort(key=lambda x: x[3], reverse=True)

print("There are " + str(len(cities_out)))
cbor_data = cbor2.dumps(cities_out)
with open("Cities.cbor", 'wb') as o:
    o.write(cbor_data)
