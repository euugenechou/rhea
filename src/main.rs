mod cli;
use cli::{Args, Subcommands};

mod tables;
use tables::{DiskTable, MachineTable, SnapshotTable};

use anyhow::Result;
use clap::Parser;
use path_macro::path;
use rhea::state::State;
use std::env;

fn main() -> Result<()> {
    let args = Args::parse();

    let path = path![env::var("HOME")? / ".config" / "rhea"];
    let mut state = State::load(path)?;

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
            state.start(&name, cores, ram, false, false, &[], Some(iso))?;
        }
        Subcommands::RemoveMachine { name } => {
            state.remove_machine(&name)?;
            state.save()?;
        }
        Subcommands::AddSnapshot { name, base } => {
            state.add_snapshot(&name, &base)?;
            state.save()?;
        }
        Subcommands::RemoveSnapshot { name } => {
            state.remove_snapshot(&name)?;
            state.save()?;
        }
        Subcommands::Disk { name } => {
            println!("{}", DiskTable::filtered(&state, &[&name]));
        }
        Subcommands::Disks => {
            println!("{}", DiskTable::new(&state));
        }
        Subcommands::Machine { name } => {
            println!("{}", MachineTable::filtered(&state, &[&name]));
        }
        Subcommands::Machines => {
            println!("{}", MachineTable::new(&state));
        }
        Subcommands::Snapshot { name } => {
            println!("{}", SnapshotTable::filtered(&state, &[&name]));
        }
        Subcommands::Snapshots => {
            println!("{}", SnapshotTable::new(&state));
        }
        Subcommands::Start {
            name,
            cores,
            ram,
            foreground,
            disks,
            snapshot,
        } => {
            state.start(&name, cores, ram, foreground, snapshot, &disks, None)?;
        }
        Subcommands::Stop { name, snapshot } => {
            state.stop(&name, snapshot)?;
        }
        Subcommands::Connect {
            forward_keys,
            username,
            name,
            snapshot,
        } => {
            state.connect(&name, username, forward_keys, snapshot)?;
        }
    };

    Ok(())
}
