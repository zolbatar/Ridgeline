use serde::{Deserialize, Serialize};
use skia_safe::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Geo {
    pub name: String,
    pub population: usize,
    pub map_colour: u8,
    pub geo: geo::Geometry<f64>,
}

pub struct GeoWithPath {
    pub name: String,
    pub population: usize,
    pub map_colour: u8,
    pub polys: Vec<Path>,
}
