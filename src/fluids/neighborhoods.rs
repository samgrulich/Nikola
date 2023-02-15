use bevy::utils::HashMap;

pub struct Neighborhoods {
    entries: HashMap,
}

impl Neighborhoods {
    pub fn new() -> Self {
        Neighborhoods {
            entries: HashMap::new(),
        }
    }
}
