use rhea::State;
use std::fmt;
use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct DiskInfo {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "SIZE (GB)")]
    size: usize,
    #[tabled(rename = "IN-USE")]
    in_use: bool,
}

pub struct DiskTable {
    table: Table,
}

impl DiskTable {
    pub fn new(state: &State) -> Self {
        Self::filtered(state, &[])
    }

    pub fn filtered(state: &State, filter: &[&str]) -> Self {
        let mut table = Table::new(
            state
                .disks()
                .filter(|disk| filter.is_empty() || filter.contains(&disk.name.as_ref()))
                .map(|disk| DiskInfo {
                    name: disk.name.clone(),
                    size: disk.size,
                    in_use: state.disk_in_use(&disk.name).unwrap(),
                })
                .collect::<Vec<_>>(),
        );
        table.with(Style::blank());
        Self { table }
    }
}

impl fmt::Display for DiskTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.table)
    }
}

#[derive(Tabled)]
pub struct MachineInfo {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PORT")]
    port: u16,
    #[tabled(rename = "SIZE (GB)")]
    size: usize,
    #[tabled(rename = "IN-USE")]
    in_use: bool,
}

pub struct MachineTable {
    table: Table,
}

impl MachineTable {
    pub fn new(state: &State) -> Self {
        Self::filtered(state, &[])
    }

    pub fn filtered(state: &State, filter: &[&str]) -> Self {
        let mut table = Table::new(
            state
                .machines()
                .filter(|machine| filter.is_empty() || filter.contains(&machine.name.as_ref()))
                .map(|machine| MachineInfo {
                    name: machine.name.clone(),
                    port: machine.port,
                    size: machine.size,
                    in_use: state.machine_in_use(&machine.name).unwrap(),
                })
                .collect::<Vec<_>>(),
        );
        table.with(Style::blank());
        Self { table }
    }
}

impl fmt::Display for MachineTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.table)
    }
}
