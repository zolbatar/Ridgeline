use crate::game::city::City;
use crate::game::player::Player;
use crate::gfx::skia::Skia;
use skia_safe::{scalar, Point};
use std::rc::Rc;

pub struct AppState {
    pub players: Vec<Player>,
    pub selected_city: Option<Rc<City>>,
}

impl AppState {
    pub fn zoom_to_selected(&self, skia: &mut Skia) {
        if let Some(selected_city) = self.selected_city.clone() {
            skia.target = Point::new(selected_city.location.x as scalar, -selected_city.location.y as scalar);
            skia.zoom = skia.zoom_max / 2.0;
        }
    }

    pub fn zoom_out(&self, skia: &mut Skia) {
        skia.zoom /= 2.0;
        skia.zoom = skia.zoom.clamp(skia.zoom_min, skia.zoom_max);
    }

    pub fn zoom_in(&self, skia: &mut Skia) {
        skia.zoom *= 2.0;
        skia.zoom = skia.zoom.clamp(skia.zoom_min, skia.zoom_max);
    }
}
