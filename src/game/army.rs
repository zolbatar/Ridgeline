use crate::game::units::UnitTrait;

pub struct Army {
    pub name: String,
}

impl UnitTrait for Army {
    fn get_name(&self) -> &str {
        self.name.as_str()
    }
}
