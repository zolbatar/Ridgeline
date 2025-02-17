use crate::geo::data::{Way, WayClass, WayForm, WayPoint, WaySkia};
use crate::geo::load::RATIO_ADJUST;
use crate::geo::optimise::{multilinestring_to_waypoints, simplify_multilinestring, waypoints_to_multilinestring};
use crate::gfx::skia::Skia;
use serde_cbor::from_reader;
use shapefile::dbase::{FieldValue, Record};
use shapefile::{PolylineZ, ShapeType};
use skia_safe::paint::Style;
use skia_safe::{scalar, Color, Paint, Point};
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub fn load_ways() -> HashMap<WayClass, Vec<WaySkia>> {
    let file = File::open("data/Ways.cbor").expect("Unable to open Ways file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let locations: HashMap<WayClass, Vec<Way>> = from_reader(reader).expect("Unable to read Ways file");

    let mut ways = HashMap::new();
    let mut count = 0;
    for (class, locations) in locations.into_iter() {
        ways.insert(class.clone(), Vec::<WaySkia>::new());
        count += locations.len();
        let mv = ways.get_mut(&class).unwrap();
        for location in locations.iter() {
            let mut p = skia_safe::Path::new();
            location.way_points.iter().enumerate().for_each(|(i, wp)| {
                let cpp = Point::new(wp.x as scalar, -wp.y as scalar);
                if wp.is_start {
                    p.move_to(cpp);
                } else {
                    p.line_to(cpp);
                }
            });
            mv.push(WaySkia {
                class: location.class.clone(),
                _form: location.form.clone(),
                path: p,
            });
        }
    }

    println!("There are {} ways in {} classes", count, ways.len());
    ways
}

pub fn serialize_ways(m: HashMap<WayClass, Vec<Way>>) -> Result<(), Box<dyn Error>> {
    let file = File::create("data/Ways.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

pub fn create_ways() -> HashMap<WayClass, Vec<Way>> {
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
            if let Some(FieldValue::Character(Some(road_name))) = record.get("roadNumber") {
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
                            for (i, point) in part.iter().enumerate() {
                                my.push(WayPoint {
                                    is_start: i == 0,
                                    x: point.x / RATIO_ADJUST as f64,
                                    y: point.y / RATIO_ADJUST as f64,
                                });
                            }
                            ways.push(Way {
                                name: road_name.to_string(),
                                class: clazz.clone(),
                                form: forme.clone(),
                                way_points: my,
                            });
                        }
                    }

                    ShapeType::PointZ => {
                        //let pointz: PointZ = shape.try_into().unwrap();
                        //                    let a = 1;
                    }

                    _ => println!("Skipping unknown shape type {:?}", shape.shapetype()),
                }
            };
        }
    }
    println!("There are {} raw ways", ways.len());

    // Concatenate lines
    let mut wayhm: HashMap<String, Way> = HashMap::new();
    for mut way in ways.into_iter() {
        if !wayhm.contains_key(&way.name) {
            wayhm.insert(way.name.clone(), way.clone());
        } else {
            wayhm.get_mut(&way.name).unwrap().way_points.append(&mut way.way_points);
        }
    }

    // Now categorise
    let mut whm = HashMap::new();
    whm.insert(WayClass::BRoad, Vec::new());
    whm.insert(WayClass::ARoad, Vec::new());
    whm.insert(WayClass::Motorway, Vec::new());
    for (_, mut way) in wayhm.into_iter() {
        let v = whm.get_mut(&way.class).unwrap();

        // Optimise way
        let road_network = waypoints_to_multilinestring(way.way_points);
        let road_network_simplified = simplify_multilinestring(&road_network, 0.001);
        let waypoints = multilinestring_to_waypoints(road_network);
        way.way_points = waypoints;

        v.push(way);
    }

    whm
}

pub fn draw_ways(skia: &mut Skia, ways: &HashMap<WayClass, Vec<WaySkia>>) {
    draw_ways_type(skia, ways.get(&WayClass::BRoad).unwrap());
    draw_ways_type(skia, ways.get(&WayClass::ARoad).unwrap());
    draw_ways_type(skia, ways.get(&WayClass::Motorway).unwrap());
}

fn draw_ways_type(skia: &mut Skia, ways: &[WaySkia]) {
    let mut paint_motorway = Paint::default();
    paint_motorway.set_anti_alias(true);
    paint_motorway.set_style(Style::Stroke);
    paint_motorway.set_color(Color::from_rgb(123, 104, 238));
    paint_motorway.set_stroke_width(0.25);

    let mut paint_a_road = Paint::default();
    paint_a_road.set_anti_alias(true);
    paint_a_road.set_style(Style::Stroke);
    paint_a_road.set_color(Color::GREEN);
    paint_a_road.set_stroke_width(0.1);

    let mut paint_b_road = Paint::default();
    paint_b_road.set_anti_alias(true);
    paint_b_road.set_style(Style::Stroke);
    paint_b_road.set_color(Color::from_rgb(232, 144, 30));
    paint_b_road.set_stroke_width(0.025);

    ways.iter().for_each(|w| {
        match w.class {
            WayClass::ARoad => {
                skia.get_canvas().draw_path(&w.path, &paint_a_road);
            }
            WayClass::BRoad => {
                skia.get_canvas().draw_path(&w.path, &paint_b_road);
            }
            WayClass::Motorway => {
                skia.get_canvas().draw_path(&w.path, &paint_motorway);
            }
        };
    });
}
