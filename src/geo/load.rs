use crate::geo::data::{Geo, GeoWithPathAndCities, Location};
use crate::geo::paths::convert_paths;
use geo::{Area, Geometry};
use geojson::GeoJson;
use serde_cbor::from_reader;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

pub fn create_geo() {
    let geo = load_geojson();
    serialize(geo).expect("Unable to serialize GEO");
}

fn load_geojson() -> Vec<Geo> {
    // Open and read the .geojson file
    let file = File::open("merged_by_region.geojson").expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Parse the files as GeoJSON
    let geojson: GeoJson = serde_json::from_reader(reader).expect("Unable to read geojson");

    // Extract features (country boundaries)
    let mut m = Vec::new();
    let mut count = 0usize;
    if let GeoJson::FeatureCollection(fc) = geojson {
        for feature in fc.features {
            let fp = feature.properties.unwrap();
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
                    let area = -poly.signed_area() / 10000000.0;
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

    m
}

fn serialize(m: Vec<Geo>) -> Result<(), Box<dyn Error>> {
    let file = File::create("Geo.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

pub fn load(radius: f64) -> Result<GeoWithPathAndCities, Box<dyn Error>> {
    let file = File::open("Geo.cbor")?;
    let reader = BufReader::new(file);
    let data: Vec<Geo> = from_reader(reader)?;
    let cities = load_cbor_file("Cities.cbor", radius);

    // Convert to Skia
    Ok(GeoWithPathAndCities {
        geo_with_path: convert_paths(data),
        cities,
    })
}

fn load_cbor_file(file_path: &str, radius: f64) -> Vec<Rc<Location>> {
    // Open the CBOR file
    let file = File::open(file_path).expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let mut locations: Vec<Location> = from_reader(reader).expect("Unable to read GEO file");

    // Now only select those that aren't too close to a neighbour, starting at largest down
    let mut locations_out: Vec<Rc<Location>> = Vec::new();
    for location in locations.into_iter() {
        let mut minimum_distance = f64::INFINITY;
        for location_out in &locations_out {
            let dist = calculate_distance(&location, location_out);
            if dist < minimum_distance {
                minimum_distance = dist;
            }
            if dist < radius {
                break;
            }
        }
        if minimum_distance >= radius {
            locations_out.push(Rc::new(location));
        }
    }

    locations_out
}

fn calculate_distance(city1: &Location, city2: &Location) -> f64 {
    let dx = city1.x - city2.x;
    let dy = city1.y - city2.y;
    (dx * dx + dy * dy).sqrt()
}
