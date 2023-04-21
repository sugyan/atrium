use std::ffi::OsStr;
use std::fs::read_dir;
use std::io::Result;
use std::path::{Path, PathBuf};

fn walk<F>(path: &Path, results: &mut Vec<PathBuf>, f: &mut F) -> Result<()>
where
    F: FnMut(&Path) -> bool,
{
    if f(path) {
        results.push(path.into());
    }
    if path.is_dir() {
        for entry in read_dir(path)? {
            walk(&entry?.path(), results, f)?;
        }
    }
    Ok(())
}

pub(crate) fn find_schemas(path: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    walk(path, &mut results, &mut |path| {
        path.extension().and_then(OsStr::to_str) == Some("json")
    })?;
    Ok(results)
}

pub(crate) fn find_dirs(path: &Path) -> Result<Vec<PathBuf>> {
    let mut results = Vec::new();
    walk(path, &mut results, &mut |path| path.is_dir())?;
    Ok(results)
}
