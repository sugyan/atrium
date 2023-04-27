mod code_writer;
mod fs;
mod generator;
mod schema;

use crate::generator::{generate_code, generate_modules, generate_records, generate_traits};
use atrium_lex::LexiconDoc;
use serde_json::from_reader;
use std::fs::File;
use std::io::Result;
use std::path::Path;

pub fn genapi(lexdir: impl AsRef<Path>, outdir: impl AsRef<Path>, prefix: &str) -> Result<()> {
    let lexdir = lexdir.as_ref().canonicalize()?;
    let outdir = outdir.as_ref().canonicalize()?;
    let paths = fs::find_schemas(&lexdir)?;
    let mut schemas = Vec::with_capacity(paths.len());
    for path in &paths {
        schemas.push(from_reader::<_, LexiconDoc>(File::open(path)?)?);
    }
    for schema in schemas
        .iter()
        .filter(|schema| schema.id.starts_with(prefix))
    {
        generate_code(schema, &outdir)?;
    }
    generate_records(&outdir, &schemas)?;
    generate_traits(&outdir, &schemas)?;
    generate_modules(&outdir)?;
    Ok(())
}
