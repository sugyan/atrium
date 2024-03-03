mod fs;
mod generator;
mod schema;
mod token_stream;

use crate::generator::{generate_client, generate_modules, generate_records, generate_schemas};
use atrium_lex::LexiconDoc;
use itertools::Itertools;
use serde_json::from_reader;
use std::error::Error;
use std::fs::File;
use std::path::{Path, PathBuf};

pub fn genapi(
    lexdir: impl AsRef<Path>,
    outdir: impl AsRef<Path>,
    prefixes: &[&str],
) -> Result<Vec<impl AsRef<Path>>, Box<dyn Error>> {
    let lexdir = lexdir.as_ref().canonicalize()?;
    let outdir = outdir.as_ref().canonicalize()?;
    let paths = fs::find_schemas(&lexdir)?;
    let mut schemas = Vec::with_capacity(paths.len());
    for path in &paths {
        schemas.push(from_reader::<_, LexiconDoc>(File::open(path)?)?);
    }
    let mut results = Vec::new();
    for &prefix in prefixes {
        let targets = schemas
            .iter()
            .filter(|schema| schema.id.starts_with(prefix))
            .collect_vec();
        results.extend(gen(&outdir, &targets)?);
    }
    results.push(generate_records(&outdir, &schemas)?);
    results.push(generate_client(&outdir, &schemas)?);
    results.extend(generate_modules(&outdir, &schemas)?);
    Ok(results)
}

fn gen(outdir: &Path, schemas: &[&LexiconDoc]) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut results = Vec::new();
    for &schema in schemas {
        results.extend(generate_schemas(schema, outdir)?);
    }
    Ok(results)
}
