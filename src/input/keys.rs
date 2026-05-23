use crossterm::event::KeyCode;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Quit,
    Back,
    ShowHelp,
}

pub fn global_action(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        KeyCode::Esc => Some(Action::Back),
        _ => None,
    }
}
