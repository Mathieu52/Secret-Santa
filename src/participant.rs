use std::fmt::{Display, Formatter};

#[derive(Eq, PartialEq, Hash, Clone)]
pub struct Participant {
    pub name: String
}

impl Display for Participant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.name)
    }
}
