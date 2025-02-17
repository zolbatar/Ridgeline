use crate::geo::load::RATIO_ADJUST;
use crate::gfx::skia::Skia;
use proj::{Coord, Proj};
use skia_safe::paint::Style;
use skia_safe::{BlendMode, Color, Image, Paint, Rect};

pub fn draw_dem(skia: &mut Skia, image: &Image) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Fill);
    paint.set_color(Color::WHITE);
    paint.set_alpha(64);

    // Rescale
    let north = 60.655938;
    let south = 49.908067;
    let west = -9.222697;
    let east = 1.559322; // 2.693491

    // Define the transformation from OSGB 1936 (EPSG:27700) to Web Mercator (EPSG:3857)
    let from_proj = "EPSG:4326";
    let to_proj = "EPSG:27700";
    let proj = Proj::new_known_crs(from_proj, to_proj, None).expect("Failed to create projection");
    let c1 = proj.convert((west, north)).unwrap();
    let c2 = proj.convert((east, south)).unwrap();
    let dst = Rect::from_xywh(
        0.0 + c1.x() / RATIO_ADJUST,
        -c1.y() / RATIO_ADJUST,
        (c2.x() - c1.x()) / RATIO_ADJUST,
        -(c2.y() - c1.y()) / RATIO_ADJUST,
    );
    paint.set_blend_mode(BlendMode::Multiply);
    skia.get_canvas().draw_image_rect(image, None, dst, &paint);
//    paint.set_blend_mode(BlendMode::default());
//    skia.get_canvas().draw_image_rect(image, None, dst, &paint);
}
