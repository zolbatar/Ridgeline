use crate::geo::data::{Way, WayClass, WayForm, WayPoint, WaySkia};
use crate::geo::load::RATIO_ADJUST;
use crate::gfx::skia::Skia;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geos::Geometry;
use serde_cbor::from_reader;
use skia_safe::paint::Style;
use skia_safe::{scalar, Color, Paint, Path, Point};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

pub fn load_ways() -> HashMap<WayClass, Vec<WaySkia>> {
    let file = File::open("data/Ways.cbor").expect("Unable to open ways file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let locations: HashMap<WayClass, Vec<Way>> = from_reader(reader).expect("Unable to read ways file");

    let mut ways = HashMap::new();
    let mut count = 0;
    for (class, locations) in locations.into_iter() {
        ways.insert(class.clone(), Vec::<WaySkia>::new());
        count += locations.len();
        let mv = ways.get_mut(&class).unwrap();
        for location in locations.iter() {
            let p = path_from_ways(&location.way_points);
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

pub fn path_from_ways(points: &Vec<WayPoint>) -> Path {
    let mut p = Path::new();
    points.iter().for_each(|wp| {
        let cpp = Point::new(wp.x as scalar, -wp.y as scalar);
        if wp.is_start {
            p.move_to(cpp);
        } else {
            p.line_to(cpp);
        }
    });
    p
}

pub fn serialize_ways(m: HashMap<WayClass, Vec<Way>>) -> Result<(), Box<dyn Error>> {
    let file = File::create("data/Ways.cbor")?;
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &m)?;
    Ok(())
}

pub fn create_ways() {
    let dataset = Dataset::open("/Users/daryl/OSM/oproad_gpkg_gb/Data/oproad_gb.gpkg").unwrap();
    let mut road_link = dataset.layer_by_name("road_link").unwrap();

    // Each feature is a separate road link
    let mut ways = Vec::new();
    for feature in road_link.features() {
        // Helper function to extract values
        let extract_string = |field: &str| {
            let uw = feature.field(field).unwrap_or_default();
            if let Some(value) = uw {
                value.into_string().unwrap()
            } else {
                String::new()
            }
        };

        /*        for (field_name, field_value) in feature.fields() {
            println!("{} {:?}", field_name, field_value);
        }*/

        // Get feature values
        let road_classification = extract_string("road_classification");
        let _road_function = extract_string("road_function");
        let _road_number = extract_string("road_number");
        let form_of_way = extract_string("form_of_way");
        let name = extract_string("name_1");

        let clazz = match road_classification.as_str() {
            "A Road" => WayClass::ARoad,
            "B Road" => WayClass::BRoad,
            "Motorway" => WayClass::Motorway,
            "Unclassified" => WayClass::Unclassified,
            "Not Classified" => WayClass::Unclassified,
            "Classified Unnumbered" => WayClass::Unclassified,
            "Unknown" => WayClass::Unknown,
            _ => todo!("Unknown class: {}", road_classification),
        };

        let form = match form_of_way.as_str() {
            "Single Carriageway" => WayForm::SingleCarriageway,
            "Shared Use Carriageway" => WayForm::SingleCarriageway,
            "Dual Carriageway" => WayForm::DualCarriageway,
            "Collapsed Dual Carriageway" => WayForm::CollapsedDualCarriageway,
            "Slip Road" => WayForm::SlipRoad,
            "Roundabout" => WayForm::Roundabout,
            "Guided Busway" => WayForm::PublicTransportWay,
            _ => todo!("Unknown form: {}", form_of_way),
        };

        // Geometry
        let geometry = feature.geometry().unwrap();
        let my = get_geometry(geometry, false);

        // And save
        ways.push(Way {
            name,
            class: clazz,
            form,
            way_points: my,
        });
    }

    println!("There are {} raw ways", ways.len());

    // Concatenate road
    let mut waypoints_concat: HashMap<String, Vec<Way>> = HashMap::new();
    for way in ways.into_iter() {
        if !waypoints_concat.contains_key(&way.name) {
            waypoints_concat.insert(way.name.clone(), vec![way]);
        } else {
            waypoints_concat.get_mut(&way.name).unwrap().push(way);
        }
    }

    // Serialise
    let file = File::create("data/WaysRaw.cbor").unwrap();
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &waypoints_concat).unwrap();
}

pub fn get_geometry(geometry: &gdal::vector::Geometry, simplify: bool) -> Vec<WayPoint> {
    let mut my = Vec::new();
    if simplify {
        geometry.simplify(0.1).unwrap();
    }
    for i in 0..geometry.point_count() {
        let (x, y, _) = geometry.get_point(i as i32);
        my.push(WayPoint {
            is_start: i == 0,
            x: x / RATIO_ADJUST as f64,
            y: y / RATIO_ADJUST as f64,
        });
    }
    my
}

pub fn categorise_ways() -> HashMap<WayClass, Vec<Way>> {
    let file = File::open("data/WaysRaw.cbor").expect("Unable to open WaysRaw file");
    let reader = BufReader::new(file);
    let waypoints_concat: HashMap<String, Vec<Way>> = from_reader(reader).expect("Unable to read WaysRaw file");

    // Now categorise
    let mut whm = HashMap::new();
    whm.insert(WayClass::BRoad, Vec::new());
    whm.insert(WayClass::ARoad, Vec::new());
    whm.insert(WayClass::Motorway, Vec::new());
    whm.insert(WayClass::Unclassified, Vec::new());
    whm.insert(WayClass::Unknown, Vec::new());
    let mut count = 0;
    for (_name, way) in waypoints_concat.into_iter() {
        way.into_iter().for_each(|way| {
            count += way.way_points.len();
            whm.get_mut(&way.class).unwrap().push(way);
        });
    }
    println!("There are {} raw points", count);

    whm
}

pub fn draw_ways(skia: &mut Skia, ways: &HashMap<WayClass, Vec<WaySkia>>) {
    draw_ways_type(skia, ways.get(&WayClass::Unknown).unwrap());
    draw_ways_type(skia, ways.get(&WayClass::Unclassified).unwrap());
    draw_ways_type(skia, ways.get(&WayClass::BRoad).unwrap());
    draw_ways_type(skia, ways.get(&WayClass::ARoad).unwrap());
    draw_ways_type(skia, ways.get(&WayClass::Motorway).unwrap());
}

fn draw_ways_type(skia: &mut Skia, ways: &[WaySkia]) {
    let anti_alias = true;
    
    let mut paint_motorway = Paint::default();
    paint_motorway.set_anti_alias(true);
    paint_motorway.set_style(Style::Stroke);
    paint_motorway.set_color(Color::from_rgb(123, 104, 238));
    paint_motorway.set_stroke_width(0.25);

    let mut paint_a_road = Paint::default();
    paint_a_road.set_anti_alias(anti_alias);
    paint_a_road.set_style(Style::Stroke);
    paint_a_road.set_color(Color::GREEN);
    paint_a_road.set_stroke_width(0.1);

    let mut paint_b_road = Paint::default();
    paint_b_road.set_anti_alias(anti_alias);
    paint_b_road.set_style(Style::Stroke);
    paint_b_road.set_color(Color::from_rgb(232, 144, 30));
    paint_b_road.set_stroke_width(0.025);

    let mut paint_unclassified_road = Paint::default();
    paint_unclassified_road.set_anti_alias(anti_alias);
    paint_unclassified_road.set_style(Style::Stroke);
    paint_unclassified_road.set_color(Color::BLACK);
    paint_unclassified_road.set_stroke_width(0.025);

    let mut paint_unknown_road = Paint::default();
    paint_unknown_road.set_anti_alias(anti_alias);
    paint_unknown_road.set_style(Style::Stroke);
    paint_unknown_road.set_color(Color::GRAY);
    paint_unknown_road.set_stroke_width(0.025);

    ways.iter().for_each(|w| {
        match w.class {
            WayClass::Unknown => {
//                skia.get_canvas().draw_path(&w.path, &paint_unknown_road);
            }
            WayClass::Unclassified => {
                skia.get_canvas().draw_path(&w.path, &paint_unclassified_road);
            }
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
