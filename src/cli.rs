use std::ffi::OsString;
use std::path::PathBuf;

use clap::error::ErrorKind;
use clap::{Parser, Subcommand};

use crate::{adapters, render, walk};

#[derive(Debug, Parser)]
#[command(name = "lupa", about = "Agent-first source navigation")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Map {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    Show {
        file: PathBuf,
        #[arg(required = true)]
        keys: Vec<String>,
    },
    Digest {
        #[arg(required = true)]
        paths: Vec<PathBuf>,
    },
    Keys {
        file: PathBuf,
    },
}

pub fn run<I>(args: I) -> Result<String, String>
where
    I: IntoIterator<Item = OsString>,
{
    let args = normalize_args(args);
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(err)
            if matches!(
                err.kind(),
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion
            ) =>
        {
            return Ok(err.to_string());
        }
        Err(err) => return Err(err.to_string()),
    };
    match cli.command {
        Commands::Map { paths } => map(paths),
        Commands::Show { file, keys } => show(file, keys),
        Commands::Digest { paths } => digest(paths),
        Commands::Keys { file } => keys(file),
    }
}

fn normalize_args<I>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = OsString>,
{
    let mut args = args.into_iter().collect::<Vec<_>>();
    if args.len() >= 2 {
        let first = args[1].to_string_lossy();
        let known = matches!(
            first.as_ref(),
            "map" | "show" | "digest" | "keys" | "help" | "-h" | "--help"
        );
        if !known {
            args.insert(1, OsString::from("map"));
        }
    }
    args
}

fn map(paths: Vec<PathBuf>) -> Result<String, String> {
    let mut out = String::new();
    for path in paths {
        if !path.exists() {
            out.push_str(&format!("# error: path not found: {}\n", path.display()));
            continue;
        }
        if path.is_file() {
            match adapters::parse_file(&path) {
                Ok(file) => render::render_map(&file, &mut out).map_err(render_error)?,
                Err(err) => push_error(&mut out, &err),
            }
        } else {
            for file_path in walk::collect_supported_files(&[path]) {
                match adapters::parse_file(&file_path) {
                    Ok(file) => render::render_map(&file, &mut out).map_err(render_error)?,
                    Err(err) => push_error(&mut out, &err),
                }
            }
        }
    }
    Ok(out)
}

fn show(file: PathBuf, keys: Vec<String>) -> Result<String, String> {
    if !file.exists() {
        return Ok(format!("# error: path not found: {}\n", file.display()));
    }
    match adapters::parse_file(&file) {
        Ok(file) => {
            let mut out = String::new();
            render::render_show(&file, &keys, &mut out).map_err(render_error)?;
            Ok(out)
        }
        Err(err) => Ok(format!("{err}\n")),
    }
}

fn digest(paths: Vec<PathBuf>) -> Result<String, String> {
    let mut out = String::new();
    let mut files = Vec::new();
    for path in paths {
        if !path.exists() {
            out.push_str(&format!("# error: path not found: {}\n", path.display()));
            continue;
        }
        if path.is_file() {
            match adapters::parse_file(&path) {
                Ok(file) => files.push(file),
                Err(err) => push_error(&mut out, &err),
            }
        } else {
            for file_path in walk::collect_supported_files(&[path]) {
                match adapters::parse_file(&file_path) {
                    Ok(file) => files.push(file),
                    Err(err) => push_error(&mut out, &err),
                }
            }
        }
    }
    render::render_digest(&files, &mut out).map_err(render_error)?;
    Ok(out)
}

fn keys(file: PathBuf) -> Result<String, String> {
    if !file.exists() {
        return Ok(format!("# error: path not found: {}\n", file.display()));
    }
    match adapters::parse_file(&file) {
        Ok(file) => {
            let mut out = String::new();
            render::render_keys(&file, &mut out).map_err(render_error)?;
            Ok(out)
        }
        Err(err) => Ok(format!("{err}\n")),
    }
}

fn push_error(out: &mut String, err: &str) {
    out.push_str(err.trim_end());
    out.push('\n');
}

fn render_error(_: std::fmt::Error) -> String {
    "failed to render output".to_owned()
}
