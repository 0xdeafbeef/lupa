use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    C,
    Cpp,
    Go,
    JavaScript,
    Json,
    Jsx,
    Markdown,
    Nix,
    Python,
    Rust,
    Tsx,
    Toml,
    TypeScript,
    Yaml,
}

impl Language {
    pub fn from_path(path: &std::path::Path) -> Option<Self> {
        let ext = path.extension().and_then(|ext| ext.to_str())?;
        match ext.to_ascii_lowercase().as_str() {
            "c" => Some(Self::C),
            "c++" | "cc" | "cpp" | "cxx" | "h" | "h++" | "hh" | "hpp" | "hxx" | "inl" | "ipp" => {
                Some(Self::Cpp)
            }
            "go" => Some(Self::Go),
            "js" | "cjs" | "mjs" => Some(Self::JavaScript),
            "json" => Some(Self::Json),
            "jsx" => Some(Self::Jsx),
            "md" | "markdown" | "mdx" => Some(Self::Markdown),
            "nix" => Some(Self::Nix),
            "py" | "pyi" => Some(Self::Python),
            "rs" => Some(Self::Rust),
            "tsx" => Some(Self::Tsx),
            "toml" => Some(Self::Toml),
            "ts" | "mts" | "cts" => Some(Self::TypeScript),
            "yaml" | "yml" => Some(Self::Yaml),
            _ => None,
        }
    }

    pub fn from_token(token: &str) -> Option<Self> {
        match token {
            "c" => Some(Self::C),
            "cpp" => Some(Self::Cpp),
            "go" => Some(Self::Go),
            "javascript" => Some(Self::JavaScript),
            "json" => Some(Self::Json),
            "jsx" => Some(Self::Jsx),
            "markdown" => Some(Self::Markdown),
            "nix" => Some(Self::Nix),
            "python" => Some(Self::Python),
            "rust" => Some(Self::Rust),
            "tsx" => Some(Self::Tsx),
            "toml" => Some(Self::Toml),
            "typescript" => Some(Self::TypeScript),
            "yaml" => Some(Self::Yaml),
            _ => None,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::C => f.write_str("c"),
            Self::Cpp => f.write_str("cpp"),
            Self::Go => f.write_str("go"),
            Self::JavaScript => f.write_str("javascript"),
            Self::Json => f.write_str("json"),
            Self::Jsx => f.write_str("jsx"),
            Self::Markdown => f.write_str("markdown"),
            Self::Nix => f.write_str("nix"),
            Self::Python => f.write_str("python"),
            Self::Rust => f.write_str("rust"),
            Self::Tsx => f.write_str("tsx"),
            Self::Toml => f.write_str("toml"),
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
