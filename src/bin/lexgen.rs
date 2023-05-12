use atrium_codegen::genapi;
use clap::Parser;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    lexdir: PathBuf,
    #[arg(short, long, default_value = "./atrium-api/src")]
    outdir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut results = Vec::new();
    for prefix in ["app.bsky", "com.atproto"] {
        results.extend(genapi(&args.lexdir, &args.outdir, prefix)?);
    }
    for path in &results {
        match Command::new("rustfmt")
            .arg("--edition")
            .arg("2021")
            .arg(path.as_ref())
            .status()
        {
            Ok(status) if status.success() => {}
            _ => {
                eprintln!("Failed to run rustfmt on {}", path.as_ref().display());
            }
        }
    }
    Ok(())
}
