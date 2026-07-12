# chiplay

A fast, lightweight CLI music player with a terminal UI and internet radio — written in Rust.

Play your local library or tune into online stations, all from a keyboard-driven TUI. No Electron, no browser, no bloat — a single ~4 MB binary.

```
┌ chiplay ─────────────────────────────────────────────────────────────────────┐
│ ♪ Tracks │ ◉ Radio                                                            │
└────────────────────────────────────────────────────────────────────────────┘
┌ give yourself a break ───────────────────────────────────────────────────────┐
│██████████            ▶ 01:12 / 03:15   Vol [█████░░░░░] 50%                   │
└────────────────────────────────────────────────────────────────────────────┘
┌ Tracks (14) ─────────────────────────────────────────────────────────────────┐
│ ♪ give yourself a break                                                      │
│ ▸ midnight drive                                                             │
│   ocean floor                                                               │
└────────────────────────────────────────────────────────────────────────────┘
```

## Features

- 🎵 **Local playback** — MP3, FLAC, OGG, WAV, M4A, AAC
- 📻 **Internet radio** — built-in stations + your own custom streams
- ⌨️ **Keyboard-driven TUI** — progress bar, volume meter, track list
- 🔀 **Shuffle & repeat** — off / one / all
- ⏩ **Seeking** — jump forward/back within a track
- 🪶 **Tiny & fast** — single static binary, instant startup

## Install

### From source (requires [Rust](https://rustup.rs/))

```bash
git clone https://codeberg.org/moyunni/chiplay
cd chiplay
cargo install --path .
```

This installs `chiplay` to `~/.cargo/bin/`.

### Linux dependencies

chiplay uses ALSA for audio output. On Debian/Ubuntu:

```bash
sudo apt install libasound2-dev
```

On Arch Linux, ALSA is already part of `base`.

## Usage

```bash
chiplay                 # play audio files in the current directory
chiplay ~/Music         # play everything in a folder
chiplay track.mp3       # play a single file
chiplay --radio         # open the radio tab
chiplay --radio-url URL # play a custom stream immediately
```

## Controls

| Key           | Action                       |
|---------------|------------------------------|
| `Space`       | Play / pause                 |
| `Enter`       | Play selected track/station  |
| `↑` / `↓` (`k`/`j`) | Move selection         |
| `n` / `p`     | Next / previous track        |
| `+` / `-`     | Volume up / down             |
| `←` / `→` (`h`/`l`) | Seek −5s / +5s         |
| `s`           | Toggle shuffle               |
| `r`           | Cycle repeat (off/one/all)   |
| `Tab`         | Switch Tracks ↔ Radio        |
| `q` / `Esc`   | Quit                         |

## Custom radio stations

Add your own stations in `~/.config/chiplay/stations.txt`, one per line:

```
# Name | URL | Genre (genre is optional)
My Station | https://example.com/stream.mp3 | Ambient
Lo-Fi Beats | https://example.com/lofi.aac
```

They appear in the Radio tab alongside the built-ins.

## Built-in stations

Европа Плюс · Record · Русское Радио · DFM · Maximum · Наше Радио · Jazz FM · FIP (France) · Radio Paradise

## Built with

[ratatui](https://github.com/ratatui/ratatui) · [rodio](https://github.com/RustAudio/rodio) · [symphonia](https://github.com/pdeljanov/Symphonia) · [crossterm](https://github.com/crossterm-rs/crossterm) · [reqwest](https://github.com/seanmonstar/reqwest) · [clap](https://github.com/clap-rs/clap)

## License

MIT — see [LICENSE](LICENSE).
