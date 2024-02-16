use atrium_codegen::genapi;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    lexdir: PathBuf,
    #[arg(short, long, default_value = "../atrium-api/src")]
    outdir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let results = genapi(&args.lexdir, &args.outdir, &["app.bsky", "com.atproto"])?;
    for path in &results {
        println!(
            "{} ({} bytes)",
            path.as_ref().display(),
            fs::metadata(path.as_ref())?.len()
        );
    }
    Ok(())
}
