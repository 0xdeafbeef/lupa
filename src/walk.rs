use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use lupa::Language;

use crate::detect::LanguageDetector;

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

pub fn collect_supported_files(paths: &[PathBuf], detector: &mut LanguageDetector) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if path.is_file() {
            if is_supported_file(path, detector) {
                files.push(path.clone());
            }
        } else if path.is_dir() {
            collect_dir(path, detector, &mut files);
        }
    }
    files.sort();
    files
}

fn collect_dir(path: &Path, detector: &mut LanguageDetector, out: &mut Vec<PathBuf>) {
    for entry in WalkBuilder::new(path)
        .hidden(false)
        .build()
        .filter_map(Result::ok)
    {
        let entry_path = entry.path();
        if should_skip(entry_path) {
            continue;
        }
        if entry_path.is_file() && is_supported_file(entry_path, detector) {
            out.push(entry_path.to_path_buf());
        }
    }
}

fn is_supported_file(path: &Path, detector: &mut LanguageDetector) -> bool {
    if Language::from_path(path).is_some() {
        return true;
    }
    detector.detect_file(path).ok().flatten().is_some()
}

fn should_skip(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|name| DEFAULT_IGNORES.contains(&name))
    })
}
