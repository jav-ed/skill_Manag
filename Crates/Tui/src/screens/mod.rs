//! The screens' state and input handling. Drawing lives in `ui`.

mod list_view;
mod menu;
mod picker;
mod setup;
mod setup_keys;
mod work;
mod work_keys;

pub(crate) use menu::{Dest, ENTRIES, Menu, MenuAction};
pub(crate) use picker::Picker;
pub(crate) use setup::{
    SaveRequest, Setup, SetupAction, SetupStart, Step, VaultCheck, check_vault,
};
pub(crate) use work::{Action, Pending, Phase, Work};

#[cfg(test)]
mod tests;
