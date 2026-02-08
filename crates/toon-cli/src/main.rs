use std::fs::File;
use std::io::{Read, stdin};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum DelimArg {
    Comma,
    Tab,
    Pipe,
}

#[derive(Parser, Debug)]
#[command(
    name = "toon-cli",
    about = "CLI for JSON ↔ TOON conversion (WIP)",
    version
)]
struct Args {
    /// Decode TOON to JSON (default encodes JSON to TOON)
    #[arg(short, long)]
    decode: bool,

    /// Delimiter for TOON arrays (when applicable)
    #[arg(long, value_enum, default_value_t = DelimArg::Comma)]
    delimiter: DelimArg,

    /// Strict mode validation (enabled by default per spec)
    #[arg(long, default_value_t = true)]
    strict: bool,

    /// Pretty-print JSON on output (when decoding)
    #[arg(long, default_value_t = false)]
    pretty: bool,

    /// Input file (defaults to stdin)
    input: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut buf = String::new();
    match &args.input {
        Some(path) => {
            let mut f = File::open(path)?;
            f.read_to_string(&mut buf)?;
        }
        None => {
            stdin().read_to_string(&mut buf)?;
        }
    }

    let delimiter = match args.delimiter {
        DelimArg::Comma => toon::Delimiter::Comma,
        DelimArg::Tab => toon::Delimiter::Tab,
        DelimArg::Pipe => toon::Delimiter::Pipe,
    };
    let options = toon::Options {
        delimiter,
        strict: args.strict,
        indent: 2,
        key_folding: toon::KeyFolding::Off,
        flatten_depth: None,
        expand_paths: toon::ExpandPaths::Off,
    };

    if args.decode {
        let value: serde_json::Value = toon::decode_from_str(&buf, &options)?;
        if args.pretty {
            println!("{}", serde_json::to_string_pretty(&value)?);
        } else {
            println!("{}", serde_json::to_string(&value)?);
        }
    } else {
        let value: serde_json::Value = serde_json::from_str(&buf)?;
        let out = toon::encode_to_string(&value, &options)?;
        println!("{}", out);
    }

    Ok(())
}
