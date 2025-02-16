use crate::geo::data::{Geo, GeoWithPath, WayClass, WaySkia, COLOR_PALETTE};
use crate::gfx::skia::Skia;
use geo::LineString;
use skia_safe::paint::Style;
use skia_safe::{scalar, Color, Paint, Path, Point, Vector};

pub fn convert_paths(geo: Vec<Geo>) -> Vec<GeoWithPath> {
    let mut paths = Vec::new();
    for y in geo.into_iter() {
        // Create skia path
        let mut polys = Vec::new();
        y.geo.iter().for_each(|v| {
            let path = build_path(v.exterior());
            polys.push(path);
        });

        paths.push(GeoWithPath {
            polys,
        });
    }
    paths
}

fn build_path(poly: &LineString) -> Path {
    let mut path = Path::new();
    poly.points().for_each(|point| {
        let x = point.x() as scalar / 1000.0;
        let y = point.y() as scalar / 1000.0;
        if path.is_empty() {
            path.move_to(Point::new(x, -y));
        } else {
            path.line_to(Point::new(x, -y));
        }
    });
    path.close();
    path
}

pub fn draw_ways(skia: &mut Skia, ways: &[WaySkia]) {
    let mut paint_motorway = Paint::default();
    paint_motorway.set_anti_alias(true);
    paint_motorway.set_style(Style::Stroke);
    paint_motorway.set_color(Color::BLUE);
    paint_motorway.set_stroke_width(1.0);

    let mut paint_a_road = Paint::default();
    paint_a_road.set_anti_alias(true);
    paint_a_road.set_style(Style::Stroke);
    paint_a_road.set_color(Color::GREEN);
    paint_a_road.set_stroke_width(0.5);

    let mut paint_b_road = Paint::default();
    paint_b_road.set_anti_alias(true);
    paint_b_road.set_style(Style::Stroke);
    paint_b_road.set_color(Color::from_rgb(232, 144, 30));
    paint_b_road.set_stroke_width(0.25);

    ways.iter().for_each(|w| {
        let paint = match w.class {
            WayClass::ARoad => paint_a_road.clone(),
            WayClass::BRoad => paint_b_road.clone(),
            WayClass::Motorway => paint_motorway.clone(),
        };
        skia.get_canvas().draw_path(&w.path, &paint);
    });
}

pub fn draw_country(skia: &mut Skia, polys: &Vec<GeoWithPath>) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Stroke);
    let bg = Color::BLACK;
    paint.set_color(Color::BLACK);
    paint.set_stroke_width(0.5);

    let mut paint_shadow = Paint::default();
    paint_shadow.set_anti_alias(true);
    paint_shadow.set_style(Style::Fill);
    paint_shadow.set_color(Color::BLACK);
    paint_shadow.set_alpha(128);

    let mut paint_fill = Paint::default();
    paint_fill.set_anti_alias(true);
    paint_fill.set_color(Color::from_rgb(0x50, 0x3A, 0x3C));
    paint_fill.set_style(Style::Fill);

    // Draw "shadow"
    skia.get_canvas().save();
    let zz = 1.0;
    skia.get_canvas().translate(Vector::new(zz, zz));
    for geo in polys {
        for path in geo.polys.iter() {
            skia.get_canvas().draw_path(path, &paint_shadow);
        }
    }
    skia.get_canvas().restore();

    // And actual polys
    for geo in polys {
        for path in geo.polys.iter() {
            skia.get_canvas().draw_path(path, &paint_fill);
            skia.get_canvas().draw_path(path, &paint);
        }
    }
}
