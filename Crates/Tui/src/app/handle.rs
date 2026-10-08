//! Events in, state changes out.

use ratatui::layout::Position;

use super::{App, Effect, SITE_URL, Screen};
use crate::binding::{HELP, QUIT};
use crate::event::Event;
use crate::hit::Target;
use crate::input::{Button, Input, Key, KeyKind, Mouse, MouseKind};
use crate::screens::{Action, Dest, ENTRIES, MenuAction, Phase, SetupAction};

impl App {
    pub(crate) fn handle(&mut self, event: Event) {
        match event {
            Event::Term(Input::Key(key)) => self.on_key(key),
            Event::Term(Input::Mouse(mouse)) => self.on_mouse(mouse),
            // A resize needs no state change: the next draw lays out again and rebuilds the hit map.
            Event::Term(Input::Resize | Input::Ignored) => {}
            Event::Job(job) => self.on_job(job),
        }
    }

    fn on_key(&mut self, key: Key) {
        if key.kind == KeyKind::Release {
            return;
        }
        if QUIT.matches(key) {
            self.quit = true;
        } else if self.help {
            self.help = false;
        } else if HELP.matches(key) && !self.typing() {
            self.help = true;
        } else {
            self.route_key(key);
        }
    }

    /// Whether keys currently go into a text field.
    fn typing(&self) -> bool {
        matches!(&self.screen, Screen::Work(w) if w.filtering && matches!(w.phase, Phase::Select))
    }

    fn route_key(&mut self, key: Key) {
        match &mut self.screen {
            Screen::Menu(menu) => match menu.on_key(key) {
                MenuAction::Open(i) => self.open_entry(i),
                MenuAction::Quit => self.quit = true,
                MenuAction::None => {}
            },
            Screen::Work(work) => {
                let action = work.on_key(key);
                self.act(action);
            }
            Screen::Setup(setup) => {
                let action = setup.on_key(key);
                self.setup_act(action);
            }
        }
    }

    fn setup_act(&mut self, action: SetupAction) {
        match action {
            SetupAction::None => {}
            SetupAction::Leave => self.back_to_menu(),
            SetupAction::Save(request) => self.save_setup(&request),
        }
    }

    fn open_entry(&mut self, index: usize) {
        match ENTRIES.get(index).map(|e| e.dest) {
            Some(Dest::Work(mode)) => self.open(mode),
            Some(Dest::Setup) => self.open_setup(),
            None => {}
        }
    }

    fn act(&mut self, action: Action) {
        match action {
            Action::None => {}
            Action::Back => self.back_to_menu(),
            Action::Run(pending) => self.start(pending),
        }
    }

    fn on_mouse(&mut self, mouse: Mouse) {
        let pos = Position::new(mouse.column, mouse.row);
        let target = self.hits.at(pos);
        if matches!(mouse.kind, MouseKind::Moved | MouseKind::Down(_)) {
            self.hover = target;
        }
        if self.help {
            if matches!(mouse.kind, MouseKind::Down(Button::Left))
                && matches!(target, Some(Target::Dismiss))
            {
                self.help = false;
            }
            return;
        }
        if matches!(mouse.kind, MouseKind::Down(Button::Left)) && self.header_click(target) {
            return;
        }
        match &mut self.screen {
            Screen::Menu(menu) => match (mouse.kind, target) {
                (MouseKind::Moved, Some(Target::MenuItem(i))) => menu.cursor = i,
                (MouseKind::Down(Button::Left), Some(Target::MenuItem(i))) => {
                    menu.cursor = i;
                    self.open_entry(i);
                }
                _ => {}
            },
            Screen::Work(work) => {
                let action = work.on_mouse(mouse, &self.hits);
                self.act(action);
            }
            Screen::Setup(setup) => {
                let action = setup.on_mouse(mouse, target);
                self.setup_act(action);
            }
        }
    }

    fn header_click(&mut self, target: Option<Target>) -> bool {
        match target {
            Some(Target::HeaderLink) => {
                self.effects.push(Effect::OpenUrl(SITE_URL.to_string()));
                true
            }
            Some(Target::HeaderBack) => {
                match &mut self.screen {
                    Screen::Work(work) if matches!(work.phase, Phase::Confirm(_)) => {
                        work.phase = Phase::Select;
                    }
                    _ => self.back_to_menu(),
                }
                true
            }
            _ => false,
        }
    }
}
