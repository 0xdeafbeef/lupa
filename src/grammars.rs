use crate::model::Language;

pub(crate) fn language(language: Language) -> Option<tree_sitter::Language> {
    Some(match language {
        Language::Bash => tree_sitter_bash::LANGUAGE.into(),
        Language::C => tree_sitter_c::LANGUAGE.into(),
        Language::Cmake => tree_sitter_cmake::LANGUAGE.into(),
        Language::Cpp => tree_sitter_cpp::LANGUAGE.into(),
        Language::Css => tree_sitter_css::LANGUAGE.into(),
        Language::Dockerfile => tree_sitter_containerfile::LANGUAGE.into(),
        Language::Fish => tree_sitter_fish::language(),
        Language::Go => tree_sitter_go::LANGUAGE.into(),
        Language::JavaScript | Language::Jsx => tree_sitter_javascript::LANGUAGE.into(),
        Language::Json => tree_sitter_json::LANGUAGE.into(),
        Language::Just => tree_sitter_just::LANGUAGE.into(),
        Language::Kotlin => tree_sitter_kotlin_ng::LANGUAGE.into(),
        Language::Lua => tree_sitter_lua::LANGUAGE.into(),
        Language::Markdown => return None,
        Language::Nginx => tree_sitter_nginx::LANGUAGE.into(),
        Language::Nix => tree_sitter_nix::LANGUAGE.into(),
        Language::Proto => tree_sitter_proto::LANGUAGE.into(),
        Language::Python => tree_sitter_python::LANGUAGE.into(),
        Language::Rust => tree_sitter_rust::LANGUAGE.into(),
        Language::Toml => tree_sitter_toml_ng::LANGUAGE.into(),
        Language::Tsx => tree_sitter_typescript::LANGUAGE_TSX.into(),
        Language::Typst => codebook_tree_sitter_typst::LANGUAGE.into(),
        Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        Language::Yaml => tree_sitter_yaml::LANGUAGE.into(),
    })
}
