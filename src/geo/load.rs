use crate::geo::data::{Geo, GeoWithPathAndCities, Location, Way, WayClass, WayForm, WayPoint, WaySkia};
use crate::geo::paths::convert_paths;
use geo::Geometry;
use geojson::GeoJson;
use proj::{Coord, Proj};
use serde_cbor::from_reader;
use shapefile::dbase::{FieldValue, Record};
use shapefile::{PointZ, PolylineZ, ShapeType};
use skia_safe::wrapper::NativeTransmutableWrapper;
use skia_safe::{scalar, Point};
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;

pub fn create_geo() {
    let geo = load_geojson();
    serialize_geo(geo).expect("Unable to serialize GEO");
    let ways = create_ways();
    serialize_ways(ways).expect("Unable to serialize Ways");
}

pub fn create_ways() -> Vec<Way> {
    let dir = "/Users/daryl/OSM/oproad_essh_gb/data/".to_string();
    let mut all_files = Vec::new();
    let entries = fs::read_dir(&dir).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.contains(".shp") {
                    all_files.push(dir.clone() + file_name);
                }
            }
        }
    }

    // Define the transformation from OSGB 1936 (EPSG:27700) to Web Mercator (EPSG:3857)
    let from_proj = "EPSG:27700"; // OSGB 1936
    let to_proj = "EPSG:3857"; // Web Mercator
    let proj = Proj::new_known_crs(from_proj, to_proj, None).expect("Failed to create projection");

    fn extract_character(record: &Record, field_name: &str) -> String {
        if let Some(FieldValue::Character(Some(x))) = record.get(field_name) {
            x.clone()
        } else {
            "".to_string()
        }
    }

    // Load OS data
    let mut ways = Vec::new();
    for filename in all_files.iter() {
        let filename = Path::new(&filename);
        let mut reader = shapefile::Reader::from_path(filename).unwrap();

        for result in reader.iter_shapes_and_records() {
            let (shape, record) = result.unwrap();
            if let Some(FieldValue::Character(Some(x))) = record.get("roadNumber") {
                //let function = extract_character(&record, "function");
                let form = extract_character(&record, "formOfWay");
                let class = extract_character(&record, "class");
                //println!("Road: '{}'/'{}'/'{}'/'{}'", x, class, function, form);

                let clazz = match class.as_str() {
                    "A Road" => WayClass::ARoad,
                    "B Road" => WayClass::BRoad,
                    "Motorway" => WayClass::Motorway,
                    _ => todo!("Unknown class: {}", class),
                };

                let forme = match form.as_str() {
                    "Single Carriageway" => WayForm::SingleCarriageway,
                    "Dual Carriageway" => WayForm::DualCarriageway,
                    "Collapsed Dual Carriageway" => WayForm::CollapsedDualCarriageway,
                    "Slip Road" => WayForm::SlipRoad,
                    "Roundabout" => WayForm::Roundabout,
                    _ => todo!("Unknown form: {}", form),
                };

                /*                for (name, value) in record {
                    println!("\t{}: {:?}, ", name, value);
                }
                println!();*/

                match shape.shapetype() {
                    ShapeType::PolylineZ => {
                        let polyline: PolylineZ = shape.try_into().unwrap();

                        // Iterate over parts (a polyline can have multiple parts)
                        for part in polyline.parts().iter() {
                            let mut my = Vec::new();
                            for point in part.iter() {
                                let cp = proj.convert((point.x, point.y)).unwrap();
                                my.push(WayPoint {
                                    x: cp.x() / 1000.0,
                                    y: cp.y() / 1000.0,
                                });
                            }
                            ways.push(Way {
                                class: clazz.clone(),
                                form: forme.clone(),
                                way_points: my,
                            });
                        }
                    }

                    ShapeType::PointZ => {
                        let pointz: PointZ = shape.try_into().unwrap();
                        //                    let a = 1;
                    }

                    _ => println!("Skipping unknown shape type {:?}", shape.shapetype()),
                }
            };
        }
    }
    println!("There are {} ways", ways.len());
    ways
}

pub fn load_ways() -> Vec<WaySkia> {
    let file = File::open("Ways.cbor").expect("Unable to open Ways file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let locations: Vec<Way> = from_reader(reader).expect("Unable to read Ways file");

    let mut ways = Vec::new();
    for location in locations.iter() {
        let mut p = skia_safe::Path::new();
        location.way_points.iter().enumerate().for_each(|(i, wp)| {
            let cpp = Point::new(wp.x as scalar, -wp.y as scalar);
            if i == 0 {
                p.move_to(cpp);
            } else {
                p.line_to(cpp);
            }
        });
        ways.push(WaySkia {
            class: location.class.clone(),
            form: location.form.clone(),
            path: p,
        });
    }

    println!("There are {} ways", ways.len());
    ways
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

    m
}

fn serialize_geo(m: Vec<Geo>) -> Result<(), Box<dyn Error>> {
    let file = File::create("Geo.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

fn serialize_ways(m: Vec<Way>) -> Result<(), Box<dyn Error>> {
    let file = File::create("Ways.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

pub fn load(radius: f64) -> Result<GeoWithPathAndCities, Box<dyn Error>> {
    let file = File::open("Geo.cbor")?;
    let reader = BufReader::new(file);
    let data: Vec<Geo> = from_reader(reader)?;
    let cities = load_cbor_file("Cities.cbor", radius);
    let ways = load_ways();

    // Convert to Skia
    Ok(GeoWithPathAndCities {
        geo_with_path: convert_paths(data),
        cities,
        ways,
    })
}

fn load_cbor_file(file_path: &str, radius: f64) -> Vec<Rc<Location>> {
    // Open the CBOR file
    let file = File::open(file_path).expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let locations: Vec<Location> = from_reader(reader).expect("Unable to read GEO file");

    // Now only select those that aren't too close to a neighbour, starting at largest down
    let mut locations_out: Vec<Rc<Location>> = Vec::new();
    for location in locations.into_iter() {
        if location.population >= 10000 {
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
    }

    locations_out
}

fn calculate_distance(city1: &Location, city2: &Location) -> f64 {
    let dx = city1.x - city2.x;
    let dy = city1.y - city2.y;
    (dx * dx + dy * dy).sqrt()
}
