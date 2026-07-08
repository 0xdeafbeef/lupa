pub mod c_family;
pub mod config;
pub mod go;
pub mod javascript;
pub mod just;
pub mod kotlin;
pub mod markdown;
pub mod nix;
pub mod python;
pub mod rust;
pub mod svelte;
pub mod syntax_nodes;
pub mod typst;

use std::path::Path;

use crate::model::{FileMap, Language};

pub fn parse_source(path: &Path, language: Language, source: String) -> Result<FileMap, String> {
    match language {
        Language::Bash
        | Language::Cmake
        | Language::Css
        | Language::Dockerfile
        | Language::Fish
        | Language::Lua
        | Language::Nginx
        | Language::Proto => Ok(syntax_nodes::parse(path, language, source)),
        Language::C | Language::Cpp => Ok(c_family::parse(path, language, source)),
        Language::Go => Ok(go::parse(path, source)),
        Language::JavaScript | Language::Jsx | Language::Tsx | Language::TypeScript => {
            Ok(javascript::parse(path, language, source))
        }
        Language::Json => Ok(config::parse_json(path, source)),
        Language::Just => Ok(just::parse(path, source)),
        Language::Kotlin => Ok(kotlin::parse(path, source)),
        Language::Markdown => Ok(markdown::parse(path, source)),
        Language::Nix => Ok(nix::parse(path, source)),
        Language::Python => Ok(python::parse(path, source)),
        Language::Rust => Ok(rust::parse(path, source)),
        Language::Svelte => Ok(svelte::parse(path, source)),
        Language::Toml => Ok(config::parse_toml(path, source)),
        Language::Typst => Ok(typst::parse(path, source)),
        Language::Yaml => Ok(config::parse_yaml(path, source)),
    }
}
