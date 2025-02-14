use crate::game::city::City;
use crate::geo::data::GeoWithPathAndCities;

pub enum PlayerType {
    Player,
    NotAssigned,
}

pub struct Player {
    pub player_type: PlayerType,
    pub cities: Vec<City>,
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
            self.cities.push(City {
                location: city.clone(),
            });
        }
    }

    pub fn change_ownership(&mut self, city: City) {
        self.cities.push(city);
    }
}
