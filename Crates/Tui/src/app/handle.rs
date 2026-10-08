//! Events in, state changes out.

use ratatui::layout::Position;

use super::{App, Effect, SITE_URL, Screen};
use crate::binding::{HELP, QUIT};
use crate::event::Event;
use crate::hit::Target;
use crate::input::{Button, Input, Key, KeyKind, Mouse, MouseKind};
use crate::screens::{
    Action, Dest, ENTRIES, HistoryAction, HistoryPhase, MenuAction, Phase, PlaceAction, SetupAction,
};

impl App {
    pub(crate) fn handle(&mut self, event: Event) {
        match event {
            Event::Term(Input::Key(key)) => self.on_key(key),
            Event::Term(Input::Mouse(mouse)) => self.on_mouse(mouse),
            // A resize needs no state change: the next draw lays out again and rebuilds the hit map.
            Event::Term(Input::Resize | Input::Ignored) => {}
            Event::Job { id, job } => self.on_job(id, job),
            Event::InputFailed(message) => {
                self.failure = Some(message);
                self.quit = true;
            }
        }
    }

    fn on_key(&mut self, key: Key) {
        if key.kind == KeyKind::Release {
            return;
        }
        if QUIT.matches(key) {
            self.ctrl_c();
        } else if self.help {
            self.help = false;
        } else if HELP.matches(key) && !self.typing() {
            self.help = true;
        } else {
            self.route_key(key);
        }
    }

    /// Ctrl-C ends the program, except while a job is writing: leaving then would stop it half way, so
    /// the first press only says so and the second one quits.
    fn ctrl_c(&mut self) {
        if self.writing() && !self.quit_warned {
            self.quit_warned = true;
        } else {
            self.quit = true;
        }
    }

    /// Whether keys currently go into a text field.
    fn typing(&self) -> bool {
        match &self.screen {
            Screen::Work(w) => w.filtering && matches!(w.phase, Phase::Select),
            Screen::Place(p) => p.typing(),
            Screen::Menu(_) | Screen::History(_) | Screen::Setup(_) => false,
        }
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
            Screen::History(history) => {
                let action = history.on_key(key);
                self.history_act(action);
            }
            Screen::Place(place) => {
                let action = place.on_key(key);
                self.place_act(action);
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
            SetupAction::Leave => {
                self.checking = None;
                self.back_to_menu();
            }
            SetupAction::Check(path) => {
                let id = self.new_job();
                self.checking = Some(id);
                crate::jobs::spawn_check(self.tx.clone(), id, path);
            }
            SetupAction::Save(request) => self.save_setup(&request),
        }
    }

    fn open_entry(&mut self, index: usize) {
        match ENTRIES.get(index).map(|e| e.dest) {
            Some(Dest::Work(mode)) => self.open(mode),
            Some(Dest::History) => self.open_history(),
            Some(Dest::Place(purpose)) => self.open_place(purpose),
            Some(Dest::Setup) => self.open_setup(),
            None => {}
        }
    }

    fn act(&mut self, action: Action) {
        // One job at a time: nothing starts while another one is writing.
        if self.writing() && !matches!(action, Action::None) {
            return;
        }
        match action {
            Action::None => {}
            Action::Back => {
                // Leaving the page while it still works out a plan drops that plan.
                self.running = None;
                self.back_to_menu();
            }
            Action::Plan(pending) => self.plan(pending),
            Action::Diff(pending) => self.diff(pending),
            Action::Run(pending) => self.start(pending),
        }
    }

    fn place_act(&mut self, action: PlaceAction) {
        match action {
            PlaceAction::None => {}
            PlaceAction::Leave => {
                self.checking = None;
                self.back_to_menu();
            }
            PlaceAction::Check(purpose, path) => {
                let id = self.new_job();
                self.checking = Some(id);
                crate::jobs_install::spawn_place(self.tx.clone(), id, purpose, path);
            }
        }
    }

    fn history_act(&mut self, action: HistoryAction) {
        match action {
            HistoryAction::None => {}
            HistoryAction::Back => {
                // Leaving drops a read or a plan that is still on its way.
                self.running = None;
                self.listing = None;
                self.back_to_menu();
            }
            HistoryAction::Plan(run) => self.plan_undo(run),
            HistoryAction::Run(run) => self.start_undo(run),
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
            Screen::History(history) => {
                let action = history.on_mouse(mouse, &self.hits);
                self.history_act(action);
            }
            Screen::Place(place) => {
                let action = place.on_mouse(mouse, target);
                self.place_act(action);
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
            // The arrow must not walk away from a job that is writing.
            Some(Target::HeaderBack) if self.writing() => true,
            Some(Target::HeaderBack) => {
                self.running = None;
                self.listing = None;
                match &mut self.screen {
                    Screen::Work(work) if matches!(work.phase, Phase::Confirm(_)) => {
                        work.phase = Phase::Select;
                    }
                    Screen::Work(work) if matches!(work.phase, Phase::Diff(_)) => {
                        work.return_to_question();
                    }
                    Screen::History(history)
                        if matches!(history.phase, HistoryPhase::Confirm(_)) =>
                    {
                        history.phase = HistoryPhase::List;
                    }
                    _ => self.back_to_menu(),
                }
                true
            }
            _ => false,
        }
    }
}
