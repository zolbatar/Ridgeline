use crate::geo::data::Location;
use std::rc::Rc;

#[derive(PartialEq)]
pub struct City {
    pub location: Rc<Location>,
}
