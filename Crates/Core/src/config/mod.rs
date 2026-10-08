//! Where configuration lives and how flags, environment and files combine.

mod env;
mod error;
mod migrate;
mod paths;
mod pointer;
mod save;
mod settings;
mod vault_config;

pub use env::EnvOverrides;
pub use error::ConfigError;
pub use migrate::{MigrateReport, migrate};
pub use paths::Dirs;
pub use pointer::{read_pointer, write_pointer};
pub use save::{ConfigUpdate, save_config};
pub use settings::{Flags, Settings, Source, Sourced};
pub use vault_config::{Profile, VaultConfig};

#[cfg(test)]
mod migrate_tests;
#[cfg(test)]
mod safety_tests;
#[cfg(test)]
mod save_tests;
#[cfg(test)]
mod tests;
