use std::{env, error::Error, fs, path::{Path, PathBuf}};

/// Unexpanded path to worlds folder.
pub fn get_worlds_folder() -> Result<PathBuf, Box<dyn Error>> {
    let path = if cfg!(target_os = "windows") {
        let profile = env::var("USERPROFILE")?;
        Path::new(&profile)
            .join(Path::new("AppData/Roaming/leafsodyssey_worlds/"))
    } else if cfg!(target_os = "macos") {
        let path = Path::new("~/Library/Application Support/leafsodyssey_worlds/");
        fs::canonicalize(path)?
    } else if cfg!(target_os = "linux") {
        let path = Path::new("~/.local/share/leafsodyssey_worlds/");
        fs::canonicalize(path)?
    } else {
        panic!("Unknown target OS, can't compute worlds folder.");
    };
    Ok(path)
}