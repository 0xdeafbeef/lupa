use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Bash,
    C,
    Cmake,
    Cpp,
    Css,
    Diff,
    Dockerfile,
    Fish,
    Go,
    Ini,
    JavaScript,
    Json,
    Jsonc,
    Just,
    Jsx,
    Kotlin,
    Lua,
    Markdown,
    Nginx,
    Nix,
    Proto,
    Python,
    Rust,
    Svelte,
    Tsx,
    Toml,
    Typst,
    TypeScript,
    Yaml,
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        if let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) {
            match file_name {
                "justfile" | "Justfile" | "JUSTFILE" => return Some(Self::Just),
                "Dockerfile" | "dockerfile" => return Some(Self::Dockerfile),
                "CMakeLists.txt" => return Some(Self::Cmake),
                "nginx.conf" => return Some(Self::Nginx),
                ".gitconfig" | ".gitmodules" => return Some(Self::Ini),
                _ => {}
            }
        }

        let ext = path.extension().and_then(|ext| ext.to_str())?;
        match ext.to_ascii_lowercase().as_str() {
            "bash" | "sh" => Some(Self::Bash),
            "c" => Some(Self::C),
            "c++" | "cc" | "cpp" | "cxx" | "h" | "h++" | "hh" | "hpp" | "hxx" | "inl" | "ipp" => {
                Some(Self::Cpp)
            }
            "cmake" => Some(Self::Cmake),
            "css" => Some(Self::Css),
            "diff" | "patch" => Some(Self::Diff),
            "docker" | "dockerfile" => Some(Self::Dockerfile),
            "fish" => Some(Self::Fish),
            "go" => Some(Self::Go),
            "ini" => Some(Self::Ini),
            "js" | "cjs" | "mjs" => Some(Self::JavaScript),
            "json" => Some(Self::Json),
            "jsonc" => Some(Self::Jsonc),
            "just" => Some(Self::Just),
            "jsx" => Some(Self::Jsx),
            "kt" | "kts" => Some(Self::Kotlin),
            "lua" => Some(Self::Lua),
            "md" | "markdown" | "mdx" => Some(Self::Markdown),
            "nginx" => Some(Self::Nginx),
            "nix" => Some(Self::Nix),
            "proto" | "protobuf" => Some(Self::Proto),
            "py" | "pyi" => Some(Self::Python),
            "rs" => Some(Self::Rust),
            "svelte" => Some(Self::Svelte),
            "tsx" => Some(Self::Tsx),
            "toml" => Some(Self::Toml),
            "typ" => Some(Self::Typst),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "bash" => Some(Self::Bash),
            "c" => Some(Self::C),
            "cmake" => Some(Self::Cmake),
            "cpp" => Some(Self::Cpp),
            "css" => Some(Self::Css),
            "diff" => Some(Self::Diff),
            "dockerfile" => Some(Self::Dockerfile),
            "fish" => Some(Self::Fish),
            "go" => Some(Self::Go),
            "ini" => Some(Self::Ini),
            "javascript" => Some(Self::JavaScript),
            "json" => Some(Self::Json),
            "jsonc" => Some(Self::Jsonc),
            "just" => Some(Self::Just),
            "jsx" => Some(Self::Jsx),
            "kotlin" => Some(Self::Kotlin),
            "lua" => Some(Self::Lua),
            "markdown" => Some(Self::Markdown),
            "nginx" => Some(Self::Nginx),
            "nix" => Some(Self::Nix),
            "proto" => Some(Self::Proto),
            "python" => Some(Self::Python),
            "rust" => Some(Self::Rust),
            "svelte" => Some(Self::Svelte),
            "tsx" => Some(Self::Tsx),
            "toml" => Some(Self::Toml),
            "typst" => Some(Self::Typst),
            "typescript" => Some(Self::TypeScript),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bash => f.write_str("bash"),
            Self::C => f.write_str("c"),
            Self::Cmake => f.write_str("cmake"),
            Self::Cpp => f.write_str("cpp"),
            Self::Css => f.write_str("css"),
            Self::Diff => f.write_str("diff"),
            Self::Dockerfile => f.write_str("dockerfile"),
            Self::Fish => f.write_str("fish"),
            Self::Go => f.write_str("go"),
            Self::Ini => f.write_str("ini"),
            Self::JavaScript => f.write_str("javascript"),
            Self::Json => f.write_str("json"),
            Self::Jsonc => f.write_str("jsonc"),
            Self::Just => f.write_str("just"),
            Self::Jsx => f.write_str("jsx"),
            Self::Kotlin => f.write_str("kotlin"),
            Self::Lua => f.write_str("lua"),
            Self::Markdown => f.write_str("markdown"),
            Self::Nginx => f.write_str("nginx"),
            Self::Nix => f.write_str("nix"),
            Self::Proto => f.write_str("proto"),
            Self::Python => f.write_str("python"),
            Self::Rust => f.write_str("rust"),
            Self::Svelte => f.write_str("svelte"),
            Self::Tsx => f.write_str("tsx"),
            Self::Toml => f.write_str("toml"),
            Self::Typst => f.write_str("typst"),
            Self::TypeScript => f.write_str("typescript"),
            Self::Yaml => f.write_str("yaml"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Class,
    Enum,
    Field,
    Function,
    Heading,
    Impl,
    Interface,
    Method,
    Node,
    Struct,
    Trait,
    Type,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class => f.write_str("class"),
            Self::Enum => f.write_str("enum"),
            Self::Field => f.write_str("field"),
            Self::Function => f.write_str("function"),
            Self::Heading => f.write_str("heading"),
            Self::Impl => f.write_str("impl"),
            Self::Interface => f.write_str("interface"),
            Self::Method => f.write_str("method"),
            Self::Node => f.write_str("node"),
            Self::Struct => f.write_str("struct"),
            Self::Trait => f.write_str("trait"),
            Self::Type => f.write_str("type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineSpan {
    pub start_line: usize,
    pub end_line: usize,
}

impl LineSpan {
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
        }
    }
}

impl fmt::Display for LineSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_line == self.end_line {
            write!(f, "L{}", self.start_line)
        } else {
            write!(f, "L{}-L{}", self.start_line, self.end_line)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    pub key: String,
    pub kind: SymbolKind,
    pub name: String,
    pub signature: String,
    pub attributes: Vec<String>,
    pub visibility: Option<String>,
    pub range: LineSpan,
    pub body_range: Option<LineSpan>,
    pub parent_key: Option<String>,
    pub children: Vec<Symbol>,
}

impl Symbol {
    pub fn new(
        key: impl Into<String>,
        kind: SymbolKind,
        name: impl Into<String>,
        signature: impl Into<String>,
        range: LineSpan,
    ) -> Self {
        Self {
            key: key.into(),
            kind,
            name: name.into(),
            signature: signature.into(),
            attributes: Vec::new(),
            visibility: None,
            range,
            body_range: None,
            parent_key: None,
            children: Vec::new(),
        }
    }

    pub fn display_signature(&self) -> String {
        if self.attributes.is_empty() {
            return self.signature.clone();
        }
        format!("{} {}", self.attributes.join(" "), self.signature)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct FileMap {
    pub path: PathBuf,
    pub language: Language,
    pub source: String,
    pub line_count: usize,
    pub byte_count: usize,
    pub parse_errors: Vec<ParseError>,
    pub warnings: Vec<String>,
    pub symbols: Vec<Symbol>,
}

impl FileMap {
    pub fn new(path: PathBuf, language: Language, source: String, symbols: Vec<Symbol>) -> Self {
        let line_count = count_lines(&source);
        let byte_count = source.len();
        Self {
            path,
            language,
            source,
            line_count,
            byte_count,
            parse_errors: Vec::new(),
            warnings: Vec::new(),
            symbols,
        }
    }

    pub fn all_symbols(&self) -> Vec<&Symbol> {
        let mut out = Vec::new();
        collect_symbols(&self.symbols, &mut out);
        out
    }
}

fn collect_symbols<'a>(symbols: &'a [Symbol], out: &mut Vec<&'a Symbol>) {
    for symbol in symbols {
        out.push(symbol);
        collect_symbols(&symbol.children, out);
    }
}

fn count_lines(source: &str) -> usize {
    if source.is_empty() {
        0
    } else {
        source.lines().count()
    }
}
