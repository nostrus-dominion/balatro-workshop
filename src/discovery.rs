use std::{
    env, fs,
    path::{Path, PathBuf},
};

pub fn candidate_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(appdata) = env::var_os("APPDATA") {
        roots.push(PathBuf::from(appdata).join("Balatro"));
    }
    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        roots.push(home.join("Library/Application Support/Balatro"));
        roots.push(home.join(".local/share/love/Balatro"));
        roots.push(home.join(".local/share/Balatro"));
        let steam_roots = [
            home.join(".steam/steam"),
            home.join(".local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ];
        let mut libraries = steam_roots.to_vec();
        for steam in &steam_roots {
            // Steam's library paths are quoted strings in libraryfolders.vdf.
            if let Ok(text) = fs::read_to_string(steam.join("steamapps/libraryfolders.vdf")) {
                libraries.extend(library_paths(&text));
            }
        }
        for library in libraries {
            roots.push(library.join(
                "steamapps/compatdata/2379780/pfx/drive_c/users/steamuser/AppData/Roaming/Balatro",
            ));
        }
    }
    if let Some(data) = env::var_os("XDG_DATA_HOME") {
        roots.push(PathBuf::from(data).join("love/Balatro"));
    }
    roots.sort();
    roots.dedup();
    roots
}

pub fn library_paths(vdf: &str) -> Vec<PathBuf> {
    vdf.lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("\"path\"")?.trim();
            let literal = rest.strip_prefix('"')?.strip_suffix('"')?;
            Some(PathBuf::from(
                literal.replace("\\\\", "\\").replace("\\\"", "\""),
            ))
        })
        .collect()
}

pub fn scan(root: &Path) -> Vec<PathBuf> {
    fn visit(root: &Path, depth: usize, files: &mut Vec<PathBuf>) {
        if depth > 3 || files.len() >= 200 {
            return;
        }
        let Ok(entries) = fs::read_dir(root) else {
            return;
        };
        for entry in entries.flatten().take(1000) {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            let path = entry.path();
            if kind.is_dir() {
                visit(&path, depth + 1, files);
            } else if kind.is_file()
                && path
                    .extension()
                    .is_some_and(|v| v.eq_ignore_ascii_case("jkr"))
            {
                files.push(path);
            }
            if files.len() >= 200 {
                break;
            }
        }
    }
    let mut files = Vec::new();
    visit(root, 0, &mut files);
    files.sort();
    files
}

pub fn discover() -> Vec<PathBuf> {
    let mut files: Vec<_> = candidate_roots()
        .iter()
        .flat_map(|p| scan(p))
        .filter_map(|p| fs::canonicalize(p).ok())
        .collect();
    files.sort();
    files.dedup();
    files
}
