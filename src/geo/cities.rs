use crate::app_state::AppState;
use crate::game::player::PlayerType;
use crate::geo::data::Location;
use crate::geo::load::RATIO_ADJUST;
use crate::gfx::skia::{Skia, LABEL_SIZE};
use serde_cbor::from_reader;
use skia_safe::paint::Style;
use skia_safe::utils::text_utils::Align;
use skia_safe::{scalar, Color, Paint, Point, Rect};
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

pub fn draw_all_cities(skia: &mut Skia, app_state: &AppState) {
    let font = &skia.font_label.clone();
    let font_bold = &skia.font_label_bold.clone();
    let radius = LABEL_SIZE * 0.65;
    let dot_radius = radius / 5.0;
    let spacing = LABEL_SIZE * 0.1;
    let border_width = spacing / 1.0;
    let corner_radius = radius + spacing;
    let bar_width = 128.0;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Fill);

    let mut paint_bg = Paint::default();
    paint_bg.set_anti_alias(true);
    paint_bg.set_style(Style::Fill);

    let mut paint_bg_alpha = Paint::default();
    paint_bg_alpha.set_anti_alias(true);
    paint_bg_alpha.set_style(Style::Fill);
    paint_bg_alpha.set_color(Color::LIGHT_GRAY);
    paint_bg_alpha.set_alpha(128);

    let mut paint_border = Paint::default();
    paint_border.set_anti_alias(true);
    paint_border.set_style(Style::Stroke);
    paint_border.set_color(Color::WHITE);
    paint_border.set_stroke_width(border_width);

    let mut paint_shadow = Paint::default();
    paint_shadow.set_anti_alias(true);
    paint_shadow.set_style(Style::Fill);
    paint_shadow.set_image_filter(skia.drop_shadow.clone());

    let mut paint_dot = Paint::default();
    paint_dot.set_anti_alias(true);
    paint_dot.set_style(Style::Fill);
    paint_dot.set_color(Color::BLACK);
    paint_dot.set_alpha(255);

    let canvas = skia.get_canvas();
    app_state.players.iter().for_each(|player| {
        // Colour according to ownership
        let (font, descent) = match player.player_type {
            PlayerType::Player => {
                paint_bg.set_color(Color::YELLOW);
                paint.set_color(Color::BLACK);
                (font_bold, font_bold.metrics().1.descent)
            }
            PlayerType::NotAssigned => {
                paint_bg.set_color(Color::DARK_GRAY);
                paint.set_color(Color::WHITE);
                (font, font.metrics().1.descent)
            }
        };

        player.cities.iter().for_each(|city| {
            let l = &city.location;

            // Width of text
            let (w, _bounds) = font.measure_text(&l.name, Some(&paint));

            let r1 = if app_state.selected_city.is_some() && Rc::eq(city, app_state.selected_city.as_ref().unwrap()) {
                Rect::from_xywh(
                    l.x as scalar - radius - spacing - bar_width,
                    -l.y as scalar - radius - spacing,
                    w + spacing * 2.0,
                    radius * 2.0 + spacing * 2.0,
                )
            } else {
                Rect::from_xywh(
                    l.x as scalar - radius - spacing,
                    -l.y as scalar - radius - spacing,
                    w + spacing * 2.0 + radius * 2.0,
                    radius * 2.0 + spacing * 2.0,
                )
            };
            let p1 = Point::new(l.x as scalar, -l.y as scalar);
            let p2 = Point::new(l.x as scalar, -l.y as scalar + radius - descent);

            // Apply the clip circle
            /*            canvas.save();
            clip_circle(canvas, p1, radius);*/
            canvas.draw_round_rect(r1, corner_radius, corner_radius, &paint_shadow);
            canvas.draw_round_rect(r1, corner_radius, corner_radius, &paint_bg);
            canvas.draw_round_rect(r1, corner_radius, corner_radius, &paint_border);
            /*            canvas.restore();
            canvas.draw_circle(p1, radius, &paint_bg_alpha);
            canvas.draw_circle(p1, dot_radius, &paint_dot);
            canvas.draw_circle(p1, radius, &paint_border);*/
            //            canvas.draw_circle(p1, dot_radius, &paint_dot);
            //println!("{}, {}", p2.x, p2.y);
            canvas.draw_text_align(&l.name, p2, font, &paint, Align::Left);
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
