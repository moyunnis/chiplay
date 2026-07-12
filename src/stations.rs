use std::fs;
use std::path::PathBuf;

pub struct Station {
    pub name: String,
    pub url: String,
    pub genre: String,
}

impl Station {
    fn new(name: &str, url: &str, genre: &str) -> Self {
        Station {
            name: name.to_string(),
            url: url.to_string(),
            genre: genre.to_string(),
        }
    }
}

fn builtin() -> Vec<Station> {
    vec![
        Station::new("Европа Плюс", "https://ep128.hostingradio.ru:8030/ep128", "Pop"),
        Station::new("Record", "https://radiorecord.hostingradio.ru/rr_main96.aacp", "Dance"),
        Station::new("Русское Радио", "https://rusradio.hostingradio.ru/rusradio128.mp3", "Pop"),
        Station::new("DFM", "https://dfm.hostingradio.ru/dfm96.aacp", "Dance"),
        Station::new("Maximum", "https://maximum.hostingradio.ru/maximum128.mp3", "Rock"),
        Station::new("Наше Радио", "https://nashe2.hostingradio.ru/nashe-128.mp3", "Rock"),
        Station::new("Jazz FM", "https://jazz-wr04.ice.infomaniak.ch/jazz-wr04-128.mp3", "Jazz"),
        Station::new("FIP (France)", "https://icecast.radiofrance.fr/fip-midfi.mp3", "Eclectic"),
        Station::new("Radio Paradise", "https://stream.radioparadise.com/aac-128", "Mix"),
    ]
}

/// Path to the user's custom stations file: ~/.config/chiplay/stations.txt
pub fn config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("chiplay").join("stations.txt"))
}

/// Parse custom stations from the config file.
/// Format, one per line: `Name | URL | Genre`  (Genre optional).
/// Lines starting with `#` and blank lines are ignored.
fn custom() -> Vec<Station> {
    let Some(path) = config_path() else {
        return Vec::new();
    };
    let Ok(content) = fs::read_to_string(&path) else {
        return Vec::new();
    };

    let mut stations = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
        match parts.as_slice() {
            [name, url] if !url.is_empty() => stations.push(Station::new(name, url, "Custom")),
            [name, url, genre, ..] if !url.is_empty() => {
                let genre = if genre.is_empty() { "Custom" } else { genre };
                stations.push(Station::new(name, url, genre));
            }
            _ => {}
        }
    }
    stations
}

pub fn builtin_stations() -> Vec<Station> {
    let mut all = builtin();
    all.extend(custom());
    all
}
