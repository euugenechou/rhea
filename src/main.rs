mod cli;
use cli::{Args, Subcommands};

mod tables;
use tables::{DiskTable, MachineTable};

use anyhow::Result;
use clap::Parser;
use path_macro::path;
use rhea::State;
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
            state.start(&name, cores, ram, false, &[], Some(iso))?;
        }
        Subcommands::RemoveMachine { name } => {
            state.remove_machine(&name)?;
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
