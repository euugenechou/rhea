use std::{env, io};
use thiserror::Error;
use toml::{de, ser};

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    IO(#[from] io::Error),

    #[error("bad path")]
    BadPath,

    #[error("machine already in-use")]
    InUse,

    #[error("invalid disk: {name}")]
    InvalidDisk { name: String },

    #[error("invalid machine: {name}")]
    InvalidMachine { name: String },

    #[error("missing environment variable")]
    EnvVar(#[from] env::VarError),

    #[error("deserialization error")]
    Deserialization(#[from] de::Error),

    #[error("serialization error")]
    Serialization(#[from] ser::Error),

    #[error("unknown error")]
    Unknown,
}
