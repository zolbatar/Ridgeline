use crate::geo::data::{Geo, GeoWithPath};
use crate::geo::paths::convert_paths;
use geojson::GeoJson;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub fn create_geo() {
    let geo = load_geojson().expect("Unable to load GEO");
    serialize(geo).expect("Unable to serialize GEO");
}

fn load_geojson() -> Result<HashMap<String, Geo>, Box<dyn Error>> {
    // Open and read the .geojson file
    //    let file = File::open("/Users/daryl/OSM/AllCountries.geojson")?;
    let file = File::open("cropped_file.geojson")?;
    let reader = BufReader::new(file);

    // Parse the file as GeoJSON
    let geojson: GeoJson = serde_json::from_reader(reader)?;

    // Extract features (country boundaries)
    let mut m = HashMap::new();
    if let GeoJson::FeatureCollection(fc) = geojson {
        for feature in fc.features {
            let fp = feature.properties.unwrap();
            let admin = fp.get("ADM0_A3").unwrap().as_str().unwrap().to_string();
            let name = fp.get("NAME").unwrap().as_str().unwrap().to_string();
            let pop = fp.get("POP_EST").unwrap().as_f64().unwrap();
            let mut map_colour = fp.get("MAPCOLOR13").unwrap().as_i64().unwrap();
            if map_colour == -99 {
                map_colour = 14;
            }

            if let Some(geometry) = feature.geometry {
                let geo_geometry: geo::Geometry<f64> = geometry.try_into()?;
                m.insert(
                    admin,
                    Geo {
                        name,
                        population: pop as usize,
                        map_colour: map_colour as u8,
                        geo: geo_geometry,
                    },
                );
            }
        }
    }

    Ok(m)
}

fn serialize(m: HashMap<String, Geo>) -> Result<(), Box<dyn Error>> {
    let file = File::create("Geo.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

pub fn load() -> Result<HashMap<String, GeoWithPath>, Box<dyn Error>> {
    let file = File::open("Geo.cbor")?;
    let reader = BufReader::new(file);
    let data: HashMap<String, Geo> = serde_cbor::from_reader(reader)?;
    
    // Convert to Skia
    let paths = convert_paths(data);
    Ok(paths)
}

