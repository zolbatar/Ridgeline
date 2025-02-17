use std::collections::HashMap;
use geo::Polygon;
use serde::{Deserialize, Serialize};
use skia_safe::{Image, Path};
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
    pub ways: HashMap<WayClass, Vec<WaySkia>>,
    pub dem: Image,
}

pub struct GeoWithPath {
    pub polys: Vec<Path>,
}

#[derive(PartialEq, Eq, Hash, Clone, Debug, Serialize, Deserialize)]
pub enum WayClass {
    BRoad,
    ARoad,
    Motorway,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WayForm {
    SingleCarriageway,
    DualCarriageway,
    CollapsedDualCarriageway,
    Roundabout,
    SlipRoad,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WayPoint {
    pub is_start: bool,
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Way {
    pub name: String,
    pub class: WayClass,
    pub form: WayForm,
    pub way_points: Vec<WayPoint>,
}

#[derive(Debug)]
pub struct WaySkia {
    pub class: WayClass,
    pub _form: WayForm,
    pub path: Path,
}
