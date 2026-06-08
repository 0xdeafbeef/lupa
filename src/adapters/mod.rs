pub mod c_family;
pub mod config;
pub mod go;
pub mod javascript;
pub mod markdown;
pub mod nix;
pub mod python;
pub mod rust;

use std::path::Path;

use crate::model::{FileMap, Language};

pub fn parse_file(path: &Path) -> Result<FileMap, String> {
    let language = Language::from_path(path)
        .ok_or_else(|| format!("# error: unsupported file type: {}", path.display()))?;
    let source = std::fs::read_to_string(path)
        .map_err(|err| format!("# error: failed to read {}: {err}", path.display()))?;

    parse_source(path, language, source)
}

pub fn parse_source(path: &Path, language: Language, source: String) -> Result<FileMap, String> {
    match language {
        Language::C | Language::Cpp => Ok(c_family::parse(path, language, source)),
        Language::Go => Ok(go::parse(path, source)),
        Language::JavaScript | Language::Jsx | Language::Tsx | Language::TypeScript => {
            Ok(javascript::parse(path, language, source))
        }
        Language::Json => Ok(config::parse_json(path, source)),
        Language::Markdown => Ok(markdown::parse(path, source)),
        Language::Nix => Ok(nix::parse(path, source)),
        Language::Python => Ok(python::parse(path, source)),
        Language::Rust => Ok(rust::parse(path, source)),
        Language::Toml => Ok(config::parse_toml(path, source)),
        Language::Yaml => Ok(config::parse_yaml(path, source)),
    }
}

pub fn is_supported(path: &Path) -> bool {
    Language::from_path(path).is_some()
}
