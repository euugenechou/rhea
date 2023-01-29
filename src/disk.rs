use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Disk {
    pub name: String,
    pub size: usize,
}
