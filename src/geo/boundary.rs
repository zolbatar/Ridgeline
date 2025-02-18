use crate::geo::data::{Geo, GeoWithPath, Way, WayClass, WayPoint, WaySkia};
use crate::geo::load::RATIO_ADJUST;
use crate::geo::ways::{get_geometry, path_from_ways};
use crate::gfx::skia::Skia;
use gdal::vector::LayerAccess;
use gdal::Dataset;
use geo::LineString;
use serde_cbor::from_reader;
use skia_safe::paint::Style;
use skia_safe::{scalar, Color, Paint, Path, Point, Vector};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::process::exit;

pub fn create_boundaries() {
    let dataset = Dataset::open("/Users/daryl/OSM/terr50_gpkg_gb/Data/terr50_gb.gpkg").unwrap();
    // contour_line
    let mut land_water_boundary = dataset.layer_by_name("land_water_boundary").unwrap();
    let mut vec = Vec::new();
    for feature in land_water_boundary.features() {
        let geometry = feature.geometry().unwrap();
        //geometry.simplify_preserve_topology(0.1).unwrap();

        // Get all points
        let my = get_geometry(geometry, false);
        vec.push(my);
    }
    println!("There are {} boundary lines", vec.len());

    // Serialise
    let file = File::create("data/Boundaries.cbor").unwrap();
    let writer = std::io::BufWriter::new(file);
    serde_cbor::to_writer(writer, &vec).unwrap();
}

pub fn load_boundaries() -> Vec<Path> {
    let file = File::open("data/Boundaries.cbor").expect("Unable to open boundaries file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let boundaries: Vec<Vec<WayPoint>> = from_reader(reader).expect("Unable to read boundaries file");

    let mut vec = Vec::new();
    boundaries.iter().for_each(|line| {
        let p = path_from_ways(line);
        vec.push(p);
    });
    
    println!("There are {} boundary lines", vec.len());
    vec
}

pub fn draw_boundaries(skia: &mut Skia, boundaries: &[Path]) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Stroke);
    paint.set_color(Color::BLACK);
    paint.set_stroke_width(0.1);

    boundaries.iter().for_each(|line| {
        skia.get_canvas().draw_path(line, &paint);
    });
}
