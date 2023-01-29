use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Machine {
    pub name: String,
    pub port: u16,
    pub size: usize,
}
