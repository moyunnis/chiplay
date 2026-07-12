use std::io::Write;
use std::path::{Path, PathBuf};

pub fn load(path: &Path) -> Vec<PathBuf> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    let base = path.parent();
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let p = PathBuf::from(line);
        let resolved = if p.is_absolute() {
            p
        } else if let Some(base) = base {
            base.join(p)
        } else {
            p
        };
        if resolved.is_file() {
            out.push(resolved);
        }
    }
    out
}

pub fn save(path: &Path, paths: &[PathBuf]) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    writeln!(file, "#EXTM3U")?;
    for p in paths {
        writeln!(file, "{}", p.display())?;
    }
    Ok(())
}

pub fn is_playlist(path: &Path) -> bool {
    path.extension()
        .map(|e| e.eq_ignore_ascii_case("m3u") || e.eq_ignore_ascii_case("m3u8"))
        .unwrap_or(false)
}
