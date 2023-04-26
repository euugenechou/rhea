use std::{env, io, path::PathBuf};
use thiserror::Error;
use toml::{de, ser};

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] io::Error),

    #[error("invalid path: {path:?}")]
    InvalidPath { path: PathBuf },

    #[error("disk exists")]
    DiskExists { name: String },

    #[error("disk in use: {name}")]
    DiskInUse { name: String },

    #[error("disk not in use: {name}")]
    DiskNotInUse { name: String },

    #[error("invalid disk: {name}")]
    InvalidDisk { name: String },

    #[error("machine exists: {name}")]
    MachineExists { name: String },

    #[error("machine in use: {name}")]
    MachineInUse { name: String },

    #[error("machine not in use: {name}")]
    MachineNotInUse { name: String },

    #[error("invalid machine: {name}")]
    InvalidMachine { name: String },

    #[error("snapshot exists: {name}")]
    SnapshotExists { name: String },

    #[error("snapshot in use: {name}")]
    SnapshotInUse { name: String },

    #[error("snapshot not in use: {name}")]
    SnapshotNotInUse { name: String },

    #[error("invalid snapshot: {name}")]
    InvalidSnapshot { name: String },

    #[error("missing environment variable")]
    MissingEnvVar(#[from] env::VarError),

    #[error("deserialization error")]
    Deserialization(#[from] de::Error),

    #[error("serialization error")]
    Serialization(#[from] ser::Error),

    #[error("unknown error")]
    Unknown,
}
