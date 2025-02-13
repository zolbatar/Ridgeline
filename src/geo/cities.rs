use crate::geo::data::Location;
use crate::gfx::skia::Skia;
use skia_safe::paint::Style;
use skia_safe::utils::text_utils::Align;
use skia_safe::{scalar, Color, Paint, Point};

pub fn draw_all_cities(skia: &mut Skia, cities: &[Location]) {
    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Fill);
    paint.set_color(Color::WHITE);
    let font = &skia.font_label.clone();
    let canvas = skia.get_canvas();
    cities.iter().for_each(|l| {
        let p1 = Point::new(l.x as scalar, -l.y as scalar);
        let p2 = Point::new(l.x as scalar + 10.0, -l.y as scalar);
        canvas.draw_circle(p1, 8.0, &paint);
        canvas.draw_text_align(&l.name, p2, font, &paint, Align::Left);
    })
}
