pub mod c_family;
pub mod go;
pub mod javascript;
pub mod markdown;
pub mod python;
pub mod rust;

use std::path::Path;

use crate::model::{FileMap, Language};

pub fn parse_file(path: &Path) -> Result<FileMap, String> {
    let language = Language::from_path(path)
        .ok_or_else(|| format!("# error: unsupported file type: {}", path.display()))?;
    let source = std::fs::read_to_string(path)
        .map_err(|err| format!("# error: failed to read {}: {err}", path.display()))?;

    match language {
        Language::C | Language::Cpp => Ok(c_family::parse(path, language, source)),
        Language::Go => Ok(go::parse(path, source)),
        Language::JavaScript | Language::Jsx | Language::Tsx | Language::TypeScript => {
            Ok(javascript::parse(path, language, source))
        }
        Language::Markdown => Ok(markdown::parse(path, source)),
        Language::Python => Ok(python::parse(path, source)),
        Language::Rust => Ok(rust::parse(path, source)),
    }
}

pub fn is_supported(path: &Path) -> bool {
    Language::from_path(path).is_some()
}
