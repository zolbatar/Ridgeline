use crate::geo::boundary::{create_boundaries, load_boundaries};
use crate::geo::cities::load_cities_cbor_file;
use crate::geo::data::{Geo, GeoWithPathAndCities};
use crate::geo::ways::{load_ways, categorise_ways, serialize_ways};
use crate::gfx::skia::load_image_from_file;
use geo::Geometry;
use geojson::GeoJson;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub const RATIO_ADJUST: f32 = 1000.0;

pub fn create_geo() {
    //load_geojson();
    //create_ways();
    //create_boundaries();
    let ways = categorise_ways();
    serialize_ways(ways).expect("Unable to serialize Ways");
}

fn load_geojson() {
    // Open and read the .geojson file
    let file = File::open("data/merged_by_region.geojson").expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Parse the files as GeoJSON
    let geojson: GeoJson = serde_json::from_reader(reader).expect("Unable to read geojson");

    // Extract features (country boundaries)
    let mut m = Vec::new();
    let mut count = 0usize;
    if let GeoJson::FeatureCollection(fc) = geojson {
        for feature in fc.features {
            if let Some(geometry) = feature.geometry {
                let geo_geometry: geo::Geometry<f64> = geometry.try_into().unwrap();

                // We don't want the islands
                let extracted = match geo_geometry {
                    Geometry::Polygon(polygon) => vec![polygon],              // Single polygon
                    Geometry::MultiPolygon(multi_polygon) => multi_polygon.0, // Multiple polygons
                    _ => panic!("Unsupported geo type"),
                };

                // Go through each polygon and decide if we want it
                let mut v = Vec::new();
                for poly in extracted.into_iter() {
                    v.push(poly);
                    count += 1;
                }
                m.push(Geo {
                    geo: v,
                });
            }
        }
    }
    println!("Polygon count: {}", count);

    let file = File::create("data/Geo.cbor").unwrap();
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m).unwrap();
}

pub fn load(radius: f64) -> Result<GeoWithPathAndCities, Box<dyn Error>> {
    let cities = load_cities_cbor_file("data/Cities.cbor", radius);
    let ways = load_ways();
    let boundaries = load_boundaries();
    let image = load_image_from_file("data/hillshade.png");

    // Convert to Skia
    Ok(GeoWithPathAndCities {
        cities,
        ways,
        dem: image,
        boundaries,
    })
}
