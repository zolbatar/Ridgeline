use crate::game::city::City;
use crate::geo::data::GeoWithPathAndCities;
use std::rc::Rc;

pub enum PlayerType {
    Player,
    NotAssigned,
}

pub struct Player {
    pub player_type: PlayerType,
    pub cities: Vec<Rc<City>>,
}

impl Player {
    pub fn new(player_type: PlayerType) -> Player {
        Player {
            player_type,
            cities: Vec::new(),
        }
    }

    pub fn assign_all(&mut self, geo_and_cities: &GeoWithPathAndCities) {
        for city in geo_and_cities.cities.iter() {
            self.cities.push(Rc::new(City {
                location: city.clone(),
            }));
        }
    }

    pub fn change_ownership(&mut self, city: Rc<City>) {
        self.cities.push(city);
    }
}
