use clap::{Parser, ValueEnum};
use std::{
    path::Path,
    process::{Command, exit},
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    #[arg(short, long)]
    release: bool,

    #[arg(short, long, value_enum, default_value = "web")]
    platform: Platform,

    #[arg(short, long)]
    serve: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Platform {
    Web,
    Desktop,
}

fn main() {
    let args = Args::parse();

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let web_worker_path = workspace_root.join("string_art_gui/string_art_gui_web_worker_registrar");
    let string_art_gui_path = workspace_root.join("string_art_gui");
    let output_path = string_art_gui_path.join("assets/pkg");

    if Platform::Web == args.platform {
        println!("Running wasm-pack for web platform...");

        let mut wasm_pack = Command::new("wasm-pack");
        wasm_pack
            .arg("build")
            .arg("--target")
            .arg("no-modules")
            .arg("--out-dir")
            .arg(&output_path);

        if !args.release {
            wasm_pack.arg("--no-opt");
        }

        let status = wasm_pack
            .current_dir(&web_worker_path)
            .status()
            .expect("Error: Failed executing wasm-pack");

        if !status.success() {
            eprintln!("Error: wasm-pack completed with error");
            exit(1);
        }

        if let Err(err) = std::fs::remove_file(&output_path.join(".gitignore")) {
            eprintln!("Warning: .gitignore was not deleted, {err}");
        }
    }

    let mut dx = Command::new("dx");
    dx.arg(match args.serve {
        true => "serve",
        false => "build",
    })
    .arg("--platform")
    .arg(match args.platform {
        Platform::Web => "web",
        Platform::Desktop => "desktop",
    });

    if args.release {
        dx.arg("--release");
    }

    let status = dx
        .current_dir(&string_art_gui_path)
        .status()
        .expect("Error: Failed executing dx");

    if !status.success() {
        eprintln!("Error: dx completed with error");
        exit(1);
    }
}
