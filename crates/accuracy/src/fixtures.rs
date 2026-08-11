use std::ffi::OsStr;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SUPPORTED_EXTENSIONS: [&str; 3] = ["rs", "ts", "tsx"];

/// Collect every fixture under `root`, in a stable order so reports and baselines are
/// reproducible.
pub fn discover(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = collect(root)?;
    paths.sort();
    Ok(paths)
}

/// Name a fixture is reported under: its path relative to the fixture root, with forward
/// slashes so baselines are identical across platforms.
pub fn fixture_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

fn collect(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<_>>>()?;

    let nested = entries
        .into_iter()
        .map(|path| match path.is_dir() {
            true => collect(&path),
            false => Ok(is_fixture(&path).then_some(path).into_iter().collect()),
        })
        .collect::<io::Result<Vec<_>>>()?;

    Ok(nested.into_iter().flatten().collect())
}

fn is_fixture(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| SUPPORTED_EXTENSIONS.contains(&extension))
}
