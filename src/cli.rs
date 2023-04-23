use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version)]
pub struct Args {
    #[clap(subcommand)]
    pub subcommand: Subcommands,
}

#[derive(Subcommand)]
pub enum Subcommands {
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
