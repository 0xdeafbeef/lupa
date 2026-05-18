use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::adapters;

const DEFAULT_IGNORES: &[&str] = &[
    ".git",
    ".jj",
    "target",
    "node_modules",
    "vendor",
    "dist",
    "build",
    ".next",
    ".turbo",
    ".cache",
    "__pycache__",
];

pub fn collect_supported_files(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if adapters::is_supported(path) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            collect_dir(path, &mut files);
        }
    }
    files.sort();
    files
}

fn collect_dir(path: &Path, out: &mut Vec<PathBuf>) {
    for entry in WalkBuilder::new(path)
        .hidden(false)
        .build()
        .filter_map(Result::ok)
    {
        let entry_path = entry.path();
        if should_skip(entry_path) {
            continue;
        }
        if entry_path.is_file() && adapters::is_supported(entry_path) {
            out.push(entry_path.to_path_buf());
        }
    }
}

fn should_skip(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| DEFAULT_IGNORES.contains(&name))
    })
}
