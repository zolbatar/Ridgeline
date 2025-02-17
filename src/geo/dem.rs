use crate::geo::load::RATIO_ADJUST;
use crate::gfx::skia::Skia;
use proj::{Coord, Proj};
use skia_safe::paint::Style;
use skia_safe::{BlendMode, Color, FilterMode, FilterOptions, Image, MipmapMode, Paint, Rect, SamplingOptions, Vector};

pub fn draw_dem(skia: &mut Skia, image: &Image) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Fill);

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

    let mut paint_shadow = Paint::default();
    paint_shadow.set_anti_alias(true);
    paint_shadow.set_style(Style::Fill);
    paint_shadow.set_color(Color::BLACK);
    paint_shadow.set_image_filter(skia.drop_shadow.clone());

    // Draw "shadow"
    skia.get_canvas().save();
    let zz = 0.1;
    skia.get_canvas().translate(Vector::new(zz, zz));
    skia.get_canvas().draw_image_rect(image, None, dst, &paint_shadow);
    skia.get_canvas().restore();

    // Draw DEM
    let sampling = SamplingOptions::new(FilterMode::Linear, MipmapMode::Linear);
    skia.get_canvas().draw_image_rect_with_sampling_options(image, None, dst, sampling, &paint);
}
