use geo::Polygon;
use serde::{Deserialize, Serialize};
use skia_safe::{Color, Path};
use std::rc::Rc;

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct Location {
    pub name: String,
    pub x: f64,
    pub y: f64,
    pub population: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Geo {
    pub geo: Vec<Polygon>,
}

pub struct GeoWithPathAndCities {
    pub geo_with_path: Vec<GeoWithPath>,
    pub cities: Vec<Rc<Location>>,
}

pub struct GeoWithPath {
    pub polys: Vec<Path>,
}

pub const COLOR_PALETTE: [Color; 17] = [
    Color::from_rgb(0xA0, 0x64, 0x14), // Brighter Rust
    Color::from_rgb(0x5C, 0x9C, 0xD0), // Brighter Strong Steel Blue
    Color::from_rgb(0x98, 0x5A, 0xB5), // Brighter Royal Purple
    Color::from_rgb(0x2C, 0xA0, 0x2C), // Brighter Strong Forest Green
    Color::from_rgb(0xD0, 0xA0, 0x14), // Brighter Rich Gold
    Color::from_rgb(0x50, 0x3A, 0x3C), // Brighter Deep Mahogany Brown
    Color::from_rgb(0x00, 0x7D, 0x00), // Brighter Deep Bottle Green
    Color::from_rgb(0xE0, 0x42, 0x14), // Brighter Burnt Orange
    Color::from_rgb(0xA5, 0x14, 0x14), // Brighter Deep Crimson
    Color::from_rgb(0x72, 0x84, 0x40), // Brighter Dark Olive Green
    Color::from_rgb(0x78, 0x44, 0x28), // Brighter Dark Copper
    Color::from_rgb(0x9B, 0x14, 0x14), // Brighter Maroon
    Color::from_rgb(0x7A, 0x4A, 0x24), // Brighter Rich Umber
    Color::from_rgb(0x45, 0x50, 0x45), // Brighter Deep Jungle Green
    Color::from_rgb(0x00, 0x64, 0xA5), // Brighter Dark Cerulean
    Color::from_rgb(0x63, 0x14, 0xA5), // Brighter Indigo
    Color::from_rgb(0xFF, 0xB5, 0x14), // Brighter Vivid Amber
];
