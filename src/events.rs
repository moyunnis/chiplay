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
    Quit,
    None,
}

pub fn poll_event() -> AppEvent {
    if event::poll(Duration::from_millis(100)).unwrap_or(false) {
        if let Ok(Event::Key(key)) = event::read() {
            // Ignore key-release events (Windows / some terminals emit them).
            if key.kind == KeyEventKind::Release {
                return AppEvent::None;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                return AppEvent::Quit;
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
