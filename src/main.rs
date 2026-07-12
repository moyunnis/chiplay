mod app;
mod events;
mod mpris;
mod player;
mod playlist;
mod radio;
mod spectrum;
mod stations;
mod track;
mod ui;

use app::{App, RepeatMode, Tab};
use clap::Parser;
use crossterm::{execute, terminal};
use events::{poll_event, AppEvent};
use mpris::Mpris;
use player::Player;
use radio::RadioPlayer;
use ratatui::prelude::*;
use spectrum::SharedSamples;
use stations::builtin_stations;
use std::io;
use std::path::{Path, PathBuf};
use track::Track;

#[derive(Parser)]
#[command(
    name = "chiplay",
    version,
    about = "CLI music player with a TUI and internet radio"
)]
struct Cli {
    paths: Vec<PathBuf>,

    #[arg(long)]
    radio: bool,

    #[arg(long, value_name = "URL")]
    radio_url: Option<String>,
}

const AUDIO_EXTS: &[&str] = &["mp3", "flac", "ogg", "wav", "m4a", "aac"];

fn is_audio(p: &Path) -> bool {
    p.extension()
        .map(|ext| AUDIO_EXTS.contains(&ext.to_string_lossy().to_lowercase().as_str()))
        .unwrap_or(false)
}

fn collect_paths(paths: &[PathBuf], out: &mut Vec<PathBuf>) {
    for path in paths {
        if playlist::is_playlist(path) {
            out.extend(playlist::load(path));
        } else if path.is_file() {
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

fn load_track(app: &mut App, player: &mut Player, radio_player: &mut RadioPlayer, mpris: &mut Option<Mpris>) {
    if let Some(path) = app.playing_path() {
        radio_player.stop();
        app.radio_playing = false;
        if let Err(e) = player.load(&path) {
            app.status_message = Some(format!("Error: {}", e));
        } else {
            app.status_message = None;
        }
        sync_mpris(mpris, app, player);
    }
}

fn sync_mpris(mpris: &mut Option<Mpris>, app: &App, player: &Player) {
    if let Some(m) = mpris {
        if let Some(t) = app.playing_track() {
            m.set_metadata(&t.title, t.artist.as_deref());
            m.set_playing(!player.is_paused());
        } else {
            m.set_stopped();
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
    let mut mpris = Mpris::new();
    let stations = builtin_stations();
    let samples = player.samples();

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

    let result = run(
        &mut terminal,
        &mut app,
        &mut player,
        &mut radio_player,
        &mut mpris,
        &stations,
        &samples,
    );

    restore_terminal();
    result
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    player: &mut Player,
    radio_player: &mut RadioPlayer,
    mpris: &mut Option<Mpris>,
    stations: &[stations::Station],
    samples: &SharedSamples,
) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| ui::draw(f, app, player, radio_player, stations, samples))?;

        for ev in mpris.as_ref().map(|m| m.poll()).unwrap_or_default() {
            handle_mpris(ev, app, player, radio_player, mpris);
        }

        match poll_event(app.search_mode) {
            AppEvent::Quit => app.running = false,

            AppEvent::StartSearch => {
                if app.tab == Tab::Tracks {
                    app.start_search();
                }
            }
            AppEvent::SearchChar(c) => app.push_query(c),
            AppEvent::SearchBackspace => app.pop_query(),
            AppEvent::SearchConfirm => app.end_search(false),
            AppEvent::SearchCancel => app.end_search(true),

            AppEvent::TogglePause => {
                if app.radio_playing {
                    radio_player.toggle_pause();
                } else {
                    player.toggle_pause();
                    sync_mpris(mpris, app, player);
                }
            }
            AppEvent::NextTrack => {
                if !app.filtered.is_empty() {
                    app.advance_track();
                    load_track(app, player, radio_player, mpris);
                }
            }
            AppEvent::PrevTrack => {
                if !app.filtered.is_empty() {
                    app.retreat_track();
                    load_track(app, player, radio_player, mpris);
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
            AppEvent::ToggleViz => app.toggle_viz(),
            AppEvent::SavePlaylist => {
                let dest = PathBuf::from("playlist.m3u");
                match playlist::save(&dest, &app.all_paths()) {
                    Ok(()) => app.status_message = Some(format!("Saved {} tracks to playlist.m3u", app.tracks.len())),
                    Err(e) => app.status_message = Some(format!("Save failed: {}", e)),
                }
            }
            AppEvent::SwitchTab => app.toggle_tab(),
            AppEvent::ScrollUp => app.scroll_up(),
            AppEvent::ScrollDown => app.scroll_down(stations.len()),
            AppEvent::Enter => match app.tab {
                Tab::Tracks => {
                    app.play_at_cursor();
                    load_track(app, player, radio_player, mpris);
                }
                Tab::Radio => {
                    if let Some(station) = stations.get(app.radio_index) {
                        player.stop();
                        app.radio_playing = true;
                        app.status_message = Some("Connecting...".to_string());
                        terminal.draw(|f| ui::draw(f, app, player, radio_player, stations, samples))?;
                        match radio_player.play_url(&station.url, &station.name) {
                            Ok(()) => app.status_message = None,
                            Err(e) => {
                                app.status_message = Some(format!("Radio error: {}", e));
                                app.radio_playing = false;
                            }
                        }
                        if let Some(m) = mpris {
                            m.set_stopped();
                        }
                    }
                }
            },
            AppEvent::None => {}
        }

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
                        sync_mpris(mpris, app, player);
                    }
                }
                RepeatMode::Off => {
                    if app.has_next() {
                        app.advance_track();
                        if let Some(path) = app.playing_path() {
                            let _ = player.load(&path);
                            sync_mpris(mpris, app, player);
                        }
                    } else {
                        app.playing_index = None;
                        if let Some(m) = mpris {
                            m.set_stopped();
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_mpris(
    ev: mpris::Event,
    app: &mut App,
    player: &mut Player,
    radio_player: &mut RadioPlayer,
    mpris: &mut Option<Mpris>,
) {
    use mpris::Event;
    match ev {
        Event::Play => {
            if !app.radio_playing {
                player.play();
                sync_mpris(mpris, app, player);
            }
        }
        Event::Pause => {
            if !app.radio_playing {
                player.pause();
                sync_mpris(mpris, app, player);
            }
        }
        Event::Toggle => {
            if app.radio_playing {
                radio_player.toggle_pause();
            } else {
                player.toggle_pause();
                sync_mpris(mpris, app, player);
            }
        }
        Event::Next => {
            if !app.filtered.is_empty() {
                app.advance_track();
                load_track(app, player, radio_player, mpris);
            }
        }
        Event::Previous => {
            if !app.filtered.is_empty() {
                app.retreat_track();
                load_track(app, player, radio_player, mpris);
            }
        }
        Event::Stop => {
            player.stop();
            app.playing_index = None;
            if let Some(m) = mpris {
                m.set_stopped();
            }
        }
        _ => {}
    }
}
