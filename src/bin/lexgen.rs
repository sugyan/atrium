use std::path::PathBuf;

use atrium_codegen::genapi;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    lexdir: PathBuf,
    #[arg(short, long, default_value = "./atrium-api/src")]
    outdir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    for prefix in ["app.bsky", "com.atproto"] {
        genapi(&args.lexdir, &args.outdir, prefix)?;
    }
    Ok(())
}
