use crate::geo::data::Location;
use std::rc::Rc;

pub struct City {
    pub location: Rc<Location>,
}
