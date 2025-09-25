//! types and functions that are common across the client and server implementations

#![warn(missing_docs)]

use std::collections::HashMap;

use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
/// response from a server implementation
pub enum Response<E: std::error::Error>{
    /// error
    Error(ServerError<E>),
    /// a list of scaling governors
    ScalingGovernors(Vec<String>),
    /// scaling information
    Information(Information),
}

/// frequency information returned from a server implementation
#[derive(Encode, Decode)]
pub enum Information {
    /// information for all CPU cores
    All(PerCpuInformation),
    /// information for each individual CPU core
    Table(HashMap<u8, PerCpuInformation>)
}

/// information for a given CPU core or for all CPUs
#[derive(Encode, Decode)]
pub struct PerCpuInformation {
    /// the current scaling governor for this CPU
    pub governor: String,
    /// the clock speed in megahertz
    pub megahertz: Option<u64>,
}

/// errors returned from a server request
#[derive(Encode, Decode)]
pub enum ServerError<E: std::error::Error> {
    /// the server is not running
    NotRunning,
    /// the server is not running as root
    NotRoot,
    /// invalid scaling governor preset
    InvalidScalingGovernor,
    /// other error
    Other(E)
}

#[derive(Debug, Clone)]
/// A request sent to the server
pub enum Request {
    /// get information at a given [CpuCores]
    Get(CpuCores),
    /// set scaling for a [CpuCore]
    Set(CpuCores, ScalingType),
    /// list scaling governors for [CpuCores]
    List(CpuCores),
}

#[derive(Debug, Clone)]
/// scaling types. may be expanded to support setting the speed in hertz instead of using a preset
/// scaling governor
pub enum ScalingType {
    /// preset scaling governor
    Preset(String),
}

/// Type for cpu cores, can be
/// - [CpuCores::All]
/// - [CpuCores::One]
/// - [CpuCores::Multiple]
/// - [CpuCores::Range]
#[derive(Debug, Default, Clone)]
pub enum CpuCores {
    #[default]
    /// Apply to all CPU cores
    All,
    /// Only one CPU core (example: CPU4)
    One(u8),
    /// Multiple CPU cores (example, CPU0, CPU2)
    Multiple(Vec<u8>),
    /// A range of CPU cores (example: CPU0-5)
    Range(u8, u8),
}
