use crate::geo::data::{Geo, GeoRegion, GeoWithPathAndCities, Location};
use crate::geo::paths::convert_paths;
use geo::{Area, Geometry};
use geojson::GeoJson;
use serde_cbor::from_reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub fn create_geo() {
    let geo = load_geojson();
    serialize(geo).expect("Unable to serialize GEO");
}

fn load_geojson() -> HashMap<u16, Geo> {
    // Open and read the .geojson file
    let file = File::open("merged_by_region.geojson").expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Parse the files as GeoJSON
    let geojson: GeoJson = serde_json::from_reader(reader).expect("Unable to read geojson");

    // Extract features (country boundaries)
    let mut m = HashMap::new();
    let mut count = 0usize;
    if let GeoJson::FeatureCollection(fc) = geojson {
        for feature in fc.features {
            let fp = feature.properties.unwrap();
            let rc = fp.get("region_id").unwrap().as_number().unwrap();
            let region_code: u16 = rc.as_f64().unwrap() as u16;
            if let Some(geometry) = feature.geometry {
                let geo_geometry: geo::Geometry<f64> = geometry.try_into().unwrap();

                // We don't want the islands
                let region = GeoRegion::from_id(region_code).unwrap();
                let extracted = match geo_geometry {
                    Geometry::Polygon(polygon) => vec![polygon],              // Single polygon
                    Geometry::MultiPolygon(multi_polygon) => multi_polygon.0, // Multiple polygons
                    _ => panic!("Unsupported geo type"),
                };

                // Go through each polygon and decide if we want it
                let mut v = Vec::new();
                for poly in extracted.into_iter() {
                    let area = -poly.signed_area() / 10000000.0;
                    if area > 10.0 {
                        v.push(poly);
                        count += 1;
                    }
                }
                m.insert(
                    region_code,
                    Geo {
                        geo: v,
                        region,
                    },
                );
            }
        }
    }
    println!("Polygon count: {}", count);

    m
}

fn serialize(m: HashMap<u16, Geo>) -> Result<(), Box<dyn Error>> {
    let file = File::create("Geo.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

pub fn load() -> Result<GeoWithPathAndCities, Box<dyn Error>> {
    let file = File::open("Geo.cbor")?;
    let reader = BufReader::new(file);
    let data: HashMap<u16, Geo> = serde_cbor::from_reader(reader)?;
    let cities = load_cbor_file("Cities.cbor");

    // Convert to Skia
    Ok(GeoWithPathAndCities {
        geo_with_path: convert_paths(data),
        cities,
    })
}

fn load_cbor_file(file_path: &str) -> Vec<Location> {
    // Open the CBOR file
    let file = File::open(file_path).expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let locations: Vec<Location> = from_reader(reader).expect("Unable to read GEO file");

    locations
}
