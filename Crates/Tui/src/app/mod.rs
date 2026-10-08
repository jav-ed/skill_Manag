//! Application state and the screen router. Nothing in here touches the terminal, so the whole
//! interface can be driven by tests with synthetic events.

mod handle;
mod jobs_events;

use std::sync::Arc;
use std::sync::mpsc::Sender;

use skillmirror_core::config::{ConfigError, Dirs, Settings, Source};

use crate::event::Event;
use crate::hit::{HitMap, Target};
use crate::items::Mode;
use crate::screens::{Dest, ENTRIES, Menu, Phase, Setup, SetupStart, Work};
use crate::session::Session;

pub(crate) const SITE_URL: &str = "https://javedab.com";

pub(crate) enum Screen {
    Menu(Menu),
    Work(Box<Work>),
    Setup(Box<Setup>),
}

/// What the interface needs from its caller.
pub struct Launch {
    pub settings: Settings,
    pub dirs: Dirs,
    /// Loads the settings again with the same flags and environment, after the wizard saved new ones.
    pub reload: Box<dyn Fn() -> Result<Settings, ConfigError>>,
}

/// Something to do outside the interface; the main loop performs it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Effect {
    OpenUrl(String),
}

pub(crate) struct App {
    pub(crate) screen: Screen,
    pub(crate) hits: HitMap,
    pub(crate) hover: Option<Target>,
    pub(crate) tick: usize,
    pub(crate) help: bool,
    quit: bool,
    effects: Vec<Effect>,
    session: Option<Arc<Session>>,
    loading: bool,
    settings: Settings,
    dirs: Dirs,
    reload: Box<dyn Fn() -> Result<Settings, ConfigError>>,
    tx: Sender<Event>,
}

impl App {
    pub(crate) fn new(launch: Launch, tx: Sender<Event>) -> Self {
        let Launch {
            settings,
            dirs,
            reload,
        } = launch;
        let unconfigured = settings.vault().is_err() || settings.root().is_err();
        let mut app = Self {
            screen: Screen::Menu(Menu::default()),
            hits: HitMap::default(),
            hover: None,
            tick: 0,
            help: false,
            quit: false,
            effects: Vec::new(),
            session: None,
            loading: false,
            settings,
            dirs,
            reload,
            tx,
        };
        if unconfigured {
            app.open_setup();
        }
        app
    }

    pub(crate) fn should_quit(&self) -> bool {
        self.quit
    }

    /// Spinner and progress need ticks; everything else is event driven, so an idle UI uses no CPU.
    pub(crate) fn animating(&self) -> bool {
        self.loading
            || matches!(&self.screen, Screen::Work(w) if matches!(w.phase, Phase::Running { .. } | Phase::Loading))
    }

    pub(crate) fn on_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub(crate) fn take_effects(&mut self) -> Vec<Effect> {
        std::mem::take(&mut self.effects)
    }

    /// The name shown in the header after the program name.
    pub(crate) fn screen_name(&self) -> Option<&'static str> {
        match &self.screen {
            Screen::Menu(_) => None,
            Screen::Setup(_) => Some("setup"),
            Screen::Work(work) => Some(match work.mode {
                Mode::Sync => "sync",
                Mode::Push => "push",
                Mode::Delete => "delete",
                Mode::List => "list",
            }),
        }
    }

    pub(crate) fn open(&mut self, mode: Mode) {
        let mut work = Work::new(mode);
        if let Some(session) = &self.session {
            work.populate(session);
        } else if !self.loading {
            self.loading = true;
            crate::jobs::spawn_load(self.tx.clone(), self.settings.clone());
        }
        self.screen = Screen::Work(Box::new(work));
    }

    /// Back to the menu with the cursor on the entry the user came from.
    pub(crate) fn back_to_menu(&mut self) {
        let cursor = match &self.screen {
            Screen::Work(work) => ENTRIES
                .iter()
                .position(|e| e.dest == Dest::Work(work.mode))
                .unwrap_or(0),
            Screen::Setup(_) => ENTRIES
                .iter()
                .position(|e| e.dest == Dest::Setup)
                .unwrap_or(0),
            Screen::Menu(menu) => menu.cursor,
        };
        self.screen = Screen::Menu(Menu { cursor });
    }
}

impl App {
    pub(crate) fn open_setup(&mut self) {
        let overridden = [self.settings.vault(), self.settings.root()]
            .into_iter()
            .flatten()
            .any(|v| matches!(v.source, Source::Flag | Source::Env));
        let note = overridden.then(|| {
            "A --vault or --root flag or an environment variable is set in this run and wins over what you save here."
                .to_string()
        });
        let start = SetupStart {
            home: self.dirs.home().to_path_buf(),
            pointer: self.dirs.pointer_file(),
            vault: self.settings.vault().ok().map(|v| v.value.clone()),
            root: self.settings.root().ok().map(|v| v.value.clone()),
            note,
        };
        self.screen = Screen::Setup(Box::new(Setup::new(start)));
    }

    /// Writes the pointer and the vault config, then loads the settings again.
    pub(crate) fn save_setup(&mut self, request: &crate::screens::SaveRequest) {
        let result = self.write_setup(request);
        if let Screen::Setup(setup) = &mut self.screen {
            setup.finish(result);
        }
    }

    fn write_setup(&mut self, request: &crate::screens::SaveRequest) -> Result<(), String> {
        use skillmirror_core::config::{ConfigUpdate, save_config, write_pointer};
        write_pointer(&self.dirs, &request.vault).map_err(|e| crate::session::describe(&e))?;
        let update = ConfigUpdate {
            root: Some(&request.root),
            mandatory: Some(&request.mandatory),
        };
        save_config(&request.vault, &update).map_err(|e| crate::session::describe(&e))?;
        self.settings = (self.reload)().map_err(|e| crate::session::describe(&e))?;
        // The scan belongs to the old settings.
        self.session = None;
        Ok(())
    }
}
