use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone)]
pub struct Machine {
    pub name: String,
    pub port: u16,
    pub size: usize,
}

impl fmt::Display for Machine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (port: {}, size: {})",
            self.name, self.port, self.size
        )
    }
}
