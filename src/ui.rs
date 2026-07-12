use ratatui::prelude::*;
use ratatui::widgets::*;
use crate::app::{App, Tab};
use crate::player::Player;
use crate::radio::RadioPlayer;
use crate::stations::Station;
use std::time::Duration;

fn format_duration(d: Duration) -> String {
    let secs = d.as_secs();
    let m = secs / 60;
    let s = secs % 60;
    format!("{:02}:{:02}", m, s)
}

pub fn draw(
    frame: &mut Frame,
    app: &App,
    player: &Player,
    radio: &RadioPlayer,
    stations: &[Station],
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(frame.area());

    let tabs = Tabs::new(vec!["♪ Tracks", "◉ Radio"])
        .select(match app.tab { Tab::Tracks => 0, Tab::Radio => 1 })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" chiplay ").title_style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)));
    frame.render_widget(tabs, chunks[0]);

    let (now_playing, pos, dur, paused) = if app.radio_playing && radio.playing {
        let name = if radio.station_name.is_empty() {
            "Radio".to_string()
        } else {
            format!("📻 {}", radio.station_name)
        };
        (name, Duration::ZERO, None, radio.is_paused())
    } else {
        (app.playing_name(), player.position(), player.duration(), player.is_paused())
    };

    let status_icon = if paused { "⏸" } else { "▶" };
    let vol = if app.radio_playing { radio.volume() } else { player.volume() };
    let vol_pct = (vol * 100.0).round() as u32;
    let vol_bars = (vol * 10.0).round() as usize;
    let vol_str: String = "█".repeat(vol_bars) + &"░".repeat(10usize.saturating_sub(vol_bars));

    let progress_text = match dur {
        Some(d) if d.as_secs() > 0 => {
            format!("{} {} / {}  Vol [{}] {}%", status_icon, format_duration(pos), format_duration(d), vol_str, vol_pct)
        }
        _ if app.radio_playing => {
            format!("{} LIVE  Vol [{}] {}%", status_icon, vol_str, vol_pct)
        }
        _ => {
            format!("{} {}  Vol [{}] {}%", status_icon, format_duration(pos), vol_str, vol_pct)
        }
    };

    let progress_ratio = match dur {
        Some(d) if d.as_secs() > 0 => (pos.as_secs_f64() / d.as_secs_f64()).min(1.0),
        _ => 0.0,
    };

    let gauge = Gauge::default()
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", now_playing)))
        .gauge_style(Style::default().fg(Color::Cyan).bg(Color::DarkGray))
        .ratio(progress_ratio)
        .label(progress_text);
    frame.render_widget(gauge, chunks[1]);

    match app.tab {
        Tab::Tracks => {
            let items: Vec<ListItem> = app.tracks.iter().enumerate().map(|(i, path)| {
                let name = path.file_stem().unwrap_or_default().to_string_lossy();
                let is_playing = app.playing_index == Some(i) && !app.radio_playing;
                let prefix = if is_playing { "♪ " } else { "  " };
                let style = if is_playing {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{}{}", prefix, name)).style(style)
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!(" Tracks ({}) ", app.tracks.len())))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
                .highlight_symbol("▸ ");

            let mut state = ListState::default();
            if !app.tracks.is_empty() {
                state.select(Some(app.cursor.min(app.tracks.len() - 1)));
            }
            frame.render_stateful_widget(list, chunks[2], &mut state);
        }
        Tab::Radio => {
            let items: Vec<ListItem> = stations.iter().map(|s| {
                let is_playing = app.radio_playing && radio.playing && radio.station_name == s.name;
                let prefix = if is_playing { "♪ " } else { "  " };
                let style = if is_playing {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(format!("{}{} [{}]", prefix, s.name, s.genre)).style(style)
            }).collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(format!(" Radio ({}) ", stations.len())))
                .highlight_style(Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD))
                .highlight_symbol("▸ ");

            let mut state = ListState::default();
            if !stations.is_empty() {
                state.select(Some(app.radio_index.min(stations.len() - 1)));
            }
            frame.render_stateful_widget(list, chunks[2], &mut state);
        }
    }

    let shuffle_str = if app.shuffle { "ON" } else { "OFF" };
    let repeat_str = app.repeat.label();
    let status = if let Some(msg) = &app.status_message {
        msg.clone()
    } else {
        format!("Shuffle: {}  |  Repeat: {}", shuffle_str, repeat_str)
    };
    let status_bar = Paragraph::new(status)
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::Gray));
    frame.render_widget(status_bar, chunks[3]);

    let help = " Space:play/pause  n/p:next/prev  +/-:vol  ←/→:seek  s:shuffle  r:repeat  Tab:switch  Enter:play  q:quit ";
    let help_bar = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray));
    frame.render_widget(help_bar, chunks[4]);
}
