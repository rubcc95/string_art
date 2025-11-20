//! xtask - helper to build wasm bindings and run the Flutter GUI with improved diagnostics.
//!
//! This file replaces the original main.rs to provide:
//! - clearer diagnostics about PATH and which `flutter` would be used,
//! - multiple invocation strategies on Windows to handle .bat/.cmd launchers,
//! - helpful hints when execution fails.

use clap::{Parser, ValueEnum};
use std::{
    env,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, exit},
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Build in release mode
    #[arg(short, long)]
    release: bool,

    /// Target platform: web or desktop
    #[arg(short, long, value_enum, default_value = "web")]
    platform: Platform,

    /// Only run flutter build (don't run)
    #[arg(short, long)]
    build: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Platform {
    Web,
    Desktop,
}

fn main() {
    let args = Args::parse();

    // Paths relative to the workspace root (CARGO_MANIFEST_DIR points to xtask crate)
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("CARGO_MANIFEST_DIR should have a parent");
    let bindings_path = workspace_root.join("string_art_bindings");
    let string_art_gui_path = workspace_root.join("string_art_gui");
    let output_path = string_art_gui_path.join("web/pkg");

    // If web target, run wasm-pack first (best-effort)
    if Platform::Web == args.platform {
        eprintln!("INFO: Running wasm-pack for web platform...");
        let mut wasm_pack = Command::new("wasm-pack");
        wasm_pack
            .arg("build")
            .arg("--target")
            .arg("web")
            .arg("--out-dir")
            .arg("--release")
            .arg(&output_path);

        if !args.release {
            wasm_pack.arg("--no-opt");
        }

        match wasm_pack.current_dir(&bindings_path).status() {
            Ok(status) => {
                if !status.success() {
                    eprintln!(
                        "ERROR: wasm-pack completed with non-zero exit code: {:?}",
                        status.code()
                    );
                    exit(1);
                }
            }
            Err(e) => {
                eprintln!("ERROR: Failed executing wasm-pack: {e}");
                exit(1);
            }
        }

        // Remove .gitignore left by wasm-pack if present (best-effort)
        if let Err(err) = fs::remove_file(output_path.join(".gitignore")) {
            eprintln!(
                "WARN: Could not remove {}: {}",
                output_path.join(".gitignore").display(),
                err
            );
        } else {
            eprintln!("DEBUG: Removed generated .gitignore");
        }

        eprintln!(
            "INFO: Your wasm pkg is ready to publish at {}",
            output_path.display()
        );
    }

    // Print diagnostic environment information so user can see what the process sees
    eprintln!("DEBUG: current_dir = {}", string_art_gui_path.display());
    eprintln!(
        "DEBUG: PATH = {}",
        env::var("PATH").unwrap_or_else(|_| "<no PATH>".to_string())
    );
    if cfg!(windows) {
        eprintln!(
            "DEBUG: PATHEXT = {}",
            env::var("PATHEXT").unwrap_or_else(|_| "<no PATHEXT>".to_string())
        );
    }

    // Print resolution from OS helpers (where / command -v)
    if cfg!(windows) {
        match Command::new("where").arg("flutter").output() {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout);
                eprintln!("DEBUG: where flutter -> {}", s.trim());
            }
            Err(e) => eprintln!("DEBUG: failed to run `where flutter`: {e}"),
        }
    } else {
        match Command::new("sh")
            .arg("-c")
            .arg("command -v flutter")
            .output()
        {
            Ok(out) => {
                let s = String::from_utf8_lossy(&out.stdout);
                eprintln!("DEBUG: command -v flutter -> {}", s.trim());
            }
            Err(e) => eprintln!("DEBUG: failed to run `command -v flutter`: {e}"),
        }
    }

    // Build flutter args according to CLI args
    let action = if args.build { "build" } else { "run" };
    let device = match args.platform {
        Platform::Web => "edge",
        Platform::Desktop => "windows",
    };
    let mut flutter_args: Vec<OsString> = Vec::new();
    flutter_args.push(OsString::from(action));
    flutter_args.push(OsString::from("-d"));
    flutter_args.push(OsString::from(device));
    if args.release {
        flutter_args.push(OsString::from("--release"));
    }

    // Try invocation strategies in order:
    // 1) Direct `flutter` (common on Unix and many Windows setups)
    // 2) On Windows: `cmd /C flutter ...` (useful when flutter is a batch script)
    // 3) Absolute path to discovered flutter (where/command-v scan)
    // Stop on first successful run.

    // Attempt 1: direct name
    match run_with(program("flutter"), &flutter_args, &string_art_gui_path) {
        Ok(status) => {
            if status.success() {
                eprintln!("INFO: `flutter {}` executed successfully (direct).", action);
                return;
            } else {
                eprintln!(
                    "WARN: `flutter {}` returned exit code {:?}",
                    action,
                    status.code()
                );
                // Keep trying fallbacks
            }
        }
        Err(e) => {
            eprintln!("DEBUG: direct spawn of `flutter` failed: {e}");
        }
    }

    // Attempt 2: Windows-specific fallback using cmd /C flutter ...
    if cfg!(windows) {
        eprintln!("INFO: trying Windows fallback: `cmd /C flutter ...`");
        // Build args: /C flutter <flutter_args...>
        let mut cmd_args: Vec<OsString> = Vec::new();
        cmd_args.push(OsString::from("/C"));
        cmd_args.push(OsString::from("flutter"));
        cmd_args.extend(flutter_args.iter().cloned());

        match run_with(program("cmd"), &cmd_args, &string_art_gui_path) {
            Ok(status) => {
                if status.success() {
                    eprintln!("INFO: `cmd /C flutter {}` executed successfully.", action);
                    return;
                } else {
                    eprintln!(
                        "WARN: `cmd /C flutter {}` returned exit code {:?}",
                        action,
                        status.code()
                    );
                }
            }
            Err(e) => {
                eprintln!("DEBUG: `cmd /C flutter` spawn failed: {e}");
            }
        }
    }

    // Attempt 3: try absolute path found by PATH scanning or helper
    if let Some(abs) = resolve_flutter_path() {
        eprintln!(
            "INFO: attempting to run resolved flutter at: {}",
            abs.display()
        );
        let mut abs_args: Vec<OsString> = flutter_args.iter().cloned().collect();
        match run_with(program(abs), &abs_args, &string_art_gui_path) {
            Ok(status) => {
                if status.success() {
                    eprintln!("INFO: resolved flutter executed successfully.");
                    return;
                } else {
                    eprintln!(
                        "WARN: resolved flutter returned exit code {:?}",
                        status.code()
                    );
                }
            }
            Err(e) => {
                eprintln!("DEBUG: executing resolved flutter failed: {e}");
            }
        }
    } else {
        eprintln!("DEBUG: no resolved absolute flutter path found by scanning PATH/helpers.");
    }

    // Final helpful failure output
    eprintln!();
    eprintln!("FATAL: failed to execute `flutter {}`.", action);
    eprintln!("Last diagnostics printed above. Common causes & fixes:");
    eprintln!(
        "- Ensure Flutter is installed and `flutter` is on the PATH visible to the process running `cargo`."
    );
    eprintln!(
        "- If you added Flutter to PATH recently, restart your terminal/IDE so the environment is propagated."
    );
    eprintln!(
        "- On Windows, flutter may be a batch file; the `cmd /C` fallback is attempted above."
    );
    eprintln!("- To force a specific Flutter binary, run cargo with PATH explicitly:");
    eprintln!("    Windows cmd:  set PATH=C:\\\\flutter\\\\bin;%PATH% && cargo run -p xtask");
    eprintln!(
        "    PowerShell:    $env:PATH = 'C:\\\\flutter\\\\bin;' + $env:PATH; cargo run -p xtask"
    );
    eprintln!("    Unix:         PATH=/path/to/flutter/bin:$PATH cargo run -p xtask");
    eprintln!();
    eprintln!("If you paste the debug output (the DEBUG: lines) I can help further.");

    exit(1);
}

/// Helper to create a program identifier that Command::new can accept.
/// Accepts either a &str or PathBuf via Into.
fn program<P: Into<Program>>(p: P) -> Program {
    p.into()
}

/// Small wrapper type to let run_with accept both &str and PathBuf as program spec.
enum Program {
    Name(String),
    Path(PathBuf),
}

impl From<&str> for Program {
    fn from(s: &str) -> Program {
        Program::Name(s.to_string())
    }
}

impl From<String> for Program {
    fn from(s: String) -> Program {
        Program::Name(s)
    }
}

impl From<PathBuf> for Program {
    fn from(p: PathBuf) -> Program {
        Program::Path(p)
    }
}

impl From<OsString> for Program {
    fn from(s: OsString) -> Program {
        if let Some(st) = s.to_str() {
            Program::Name(st.to_string())
        } else {
            Program::Path(PathBuf::from(s))
        }
    }
}

/// Run a program (name or absolute path) with args and working directory.
/// Returns the command ExitStatus or an io::Error if spawning failed.
fn run_with<P: Into<Program>>(prog: P, args: &[OsString], cwd: &Path) -> io::Result<ExitStatus> {
    match prog.into() {
        Program::Name(name) => {
            let mut cmd = Command::new(name);
            for a in args {
                cmd.arg(a);
            }
            cmd.current_dir(cwd).status()
        }
        Program::Path(path) => {
            let mut cmd = Command::new(path);
            for a in args {
                cmd.arg(a);
            }
            cmd.current_dir(cwd).status()
        }
    }
}

/// Try to find flutter using OS helper commands first, then fall back to scanning PATH.
/// Returns Some(PathBuf) with the first absolute candidate found.
fn resolve_flutter_path() -> Option<PathBuf> {
    // 1) OS helper
    if cfg!(windows) {
        if let Ok(out) = Command::new("where").arg("flutter").output() {
            let s = String::from_utf8_lossy(&out.stdout);
            for line in s.lines() {
                let t = line.trim();
                if !t.is_empty() {
                    return Some(PathBuf::from(t));
                }
            }
        }
    } else {
        if let Ok(out) = Command::new("sh")
            .arg("-c")
            .arg("command -v flutter")
            .output()
        {
            let s = String::from_utf8_lossy(&out.stdout);
            let t = s.trim();
            if !t.is_empty() {
                return Some(PathBuf::from(t));
            }
        }
    }

    // 2) Scan PATH ourselves
    find_executable_in_path("flutter")
}

/// Scan PATH for an executable named `name`. On Windows, consider PATHEXT extensions.
fn find_executable_in_path(name: &str) -> Option<PathBuf> {
    let path_os = env::var_os("PATH")?;
    let paths = env::split_paths(&path_os);

    #[cfg(windows)]
    let exts: Vec<String> = env::var("PATHEXT")
        .map(|pe| {
            pe.split(';')
                .filter_map(|s| {
                    let s = s.trim();
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.to_string())
                    }
                })
                .collect()
        })
        .unwrap_or_else(|_| vec![".EXE".into(), ".BAT".into(), ".CMD".into(), ".COM".into()]);

    #[cfg(not(windows))]
    let exts: Vec<String> = vec!["".into()];

    for dir in paths {
        // Compose candidate without extension first
        let base = dir.join(name);
        #[cfg(windows)]
        {
            for ext in &exts {
                // Ensure extension begins with dot
                let ext_str = if ext.starts_with('.') {
                    ext.clone()
                } else {
                    format!(".{}", ext)
                };
                let cand = if base.extension().is_some() {
                    base.clone()
                } else {
                    base.with_extension(ext_str.trim_start_matches('.'))
                };
                if cand.exists() {
                    return Some(cand);
                }
            }
        }
        #[cfg(not(windows))]
        {
            let cand = base.clone();
            if cand.exists() {
                // on unix check executable bit
                if let Ok(meta) = fs::metadata(&cand) {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if meta.permissions().mode() & 0o111 != 0 {
                            return Some(cand);
                        }
                    }
                    #[cfg(not(unix))]
                    {
                        // conservative fallback: return if exists
                        return Some(cand);
                    }
                }
            }
        }
    }

    None
}
