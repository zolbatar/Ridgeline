use crate::game::units::UnitTrait;

pub struct Spy {
    pub name: String,
}

impl UnitTrait for Spy {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
