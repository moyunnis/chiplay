mod app;
mod events;
mod player;
mod radio;
mod stations;
mod track;
mod ui;

use app::{App, RepeatMode, Tab};
use clap::Parser;
use crossterm::{execute, terminal};
use events::{poll_event, AppEvent};
use player::Player;
use radio::RadioPlayer;
use ratatui::prelude::*;
use stations::builtin_stations;
use std::io;
use std::path::PathBuf;
use track::Track;

#[derive(Parser)]
#[command(
    name = "chiplay",
    version,
    about = "CLI music player with a TUI and internet radio"
)]
struct Cli {
    /// Audio files or directories to play (defaults to the current directory)
    paths: Vec<PathBuf>,

    /// Open the radio tab on startup
    #[arg(long)]
    radio: bool,

    /// Play a custom radio stream URL immediately
    #[arg(long, value_name = "URL")]
    radio_url: Option<String>,
}

const AUDIO_EXTS: &[&str] = &["mp3", "flac", "ogg", "wav", "m4a", "aac"];

fn is_audio(p: &std::path::Path) -> bool {
    p.extension()
        .map(|ext| AUDIO_EXTS.contains(&ext.to_string_lossy().to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Recursively collect audio files under the given paths, sorted.
fn collect_paths(paths: &[PathBuf], out: &mut Vec<PathBuf>) {
    for path in paths {
        if path.is_file() {
            if is_audio(path) {
                out.push(path.clone());
            }
        } else if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut children: Vec<PathBuf> =
                    entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                children.sort();
                collect_paths(&children, out);
            }
        }
    }
}

fn scan_tracks(paths: &[PathBuf]) -> Vec<Track> {
    let mut files = Vec::new();
    collect_paths(paths, &mut files);
    files.into_iter().map(Track::from_path).collect()
}

fn load_track(app: &mut App, player: &mut Player, radio_player: &mut RadioPlayer) {
    if let Some(path) = app.playing_path() {
        radio_player.stop();
        app.radio_playing = false;
        if let Err(e) = player.load(&path) {
            app.status_message = Some(format!("Error: {}", e));
        } else {
            app.status_message = None;
        }
    }
}

fn restore_terminal() {
    let _ = terminal::disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    );
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let tracks = if cli.paths.is_empty() && cli.radio_url.is_none() && !cli.radio {
        scan_tracks(&[PathBuf::from(".")])
    } else {
        scan_tracks(&cli.paths)
    };

    let mut app = App::new(tracks);
    let mut player = Player::new();
    let mut radio_player = RadioPlayer::new();
    let stations = builtin_stations();

    if cli.radio || cli.radio_url.is_some() {
        app.tab = Tab::Radio;
    }

    if let Some(url) = &cli.radio_url {
        app.radio_playing = true;
        if let Err(e) = radio_player.play_url(url, "Custom") {
            app.status_message = Some(format!("Radio error: {}", e));
            app.radio_playing = false;
        }
    }

    // Restore the terminal on panic so a crash never leaves it garbled.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    terminal::enable_raw_mode()?;
    execute!(
        io::stdout(),
        terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    )?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let result = run(&mut terminal, &mut app, &mut player, &mut radio_player, &stations);

    restore_terminal();
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    player: &mut Player,
    radio_player: &mut RadioPlayer,
    stations: &[stations::Station],
) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app, player, radio_player, stations))?;

        match poll_event(app.search_mode) {
            AppEvent::Quit => app.running = false,

            // --- search mode ---
            AppEvent::StartSearch => {
                if app.tab == Tab::Tracks {
                    app.start_search();
                }
            }
            AppEvent::SearchChar(c) => app.push_query(c),
            AppEvent::SearchBackspace => app.pop_query(),
            AppEvent::SearchConfirm => app.end_search(false),
            AppEvent::SearchCancel => app.end_search(true),

            // --- playback ---
            AppEvent::TogglePause => {
                if app.radio_playing {
                    radio_player.toggle_pause();
                } else {
                    player.toggle_pause();
                }
            }
            AppEvent::NextTrack => {
                if !app.filtered.is_empty() {
                    app.advance_track();
                    load_track(app, player, radio_player);
                }
            }
            AppEvent::PrevTrack => {
                if !app.filtered.is_empty() {
                    app.retreat_track();
                    load_track(app, player, radio_player);
                }
            }
            AppEvent::VolumeUp => {
                if app.radio_playing {
                    radio_player.set_volume(radio_player.volume() + 0.05);
                } else {
                    player.volume_up();
                }
            }
            AppEvent::VolumeDown => {
                if app.radio_playing {
                    radio_player.set_volume(radio_player.volume() - 0.05);
                } else {
                    player.volume_down();
                }
            }
            AppEvent::SeekForward => {
                if !app.radio_playing {
                    player.seek_forward(5);
                }
            }
            AppEvent::SeekBackward => {
                if !app.radio_playing {
                    player.seek_backward(5);
                }
            }
            AppEvent::ToggleShuffle => app.toggle_shuffle(),
            AppEvent::ToggleRepeat => app.toggle_repeat(),
            AppEvent::SwitchTab => app.toggle_tab(),
            AppEvent::ScrollUp => app.scroll_up(),
            AppEvent::ScrollDown => app.scroll_down(stations.len()),
            AppEvent::Enter => match app.tab {
                Tab::Tracks => {
                    app.play_at_cursor();
                    load_track(app, player, radio_player);
                }
                Tab::Radio => {
                    if let Some(station) = stations.get(app.radio_index) {
                        player.stop();
                        app.radio_playing = true;
                        app.status_message = Some("Connecting...".to_string());
                        terminal.draw(|f| ui::draw(f, app, player, radio_player, stations))?;
                        match radio_player.play_url(&station.url, &station.name) {
                            Ok(()) => app.status_message = None,
                            Err(e) => {
                                app.status_message = Some(format!("Radio error: {}", e));
                                app.radio_playing = false;
                            }
                        }
                    }
                }
            },
            AppEvent::None => {}
        }

        // Auto-advance when the current track finishes.
        if !app.radio_playing && player.is_empty() && app.playing_index.is_some() {
            match app.repeat {
                RepeatMode::One => {
                    if let Some(path) = app.playing_path() {
                        let _ = player.load(&path);
                    }
                }
                RepeatMode::All => {
                    app.advance_track();
                    if let Some(path) = app.playing_path() {
                        let _ = player.load(&path);
                    }
                }
                RepeatMode::Off => {
                    if app.has_next() {
                        app.advance_track();
                        if let Some(path) = app.playing_path() {
                            let _ = player.load(&path);
                        }
                    } else {
                        app.playing_index = None;
                    }
                }
            }
        }
    }
    Ok(())
}
