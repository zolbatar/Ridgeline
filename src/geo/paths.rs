use crate::geo::data::{Geo, GeoWithPath};
use crate::gfx::skia::Skia;
use geo::LineString;
use skia_safe::paint::Style;
use skia_safe::{scalar, Color, Paint, Path, Point, Vector};
use std::collections::{HashMap, HashSet};

pub fn convert_paths(geo: HashMap<u16, Geo>, regions: &HashSet<u16>) -> HashMap<u16, GeoWithPath> {
    let mut paths = HashMap::new();
    for (admin, y) in geo.into_iter() {
        // Create skia path
        let mut polys = Vec::new();
        y.geo.iter().for_each(|v| {
            let path = build_path(v.exterior());
            polys.push(path);
        });

        paths.insert(
            admin,
            GeoWithPath {
                enabled: regions.contains(&admin),
                polys,
                region: y.region,
            },
        );
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

pub fn draw_all_paths(skia: &mut Skia, polys: &HashMap<u16, GeoWithPath>) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Stroke);
    let bg = Color::BLACK;
    paint.set_color(bg);
    paint.set_stroke_width(3.0);

    let mut paint_shadow = Paint::default();
    paint_shadow.set_anti_alias(true);
    paint_shadow.set_style(Style::Fill);
    paint_shadow.set_color(Color::BLACK);
    paint_shadow.set_alpha(255);

    let mut paint_fill = Paint::default();
    paint_fill.set_anti_alias(true);
    paint_fill.set_style(Style::Fill);

    // Draw "shadow"
    skia.get_canvas().save();
    let zz = 5.0;
    skia.get_canvas().translate(Vector::new(zz, zz));
    for geo in polys.values() {
        if geo.enabled {
            for path in geo.polys.iter() {
                skia.get_canvas().draw_path(path, &paint_shadow);
            }
        }
    }
    skia.get_canvas().restore();

    // And actual polys
    for geo in polys.values() {
        let colour = geo.region.colour();
        if geo.enabled {
            paint_fill.set_color(colour);
        } else {
            paint_fill.set_color(Color::from_argb(96, 80, 80, 80));
        }
        for path in geo.polys.iter() {
            skia.get_canvas().draw_path(path, &paint_fill);
            skia.get_canvas().draw_path(path, &paint);
        }
    }
}
