use anyhow::Result;
use clap::{Parser, Subcommand};
use path_macro::path;
use rhea::Library;
use std::{env, path::PathBuf};
use tabled::{settings::Style, Table, Tabled};

#[derive(Parser)]
#[command(author, version)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    /// Add a new disk
    AddDisk {
        /// Name of the disk
        #[arg(value_parser)]
        name: String,

        /// Size of the disk (GB)
        #[arg(value_parser)]
        size: usize,
    },
    /// Remove a disk
    RemoveDisk {
        /// Name of the disk
        #[arg(value_parser)]
        name: String,
    },
    /// Add a new virtual machine
    AddMachine {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,

        /// Image to install on the virtual machine
        #[arg(value_parser)]
        iso: PathBuf,

        /// Size of the virtual machine (GB)
        #[arg(short, long, value_parser, default_value_t = 128)]
        size: usize,

        /// Cores to allocate for the virtual machine
        #[arg(short, long, value_parser, default_value_t = 4)]
        cores: usize,

        /// RAM to allocate for the virtual machine (GB)
        #[arg(short, long, value_parser, default_value_t = 4)]
        ram: usize,

        /// Port to assign the virtual machine
        #[arg(short, long, value_parser, default_value_t = 8192)]
        port: u16,
    },
    /// Remove a virtual machine
    RemoveMachine {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,
    },
    /// Print information about a disk
    Disk {
        #[arg(value_parser)]
        name: String,
    },
    /// Print information about all disks
    Disks,
    /// Print information about a virtual machine
    Machine {
        #[arg(value_parser)]
        name: String,
    },
    /// Print information about all virtual machines
    Machines,
    /// Run a virtual machine
    Start {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,

        /// Cores to allocate for the virtual machine
        #[arg(short, long, value_parser, default_value_t = 4)]
        cores: usize,

        /// RAM to allocate for the virtual machine (GB)
        #[arg(short, long, value_parser, default_value_t = 4)]
        ram: usize,

        /// Run virtual machine in foreground.
        #[arg(short, long, default_value_t = false)]
        foreground: bool,

        /// Names of disks to attach to the virtual machine
        #[arg(short, long, value_delimiter = ',')]
        disks: Vec<String>,
    },
    /// Stop a virtual machine
    Stop {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,
    },
    /// Connect to a virtual machine
    Connect {
        /// Enable SSH agent forwarding
        #[arg(short = 'A', long, default_value_t = false)]
        forward_keys: bool,

        /// Username (default: $USER)
        #[arg(short, long)]
        username: Option<String>,

        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,
    },
}

#[derive(Tabled)]
struct DiskInfo {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "SIZE (GB)")]
    size: usize,
    #[tabled(rename = "IN-USE")]
    in_use: bool,
}

#[derive(Tabled)]
struct MachineInfo {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PORT")]
    port: u16,
    #[tabled(rename = "SIZE (GB)")]
    size: usize,
    #[tabled(rename = "IN-USE")]
    in_use: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let path = path!(env::var("HOME")? / ".config" / "rhea");
    let mut state = Library::load(path)?;

    match args.subcommand {
        Subcommands::AddDisk { name, size } => {
            state.add_disk(&name, size)?;
            state.save()?;
        }
        Subcommands::RemoveDisk { name } => {
            state.remove_disk(&name)?;
            state.save()?;
        }
        Subcommands::AddMachine {
            name,
            iso,
            size,
            cores,
            ram,
            port,
        } => {
            state.add_machine(&name, port, size)?;
            state.save()?;
            state.start(&name, cores, ram, false, &[], Some(iso))?;
        }
        Subcommands::RemoveMachine { name } => {
            state.remove_machine(&name)?;
            state.save()?;
        }
        Subcommands::Disk { name } => {
            println!(
                "{}",
                Table::new(
                    state
                        .disks()
                        .filter(|disk| disk.name == name)
                        .map(|disk| DiskInfo {
                            name: disk.name.clone(),
                            size: disk.size,
                            in_use: state.disk_in_use(&disk.name).unwrap(),
                        })
                        .collect::<Vec<_>>(),
                )
                .with(Style::blank())
                .to_string()
            );
        }
        Subcommands::Disks => {
            println!(
                "{}",
                Table::new(
                    state
                        .disks()
                        .map(|disk| DiskInfo {
                            name: disk.name.clone(),
                            size: disk.size,
                            in_use: state.disk_in_use(&disk.name).unwrap(),
                        })
                        .collect::<Vec<_>>(),
                )
                .with(Style::blank())
                .to_string()
            );
        }
        Subcommands::Machine { name } => {
            println!(
                "{}",
                Table::new(
                    state
                        .machines()
                        .filter(|machine| machine.name == name)
                        .map(|machine| MachineInfo {
                            name: machine.name.clone(),
                            port: machine.port,
                            size: machine.size,
                            in_use: state.machine_in_use(&machine.name).unwrap(),
                        })
                        .collect::<Vec<_>>(),
                )
                .with(Style::blank())
                .to_string()
            );
        }
        Subcommands::Machines => {
            println!(
                "{}",
                Table::new(
                    state
                        .machines()
                        .map(|machine| MachineInfo {
                            name: machine.name.clone(),
                            port: machine.port,
                            size: machine.size,
                            in_use: state.machine_in_use(&machine.name).unwrap(),
                        })
                        .collect::<Vec<_>>(),
                )
                .with(Style::blank())
                .to_string()
            );
        }
        Subcommands::Start {
            name,
            cores,
            ram,
            foreground,
            disks,
        } => {
            state.start(&name, cores, ram, foreground, &disks, None)?;
        }
        Subcommands::Stop { name } => {
            state.stop(&name)?;
        }
        Subcommands::Connect {
            forward_keys,
            username,
            name,
        } => {
            state.connect(&name, username, forward_keys)?;
        }
    };

    Ok(())
}
