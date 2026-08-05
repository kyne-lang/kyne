//! `kyne_cli` - the `kyne` binary.
//!
//! Specified by docs/TOOLCHAIN.md §3 (CLI Philosophy) and §9 (CLI
//! Commands). Kyne exposes exactly one executable; every capability -
//! build, check, fmt, test, doc, and more - is a subcommand of this
//! binary, composed entirely from `kyne_driver` and the `tools/` crates,
//! per docs/COMPILER_ARCHITECTURE.md §22's library-first architecture.
//!
//! This binary implements exactly the four subcommands the v0.1 release
//! gate requires ("Basic CLI operational", per docs/ROADMAP.md §25):
//! `new`, `init`, `fmt`, and `check`. Every other subcommand in
//! docs/TOOLCHAIN.md §9 depends on a later compiler stage that does not
//! exist yet and is out of scope here.

mod scaffold;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kyne_diagnostics::Diagnostic;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(run(&args) as u8)
}

fn run(args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("new") => cmd_new(&args[1..]),
        Some("init") => cmd_init(&args[1..]),
        Some("fmt") => cmd_fmt(&args[1..]),
        Some("check") => cmd_check(&args[1..]),
        Some("--help") | Some("-h") | None => {
            print_usage();
            0
        }
        Some(other) => {
            eprintln!("kyne: unknown subcommand `{other}`");
            eprintln!();
            print_usage();
            2
        }
    }
}

fn print_usage() {
    println!(
        "\
kyne - the Kyne compiler and toolchain

USAGE:
    kyne <COMMAND>

COMMANDS:
    new <name> [--template <template>]   Create a new project in a new directory
    init [--template <template>]         Initialize a project in the current directory
    fmt [--check]                        Format every .kyn file under src/
    check                                Run every compiler check currently implemented

TEMPLATES:
    hello (default), token, nft, oracle

More commands (build, run, test, doc, ...) are added as their underlying
compiler stages are implemented - see docs/TOOLCHAIN.md §9 for the full,
eventual command set."
    );
}

// ---- `kyne new` / `kyne init` ----

/// Parses `--template <name>` out of `args`, returning the remaining
/// positional arguments alongside it. Returns `Err(exit_code)` on a
/// usage error.
fn parse_template_flag(args: &[String]) -> Result<(&'static scaffold::Template, Vec<&str>), i32> {
    let mut template_name = "hello";
    let mut positionals = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--template" => {
                i += 1;
                match args.get(i) {
                    Some(t) => template_name = t,
                    None => {
                        eprintln!("kyne: --template requires a value");
                        return Err(2);
                    }
                }
            }
            other => positionals.push(other),
        }
        i += 1;
    }
    match scaffold::find_template(template_name) {
        Some(t) => Ok((t, positionals)),
        None => {
            eprintln!(
                "kyne: unknown template `{template_name}` (available: {})",
                scaffold::TEMPLATES
                    .iter()
                    .map(|t| t.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            Err(2)
        }
    }
}

fn cmd_new(args: &[String]) -> i32 {
    let (template, positionals) = match parse_template_flag(args) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let name = match positionals.as_slice() {
        [name] => *name,
        [] => {
            eprintln!("kyne: `kyne new` requires a project name");
            return 2;
        }
        _ => {
            eprintln!("kyne: `kyne new` takes exactly one project name");
            return 2;
        }
    };
    let dir = Path::new(name);
    if dir.exists()
        && dir
            .read_dir()
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
    {
        eprintln!("kyne: `{name}` already exists and is not empty");
        return 2;
    }
    match scaffold::scaffold(dir, name, template) {
        Ok(()) => {
            println!("Created `{name}` from template `{}`.", template.name);
            0
        }
        Err(e) => {
            eprintln!("kyne: failed to create project: {e}");
            2
        }
    }
}

fn cmd_init(args: &[String]) -> i32 {
    let (template, positionals) = match parse_template_flag(args) {
        Ok(v) => v,
        Err(code) => return code,
    };
    if !positionals.is_empty() {
        eprintln!("kyne: `kyne init` takes no positional arguments");
        return 2;
    }
    let dir = Path::new(".");
    if dir.join("Kyne.toml").exists() {
        eprintln!("kyne: this directory already contains a Kyne.toml");
        return 2;
    }
    let name = std::env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "project".to_string());
    match scaffold::scaffold(dir, &name, template) {
        Ok(()) => {
            println!(
                "Initialized a `{}` project in the current directory.",
                template.name
            );
            0
        }
        Err(e) => {
            eprintln!("kyne: failed to initialize project: {e}");
            2
        }
    }
}

// ---- `kyne fmt` ----

fn cmd_fmt(args: &[String]) -> i32 {
    let check_only = args.iter().any(|a| a == "--check");
    if args.iter().any(|a| a != "--check") {
        eprintln!("kyne: `kyne fmt` only accepts `--check`");
        return 2;
    }

    let files = collect_kyn_files(Path::new("src"));
    if files.is_empty() {
        eprintln!("kyne: no .kyn files found under src/ - is this a Kyne project? (`kyne init` to create one)");
        return 2;
    }

    let mut needs_changes = false;
    let mut had_error = false;
    for file in &files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("kyne: failed to read {}: {e}", file.display());
                had_error = true;
                continue;
            }
        };
        match kyne_formatter::format(&source) {
            Ok(formatted) if formatted == source => {}
            Ok(formatted) => {
                needs_changes = true;
                if check_only {
                    println!("would reformat {}", file.display());
                } else if let Err(e) = std::fs::write(file, &formatted) {
                    eprintln!("kyne: failed to write {}: {e}", file.display());
                    had_error = true;
                } else {
                    println!("reformatted {}", file.display());
                }
            }
            Err(diagnostics) => {
                had_error = true;
                print_diagnostics(&diagnostics, &source);
            }
        }
    }

    if had_error || (check_only && needs_changes) {
        1
    } else {
        0
    }
}

// ---- `kyne check` ----

fn cmd_check(args: &[String]) -> i32 {
    if !args.is_empty() {
        eprintln!("kyne: `kyne check` takes no arguments");
        return 2;
    }

    let files = collect_kyn_files(Path::new("src"));
    if files.is_empty() {
        eprintln!("kyne: no .kyn files found under src/ - is this a Kyne project? (`kyne init` to create one)");
        return 2;
    }

    let mut had_error = false;
    for file in &files {
        let source = match std::fs::read_to_string(file) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("kyne: failed to read {}: {e}", file.display());
                had_error = true;
                continue;
            }
        };
        let diagnostics = kyne_driver::check(&source, &file.display().to_string());
        if !diagnostics.is_empty() {
            had_error = true;
            print_diagnostics(&diagnostics, &source);
        }
    }

    if had_error {
        1
    } else {
        println!(
            "kyne check: no errors found in {} file(s) (parse-only for now - name resolution, type checking, and semantic analysis are not implemented yet).",
            files.len()
        );
        0
    }
}

// ---- Shared helpers ----

fn print_diagnostics(diagnostics: &[Diagnostic], source: &str) {
    for diagnostic in diagnostics {
        eprintln!("{}", diagnostic.render(source));
    }
}

fn collect_kyn_files(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    collect_kyn_files_into(dir, &mut out);
    out
}

fn collect_kyn_files_into(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_kyn_files_into(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("kyn") {
            out.push(path);
        }
    }
}
