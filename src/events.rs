use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use std::time::Duration;

pub enum AppEvent {
    TogglePause,
    NextTrack,
    PrevTrack,
    VolumeUp,
    VolumeDown,
    SeekForward,
    SeekBackward,
    ToggleShuffle,
    ToggleRepeat,
    SwitchTab,
    Enter,
    ScrollUp,
    ScrollDown,
    StartSearch,
    // Search-mode input
    SearchChar(char),
    SearchBackspace,
    SearchConfirm,
    SearchCancel,
    Quit,
    None,
}

pub fn poll_event(search_mode: bool) -> AppEvent {
    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            if key.kind == KeyEventKind::Release {
                return AppEvent::None;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return AppEvent::Quit;
            }

            if search_mode {
                return match key.code {
                    KeyCode::Enter => AppEvent::SearchConfirm,
                    KeyCode::Esc => AppEvent::SearchCancel,
                    KeyCode::Backspace => AppEvent::SearchBackspace,
                    KeyCode::Char(c) => AppEvent::SearchChar(c),
                    _ => AppEvent::None,
                };
            }

            return match key.code {
                KeyCode::Char(' ') => AppEvent::TogglePause,
                KeyCode::Char('n') => AppEvent::NextTrack,
                KeyCode::Char('p') => AppEvent::PrevTrack,
                KeyCode::Char('+') | KeyCode::Char('=') => AppEvent::VolumeUp,
                KeyCode::Char('-') | KeyCode::Char('_') => AppEvent::VolumeDown,
                KeyCode::Left | KeyCode::Char('h') => AppEvent::SeekBackward,
                KeyCode::Right | KeyCode::Char('l') => AppEvent::SeekForward,
                KeyCode::Char('s') => AppEvent::ToggleShuffle,
                KeyCode::Char('r') => AppEvent::ToggleRepeat,
                KeyCode::Char('/') => AppEvent::StartSearch,
                KeyCode::Tab => AppEvent::SwitchTab,
                KeyCode::Enter => AppEvent::Enter,
                KeyCode::Up | KeyCode::Char('k') => AppEvent::ScrollUp,
                KeyCode::Down | KeyCode::Char('j') => AppEvent::ScrollDown,
                KeyCode::Char('q') | KeyCode::Esc => AppEvent::Quit,
                _ => AppEvent::None,
            };
        }
    }
    AppEvent::None
}
