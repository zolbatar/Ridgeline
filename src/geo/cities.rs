use crate::app_state::AppState;
use crate::game::player::PlayerType;
use crate::geo::data::Location;
use crate::geo::load::RATIO_ADJUST;
use crate::gfx::skia::{Skia, LABEL_SIZE};
use serde_cbor::from_reader;
use skia_safe::image_filters::drop_shadow_only;
use skia_safe::paint::Style;
use skia_safe::utils::text_utils::Align;
use skia_safe::{scalar, Color, Paint, Point, Rect, Vector};
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

pub fn draw_all_cities(skia: &mut Skia, app_state: &AppState) {
    let font = &skia.font_label.clone();
    let font_bold = &skia.font_label_bold.clone();
    let descent = font.metrics().1.descent;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::StrokeAndFill);
    paint.set_color(Color::BLACK);

    let drop_shadow_white = drop_shadow_only(Vector::new(0.00, 0.00), (0.15, 0.15), Color::WHITE,
                                             None, None, None);

    let mut paint_shadow = Paint::default();
    paint_shadow.set_anti_alias(true);
    paint_shadow.set_style(Style::Fill);
    paint_shadow.set_image_filter(drop_shadow_white);

    let canvas = skia.get_canvas();
    app_state.players.iter().for_each(|player| {
        player.cities.iter().for_each(|city| {
            let l = &city.location;
            let (w, bounds) = font.measure_text(&l.name, Some(&paint));
            let p2 = Point::new(l.x as scalar - w / 2.0, -l.y as scalar - bounds.y() / 2.0);
            canvas.draw_text_align(&l.name, p2, font_bold, &paint_shadow, Align::Left);
            canvas.draw_text_align(&l.name, p2, font_bold, &paint, Align::Left);
        })
    })
}

pub fn load_cities_cbor_file(file_path: &str, radius: f64) -> Vec<Rc<Location>> {
    // Open the CBOR file
    let file = File::open(file_path).expect("Unable to open GEO file");
    let reader = BufReader::new(file);

    // Deserialize the CBOR data into a Vec<Location>
    let locations: Vec<Location> = from_reader(reader).expect("Unable to read GEO file");

    // Now only select those that aren't too close to a neighbour, starting at largest down
    let mut locations_out: Vec<Rc<Location>> = Vec::new();
    for mut location in locations.into_iter() {
        if location.population >= 25000 {
            location.x /= RATIO_ADJUST as f64;
            location.y /= RATIO_ADJUST as f64;
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
