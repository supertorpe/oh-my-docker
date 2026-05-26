use crossterm::event::KeyCode;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Quit,
    Back,
    ShowHelp,
}

#[allow(dead_code)]
pub fn global_action(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        KeyCode::Esc => Some(Action::Back),
        _ => None,
    }
}
