use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Deserialize, Serialize, Clone)]
pub struct Snapshot {
    pub name: String,
    pub base: String,
    pub port: u16,
    pub size: usize,
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (base: {}, port: {}, size: {})",
            self.name, self.base, self.port, self.size
        )
    }
}
