use crate::game::player::{Player, PlayerType};
use crate::gfx::skia::{clip_circle, Skia};
use skia_safe::paint::Style;
use skia_safe::utils::text_utils::Align;
use skia_safe::{scalar, Color, Paint, Point, Rect};

pub fn draw_all_cities(skia: &mut Skia, players: &Vec<Player>) {
    let font = &skia.font_label.clone();
    let radius = 40.0;
    let border_width = 3.0;
    let spacing = 7.0;
    let corner_radius = radius + spacing;
    let metrics = font.metrics();
    let descent = metrics.1.descent;

    let mut paint = Paint::default();
    paint.set_anti_alias(true);
    paint.set_style(Style::Fill);

    let mut paint_bg = Paint::default();
    paint_bg.set_anti_alias(true);
    paint_bg.set_style(Style::Fill);

    let mut paint_bg_alpha = Paint::default();
    paint_bg_alpha.set_anti_alias(true);
    paint_bg_alpha.set_style(Style::Fill);
    paint_bg_alpha.set_color(Color::DARK_GRAY);
    paint_bg_alpha.set_alpha(128);

    let mut paint_border = Paint::default();
    paint_border.set_anti_alias(true);
    paint_border.set_style(Style::Stroke);
    paint_border.set_color(Color::BLACK);
    paint_border.set_stroke_width(border_width);

    let mut paint_shadow = Paint::default();
    paint_shadow.set_anti_alias(true);
    paint_shadow.set_style(Style::Fill);
    paint_shadow.set_image_filter(skia.drop_shadow.clone());

    let canvas = skia.get_canvas();
    players.iter().for_each(|player| {
        // Colour according to ownership
        match player.player_type {
            PlayerType::Player => {
                paint_bg.set_color(Color::YELLOW);
                paint.set_color(Color::BLACK);
            }
            PlayerType::NotAssigned => {
                paint_bg.set_color(Color::GRAY);
                paint.set_color(Color::WHITE);
            }
        };

        player.cities.iter().for_each(|city| {
            let l = &city.location;

            // Width of text
            let (w, _bounds) = font.measure_text(&l.name, Some(&paint));

            let r1 = Rect::from_xywh(
                l.x as scalar - radius - spacing,
                -l.y as scalar - radius - spacing,
                w + radius * 2.0 + spacing * 4.0 + 16.0,
                radius * 2.0 + spacing * 2.0,
            );
            let p1 = Point::new(l.x as scalar, -l.y as scalar);
            let p2 = Point::new(l.x as scalar + radius + 8.0, -l.y as scalar + radius - descent /* - h / 2.0*/);

            // Apply the clip circle
            canvas.save();
            clip_circle(canvas, p1, radius);
            canvas.draw_round_rect(r1, corner_radius, corner_radius, &paint_shadow);
            canvas.draw_round_rect(r1, corner_radius, corner_radius, &paint_bg);
            canvas.draw_round_rect(r1, corner_radius, corner_radius, &paint_border);
            canvas.restore();
            canvas.draw_circle(p1, radius, &paint_bg_alpha);
            canvas.draw_circle(p1, radius, &paint_border);
            canvas.draw_text_align(&l.name, p2, font, &paint, Align::Left);
        })
    })
}
