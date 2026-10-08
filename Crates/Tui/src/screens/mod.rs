//! The screens' state and input handling. Drawing lives in `ui`.

mod history;
mod list_view;
mod menu;
mod picker;
mod place;
mod setup;
mod setup_keys;
mod work;
mod work_keys;

pub(crate) use history::{Action as HistoryAction, History, Phase as HistoryPhase};
pub(crate) use list_view::ListView;
pub(crate) use menu::{Dest, ENTRIES, Menu, MenuAction};
pub(crate) use picker::Picker;
pub(crate) use place::{Place, PlaceAction, Purpose, Stage};
pub(crate) use setup::{
    SaveRequest, Setup, SetupAction, SetupStart, Step, VaultCheck, check_vault,
};
pub(crate) use work::{Action, Install, Pending, Phase, Work};

#[cfg(test)]
mod tests;
