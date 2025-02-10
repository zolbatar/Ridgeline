use crate::geo::data::{Geo, GeoWithPath};
use crate::gfx::skia::Skia;
use geo::{Geometry, LineString};
use skia_safe::paint::Style;
use skia_safe::{op, scalar, Color, Paint, Path, PathOp, Point};
use std::collections::HashMap;

const COLOR_PALETTE: [Color; 14] = [
    Color::from_rgb(0x4E, 0x2A, 0x2A), // Dark Maroon
    Color::from_rgb(0x3A, 0x2F, 0x2F), // Deep Brown
    Color::from_rgb(0x5A, 0x43, 0x3B), // Muted Chestnut
    Color::from_rgb(0x2E, 0x3B, 0x2F), // Dark Moss Green
    Color::from_rgb(0x22, 0x22, 0x1F), // Almost Black (Charcoal)
    Color::from_rgb(0x3F, 0x2E, 0x56), // Deep Purple
    Color::from_rgb(0x2A, 0x1B, 0x3D), // Muted Eggplant
    Color::from_rgb(0x38, 0x28, 0x4C), // Dark Indigo
    Color::from_rgb(0x3C, 0x2E, 0x3F), // Dusty Plum
    Color::from_rgb(0x2D, 0x26, 0x2C), // Smoky Brown
    Color::from_rgb(0x24, 0x1B, 0x1E), // Deep Wine Red
    Color::from_rgb(0x1F, 0x30, 0x24), // Dark Forest Green
    Color::from_rgb(0x26, 0x1A, 0x26), // Faded Aubergine
    Color::from_rgb(0x29, 0x21, 0x3D), // Deep Grape
];

pub fn convert_paths(geo: HashMap<String, Geo>) -> HashMap<String, GeoWithPath> {
    let mut paths = HashMap::new();
    for (admin, y) in geo.into_iter() {
        let extracted = match &y.geo {
            Geometry::Polygon(polygon) => vec![polygon.clone()], // Single polygon
            Geometry::MultiPolygon(multi_polygon) => multi_polygon.0.clone(), // Multiple polygons
            _ => vec![],
        };

        // Create skia pth
        let mut polys = Vec::new();
        extracted.iter().for_each(|v| {
            let mut path = build_path(v.exterior());

            // Interiors
            v.interiors().iter().for_each(|v| {
                let path_interior = build_path(v);
                path = op(&path, &path_interior, PathOp::Difference).unwrap();
            });

            polys.push(path);
        });
        paths.insert(
            admin,
            GeoWithPath {
                name: y.name,
                population: y.population,
                map_colour: y.map_colour,
                polys,
            },
        );
    }
    paths
}

fn build_path(poly: &LineString) -> Path {
    let mut path = Path::new();
    poly.points().for_each(|point| {
        if path.is_empty() {
            path.move_to(Point::new(point.x() as scalar, -point.y() as scalar));
        } else {
            path.line_to(Point::new(point.x() as scalar, -point.y() as scalar));
        }
    });
    path.close();
    path
}

pub fn draw_all_paths(skia: &mut Skia, polys: &HashMap<String, GeoWithPath>) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Stroke);
    paint.set_color(Color::BLACK);
    paint.set_stroke_width(0.1);

    let mut paint_fill = Paint::default();
    paint_fill.set_anti_alias(true);
    paint_fill.set_style(Style::Fill);

    for (_admin, geo) in polys {
        let colour = COLOR_PALETTE[((geo.map_colour - 1) as usize) % COLOR_PALETTE.len()];
        paint_fill.set_color(colour);
        for path in geo.polys.iter() {
            skia.get_canvas().draw_path(path, &paint_fill);
            skia.get_canvas().draw_path(path, &paint);
        }
    }
}
