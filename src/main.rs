use anyhow::Result;
use clap::{Parser, Subcommand};
use path_macro::path;
use rhea::{Disk, Library, Machine};
use std::{env, path::PathBuf};

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
    RmDisk {
        /// Name of the disk
        #[arg(value_parser)]
        name: String,
    },
    /// Back up a disk
    BackupDisk {
        /// Name of the disk
        #[arg(value_parser)]
        name: String,
    },
    /// Add a new virtual machine
    AddVm {
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
    RmVm {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,
    },
    /// Back up a virtual machine
    BackupVm {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,
    },
    /// Run a virtual machine
    RunVm {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,

        /// Cores to allocate for the virtual machine
        #[arg(short, long, value_parser, default_value_t = 4)]
        cores: usize,

        /// RAM to allocate for the virtual machine (GB)
        #[arg(short, long, value_parser, default_value_t = 4)]
        ram: usize,

        /// Names of disks to attach to the virtual machine
        #[arg(short, long)]
        disks: Vec<String>,
    },
    /// Print the port assigned to a virtual machine
    Port {
        /// Name of the virtual machine
        #[arg(value_parser)]
        name: String,
    },
    /// List disks
    Disks,
    /// List virtual machines
    Vms,
    /// List disk backups
    DiskBackups,
    /// List virtual machine backups
    VmBackups,
    /// List ports used by virtual machines
    Ports,
    /// Connect to a running virtual machine
    Connect {
        /// Forward SSH keys
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

fn main() -> Result<()> {
    let args = Args::parse();

    let path = path!(env::var("HOME")? / ".config" / "rhea");
    let mut state = Library::load(path)?;

    match args.subcommand {
        Subcommands::AddDisk { name, size } => {
            state.add_disk(Disk { name, size })?;
            state.save()?;
        }
        Subcommands::RmDisk { name } => {
            state.remove_disk(&name)?;
            state.save()?;
        }
        Subcommands::BackupDisk { name } => {
            state.backup_disk(name)?;
            state.save()?;
        }
        Subcommands::AddVm {
            name,
            iso,
            size,
            cores,
            ram,
            port,
        } => {
            state.add_machine(Machine {
                name: name.clone(),
                port,
                size,
            })?;
            state.save()?;
            state.run_machine(name, cores, ram, &[], Some(iso))?;
        }
        Subcommands::RmVm { name } => {
            state.remove_machine(&name)?;
            state.save()?;
        }
        Subcommands::BackupVm { name } => {
            state.backup_machine(name)?;
            state.save()?;
        }
        Subcommands::RunVm {
            name,
            cores,
            ram,
            disks,
        } => {
            state.run_machine(name, cores, ram, &disks, None)?;
        }
        Subcommands::Port { name } => {
            println!("{}", state.get_machine_port(name)?);
        }
        Subcommands::Disks => {
            for disk in state.disks() {
                println!("{}", disk.name);
            }
        }
        Subcommands::Vms => {
            for machine in state.machines() {
                println!("{}", machine.name);
            }
        }
        Subcommands::DiskBackups => {
            for backup in state.disk_backups() {
                println!("{}", backup.name);
            }
        }
        Subcommands::VmBackups => {
            for backup in state.machine_backups() {
                println!("{}", backup.name);
            }
        }
        Subcommands::Ports => {
            for port in state.ports() {
                println!("{port}");
            }
        }
        Subcommands::Connect {
            forward_keys,
            username,
            name,
        } => {
            state.connect(name, username, forward_keys)?;
        }
    };

    Ok(())
}
