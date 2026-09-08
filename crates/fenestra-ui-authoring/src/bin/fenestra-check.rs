#![forbid(unsafe_code)]

//! Command-line validation for experimental format-2 authoring sources.

use std::env;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use fenestra_ui_authoring::prototype::{
    AuthoringDiagnosticV2, AuthoringLimitKindV2, DiagnosticLocationV2, FenSourceV2,
    REFERENCE_AUTHORING_LIMITS_V2, canonical_rust_v2, compile_fen_v2,
};
use fenestra_ui_ir::prototype::SourceId;

const USAGE: &str = "Usage: fenestra-check [--emit-rust] <file.fen>

Validate experimental format-2 .fen syntax and typed IR.
Use --emit-rust to write canonical Rust to stdout instead of a summary.
Diagnostics use one-based lines and byte columns, plus zero-based byte ranges.
Uses REFERENCE_AUTHORING_LIMITS_V2: 8192 source bytes, 2048 tokens,
7 templates, 8 initial instances, and 107789 generated Rust bytes.
This bounded prototype does not open a window or execute application code.";

enum Arguments {
    Help,
    Check { path: PathBuf, emit_rust: bool },
}

fn arguments() -> Result<Arguments, ()> {
    let mut path = None;
    let mut emit_rust = false;
    let mut positional_only = false;
    for argument in env::args_os().skip(1) {
        if !positional_only && argument == "--" {
            positional_only = true;
        } else if !positional_only && (argument == "--help" || argument == "-h") {
            return Ok(Arguments::Help);
        } else if !positional_only && argument == "--emit-rust" && !emit_rust {
            emit_rust = true;
        } else if (!positional_only && argument.to_string_lossy().starts_with('-'))
            || path.replace(PathBuf::from(argument)).is_some()
        {
            return Err(());
        }
    }
    Ok(Arguments::Check {
        path: path.ok_or(())?,
        emit_rust,
    })
}

fn main() -> ExitCode {
    let (path, emit_rust) = match arguments() {
        Ok(Arguments::Help) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Ok(Arguments::Check { path, emit_rust }) => (path, emit_rust),
        Err(()) => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let maximum = REFERENCE_AUTHORING_LIMITS_V2.limit(AuthoringLimitKindV2::FenSourceBytes);
    let mut bytes = Vec::new();
    let read =
        File::open(&path).and_then(|file| file.take(maximum as u64 + 1).read_to_end(&mut bytes));
    if let Err(error) = read {
        eprintln!("{}: error: cannot read source: {error}", path.display());
        return ExitCode::FAILURE;
    }
    let result = compile_fen_v2(
        FenSourceV2::new(SourceId::new(1), &bytes),
        REFERENCE_AUTHORING_LIMITS_V2,
    )
    .and_then(|compiled| {
        let generated = canonical_rust_v2(&compiled, REFERENCE_AUTHORING_LIMITS_V2)?;
        Ok((compiled, generated))
    });
    let (compiled, generated) = match result {
        Ok(result) => result,
        Err(diagnostic) => {
            report(&path, &bytes, diagnostic);
            return ExitCode::FAILURE;
        }
    };
    let mut output = io::stdout().lock();
    let written = if emit_rust {
        output.write_all(generated.as_str().as_bytes())
    } else {
        writeln!(
            output,
            "ok|format=2|source-bytes={}|anchors={}|generated-bytes={}",
            bytes.len(),
            compiled.source_map().entries().len(),
            generated.as_str().len()
        )
    };
    if let Err(error) = written {
        eprintln!("fenestra-check: error: cannot write output: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn report(path: &std::path::Path, bytes: &[u8], diagnostic: AuthoringDiagnosticV2) {
    let origin = match diagnostic.location() {
        DiagnosticLocationV2::Physical(origin)
        | DiagnosticLocationV2::Anchored {
            physical: origin, ..
        } => origin,
    };
    let (start, end) = origin.fen_byte_range().unwrap_or((0, 0));
    // Count bytes directly so invalid UTF-8 and partial code points are reportable.
    let prefix = &bytes[..(start as usize).min(bytes.len())];
    let line = prefix.iter().filter(|&&byte| byte == b'\n').count() + 1;
    let column = prefix
        .iter()
        .rposition(|&byte| byte == b'\n')
        .map_or(prefix.len() + 1, |newline| prefix.len() - newline);
    eprintln!(
        "{}:{line}:{column}: error: {diagnostic} (bytes={start}..{end}; kind={:?})",
        path.display(),
        diagnostic.kind()
    );
}
